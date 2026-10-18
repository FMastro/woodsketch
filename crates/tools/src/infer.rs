//! SketchUp's inference engine for tools that work from a base point (Line, Move, Tape
//! Measure): where the cursor's point lands, and why.
//!
//! In priority order:
//! 1. A locked direction (arrow keys, or Shift held while a direction shows): the point slides
//!    along it.
//! 2. A point on geometry: endpoints, midpoints, edges, guides ([`snap_feature`]).
//! 3. A direction through the base point: the red, green or blue axis, or parallel or
//!    perpendicular to the last edge the cursor passed over (magenta, as in SketchUp).
//! 4. Free, on the tool's fallback surface.
//!
//! Where nothing on geometry is snapped, the point also lines up with the last point the cursor
//! passed over ("From Point"): along an axis through that point, shown dotted.

use glam::{DVec3, Vec2};
use woodsketch_doc::{EPS, Model, VertexId};

use crate::Axis;
use crate::pick::{AXIS_RADIUS, Ray, Screen, Snap, SnapKind, closest_on_line, snap_feature};

/// What a direction follows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Along {
    Axis(Axis),
    /// Parallel to the remembered edge.
    Parallel,
    /// Perpendicular to the remembered edge, in the plane being drawn on.
    Perpendicular,
}

impl Along {
    /// SketchUp-style inference tooltip.
    pub fn label(self) -> &'static str {
        match self {
            Self::Axis(axis) => axis.label(),
            Self::Parallel => "Parallel to Edge",
            Self::Perpendicular => "Perpendicular to Edge",
        }
    }
}

/// A direction through the base point, and why.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Direction {
    pub along: Along,
    /// Unit length.
    pub dir: DVec3,
}

impl Direction {
    pub fn axis(axis: Axis) -> Self {
        Self { along: Along::Axis(axis), dir: axis.dir() }
    }
}

/// What the cursor recently passed over, which later inferences use (SketchUp "remembers" an
/// edge or point you hover), and whether Shift is held.
#[derive(Clone, Copy, Debug, Default)]
pub struct Hints {
    /// The last edge (or guide line) hovered: its direction.
    pub edge: Option<DVec3>,
    /// The last point hovered: an endpoint, midpoint, guide point or intersection.
    pub point: Option<DVec3>,
    /// The last snap on geometry, which wins over a new one that isn't clearly closer
    /// ("sticky", so the point doesn't flicker between two snaps near each other).
    pub last_snap: Option<Snap>,
    /// The last direction followed (an axis, parallel, perpendicular), which likewise holds
    /// against another that isn't clearly closer.
    pub last_direction: Option<Direction>,
}

impl Hints {
    /// Remember what a target snapped to and the direction it followed.
    pub fn note_target(&mut self, target: &Target) {
        self.note(target.snap.as_ref());
        self.last_direction = target.direction;
    }

    /// Remember what a snap is on.
    pub fn note(&mut self, snap: Option<&Snap>) {
        let Some(snap) = snap else {
            self.last_snap = None;
            return;
        };
        self.last_snap = Some(*snap);
        if let Some(dir) = snap.line {
            self.edge = Some(dir);
        }
        if matches!(
            snap.kind,
            SnapKind::Endpoint | SnapKind::Midpoint | SnapKind::GuidePoint | SnapKind::Intersection
        ) {
            self.point = Some(snap.point);
        }
    }
}

/// Where the point lands.
#[derive(Clone, Copy, Debug, Default)]
pub struct Target {
    pub point: DVec3,
    /// The direction it follows from the base point, if any.
    pub direction: Option<Direction>,
    /// The geometry it snapped to, if any.
    pub snap: Option<Snap>,
    /// "From Point": the remembered point it lines up with, and along which axis.
    pub from: Option<(DVec3, Axis)>,
}

/// Everything the inference needs besides the drawing and cursor.
pub struct Query<'a> {
    pub base: DVec3,
    pub hints: &'a Hints,
    pub lock: Option<Direction>,
    /// The plane being drawn on (for "perpendicular").
    pub plane_normal: DVec3,
    /// The last point, for when the cursor gives no answer (looking straight down a lock).
    pub current: DVec3,
}

/// Infer the point under `cursor`. `exclude` skips geometry (what's being moved); `fallback`
/// gives the free point (a face under the cursor, or the plane being drawn on).
pub fn target(
    model: &Model,
    screen: Screen<'_>,
    cursor: Vec2,
    query: &Query<'_>,
    exclude: impl Fn(VertexId) -> bool,
    fallback: impl FnOnce(&Ray) -> Option<(DVec3, Option<Snap>)>,
) -> Target {
    let base = query.base;
    let ray = screen.ray(cursor);
    let last = query.hints.last_snap.filter(|l| still_there(model, l));
    let feature = sticky(screen, cursor, last, snap_feature(model, screen, cursor, exclude));

    if let Some(lock) = query.lock {
        let t = match &feature {
            Some(s) => Some((s.point - base).dot(lock.dir)),
            None => closest_on_line(base, lock.dir, &ray),
        };
        let mut target = Target {
            point: base + lock.dir * t.unwrap_or((query.current - base).dot(lock.dir)),
            direction: Some(lock),
            snap: feature,
            from: None,
        };
        if feature.is_none() {
            target.align(screen, cursor, query, Some(lock.dir));
        }
        return target;
    }
    if let Some(s) = feature {
        return Target { point: s.point, direction: None, snap: Some(s), from: None };
    }
    if let Some((point, direction)) = infer_direction(screen, base, cursor, query.hints, query.plane_normal) {
        let mut target = Target { point, direction: Some(direction), snap: None, from: None };
        target.align(screen, cursor, query, Some(direction.dir));
        return target;
    }
    let (point, snap) = fallback(&ray).unwrap_or((query.current, None));
    let mut target = Target { point, direction: None, snap, from: None };
    target.align(screen, cursor, query, None);
    target
}

impl Target {
    /// Line the point up with the remembered point along an axis, when the cursor is near
    /// that line: on the current direction if there is one, else freely.
    fn align(&mut self, screen: Screen<'_>, cursor: Vec2, query: &Query<'_>, along: Option<DVec3>) {
        let Some(anchor) = query.hints.point else { return };
        if anchor.distance(query.base) < EPS {
            return;
        }
        let ray = screen.ray(cursor);
        let best = Axis::ALL
            .into_iter()
            .filter_map(|axis| {
                let a = axis.dir();
                let point = match along {
                    // Where the current direction crosses the axis line through the anchor.
                    Some(d) => {
                        if d.cross(a).length() < 1e-6 {
                            return None;
                        }
                        line_crossing(query.base, d, anchor, a)?.1
                    }
                    None => anchor + a * closest_on_line(anchor, a, &ray)?,
                };
                // Near the axis line through the anchor on screen.
                let on_line = anchor + a * closest_on_line(anchor, a, &ray)?;
                let distance = screen.project(on_line)?.distance(cursor);
                (distance < AXIS_RADIUS && point.distance(anchor) > EPS).then_some((distance, point, axis))
            })
            .min_by(|x, y| x.0.total_cmp(&y.0));
        if let Some((_, point, axis)) = best {
            self.point = point;
            self.from = Some((anchor, axis));
        }
    }
}

/// Keep `last` (the snap before) over `new` while the cursor is still near it and `new` isn't
/// clearly closer on screen.
fn sticky(screen: Screen<'_>, cursor: Vec2, last: Option<Snap>, new: Option<Snap>) -> Option<Snap> {
    // Only points stick (a point on an edge moves with the cursor).
    let Some(last) = last.filter(|l| is_point(l.kind)) else { return new };
    let Some(was) = screen.project(last.point).map(|p| p.distance(cursor)) else { return new };
    if was > STICKY_RADIUS {
        return new;
    }
    match new {
        Some(n) if n.point.distance(last.point) < EPS => Some(n),
        Some(n) if screen.project(n.point).is_some_and(|p| p.distance(cursor) < was * 0.5) => Some(n),
        _ => Some(last),
    }
}

/// Whether a remembered snap is still in the drawing (it may have been moved or erased since).
fn still_there(model: &Model, snap: &Snap) -> bool {
    match snap.kind {
        SnapKind::Endpoint => model.vertices().any(|(_, p)| p.distance(snap.point) < EPS),
        SnapKind::Midpoint => model.edges().any(|(_, e)| {
            let [a, b] = model.edge_points(e);
            ((a + b) * 0.5).distance(snap.point) < EPS
        }),
        _ => true,
    }
}

fn is_point(kind: SnapKind) -> bool {
    matches!(kind, SnapKind::Endpoint | SnapKind::Midpoint | SnapKind::GuidePoint | SnapKind::Intersection | SnapKind::Origin)
}

/// How much farther than [`AXIS_RADIUS`] the direction followed last still holds.
const STICKY_DIRECTION: f32 = 1.5;

/// How near, in points, the cursor must stay to the last snap for it to stick.
const STICKY_RADIUS: f32 = 10.0;

/// Where the line `p + s d` meets the line through `q` along `e` (both unit), if they cross.
fn line_crossing(p: DVec3, d: DVec3, q: DVec3, e: DVec3) -> Option<(f64, DVec3)> {
    let b = d.dot(e);
    let denom = 1.0 - b * b;
    if denom < 1e-9 {
        return None;
    }
    let w = p - q;
    let s = (b * e.dot(w) - d.dot(w)) / denom;
    let t = (e.dot(w) - b * d.dot(w)) / denom;
    let (x, y) = (p + d * s, q + e * t);
    (x.distance(y) <= 1e-6 * (1.0 + x.length())).then_some((s, x))
}

/// The direction through `base` the cursor is near on screen: an axis, or parallel or
/// perpendicular to the remembered edge. Returns the point on it nearest the cursor.
pub fn infer_direction(
    screen: Screen<'_>,
    base: DVec3,
    cursor: Vec2,
    hints: &Hints,
    plane_normal: DVec3,
) -> Option<(DVec3, Direction)> {
    let mut candidates: Vec<Direction> = Axis::ALL.into_iter().map(Direction::axis).collect();
    let is_axis = |d: DVec3| Axis::ALL.iter().any(|a| d.cross(a.dir()).length() < 1e-6);
    if let Some(edge) = hints.edge {
        // An edge along an axis adds nothing the axes don't already give.
        if !is_axis(edge) {
            candidates.push(Direction { along: Along::Parallel, dir: edge.normalize() });
        }
        let across = edge.cross(plane_normal);
        if across.length() > 1e-6 && !is_axis(across.normalize()) {
            candidates.push(Direction { along: Along::Perpendicular, dir: across.normalize() });
        }
    }
    let ray = screen.ray(cursor);
    let near = |d: Direction, radius: f32| {
        let point = base + d.dir * closest_on_line(base, d.dir, &ray)?;
        let distance = screen.project(point)?.distance(cursor);
        (distance < radius).then_some((distance, point, d))
    };
    let best = candidates.iter().filter_map(|&d| near(d, AXIS_RADIUS)).min_by(|a, b| a.0.total_cmp(&b.0));
    // Sticky: the direction followed last holds (a little farther out, too) unless another is
    // clearly closer, so the point doesn't flip between two directions close on screen.
    let held = hints
        .last_direction
        .and_then(|last| candidates.iter().copied().find(|d| d.along == last.along && d.dir.cross(last.dir).length() < 1e-9))
        .and_then(|d| near(d, AXIS_RADIUS * STICKY_DIRECTION));
    match (held, best) {
        (Some(h), Some(b)) if b.0 < h.0 * 0.5 => Some((b.1, b.2)),
        (Some(h), _) => Some((h.1, h.2)),
        (None, b) => b.map(|(_, point, d)| (point, d)),
    }
}

#[cfg(test)]
mod tests {
    use woodsketch_view::{OrbitCamera, StandardView};

    use super::*;

    const SIZE: Vec2 = Vec2::new(1000.0, 800.0);

    fn top() -> OrbitCamera {
        let mut camera = OrbitCamera { target: glam::Vec3::new(5.0, 5.0, 0.0), distance: 40.0, ..Default::default() };
        camera.set_view(StandardView::Top);
        camera
    }

    fn at(screen: Screen<'_>, x: f64, y: f64) -> Vec2 {
        screen.project(DVec3::new(x, y, 0.0)).unwrap()
    }

    fn query(base: DVec3, hints: &Hints) -> Query<'_> {
        Query { base, hints, lock: None, plane_normal: DVec3::Z, current: base }
    }

    fn ground(ray: &Ray) -> Option<(DVec3, Option<Snap>)> {
        ray.hit_plane(DVec3::ZERO, DVec3::Z).map(|p| (p, None))
    }

    #[test]
    fn parallel_and_perpendicular_to_a_remembered_edge() {
        let model = Model::default();
        let camera = top();
        let screen = Screen::new(&camera, SIZE);
        // An edge at 30 degrees was hovered.
        let edge = DVec3::new(30f64.to_radians().cos(), 30f64.to_radians().sin(), 0.0);
        let hints = Hints { edge: Some(edge), point: None, last_snap: None, last_direction: None };
        let base = DVec3::new(2.0, 2.0, 0.0);
        let near_parallel = base + edge * 6.0 + DVec3::new(0.05, -0.05, 0.0);
        let t = target(&model, screen, at(screen, near_parallel.x, near_parallel.y), &query(base, &hints), |_| false, ground);
        assert_eq!(t.direction.map(|d| d.along), Some(Along::Parallel));
        assert!((t.point - base).cross(edge).length() < 1e-9, "exactly parallel");
        let across = DVec3::new(-edge.y, edge.x, 0.0);
        let near_perpendicular = base + across * 5.0 + DVec3::new(0.05, 0.05, 0.0);
        let t = target(&model, screen, at(screen, near_perpendicular.x, near_perpendicular.y), &query(base, &hints), |_| false, ground);
        assert_eq!(t.direction.map(|d| d.along), Some(Along::Perpendicular));
        assert!((t.point - base).dot(edge).abs() < 1e-9, "exactly perpendicular");
    }

    #[test]
    fn the_last_endpoint_sticks_until_another_is_clearly_closer() {
        let mut model = Model::default();
        model.add_line(DVec3::new(2.0, 2.0, 0.0), DVec3::new(2.3, 2.0, 0.0));
        let camera = top();
        let screen = Screen::new(&camera, SIZE);
        let (a, b) = (DVec3::new(2.0, 2.0, 0.0), DVec3::new(2.3, 2.0, 0.0));
        let mut hints = Hints::default();
        let base = DVec3::ZERO;
        let first = target(&model, screen, at(screen, 2.0, 2.0), &query(base, &hints), |_| false, ground);
        assert!(first.point.abs_diff_eq(a, 1e-9));
        hints.note(first.snap.as_ref());
        // A little past halfway to the other end: still the first.
        let (pa, pb) = (screen.project(a).unwrap(), screen.project(b).unwrap());
        let cursor = pa + (pb - pa) * 0.55;
        assert!(pa.distance(cursor) < STICKY_RADIUS, "test setup: {}", pa.distance(cursor));
        let t = target(&model, screen, cursor, &query(base, &hints), |_| false, ground);
        assert!(t.point.abs_diff_eq(a, 1e-9), "stuck: {}", t.point);
        // Right on the other end: it wins.
        let t = target(&model, screen, pb, &query(base, &hints), |_| false, ground);
        assert!(t.point.abs_diff_eq(b, 1e-9));
    }

    #[test]
    fn the_direction_followed_last_holds_a_little_farther_out() {
        let model = Model::default();
        let camera = top();
        let screen = Screen::new(&camera, SIZE);
        let base = DVec3::new(2.0, 2.0, 0.0);
        let per_inch = screen.project(base + DVec3::Y).unwrap().distance(screen.project(base).unwrap());
        // 1.2 times the usual reach off the red line through the base.
        let off = f64::from(AXIS_RADIUS * 1.2 / per_inch);
        let cursor = at(screen, base.x + 6.0, base.y + off);
        let fresh = Hints::default();
        assert!(target(&model, screen, cursor, &query(base, &fresh), |_| false, ground).direction.is_none(), "too far for a new one");
        let held = Hints { last_direction: Some(Direction::axis(Axis::X)), ..Hints::default() };
        let t = target(&model, screen, cursor, &query(base, &held), |_| false, ground);
        assert_eq!(t.direction.map(|d| d.along), Some(Along::Axis(Axis::X)), "still on red");
        assert!((t.point.y - base.y).abs() < 1e-9);
    }

    #[test]
    fn lines_up_with_a_remembered_point() {
        let model = Model::default();
        let camera = top();
        let screen = Screen::new(&camera, SIZE);
        // A corner at (8, 6) was hovered; draw along red from the origin.
        let hints = Hints { edge: None, point: Some(DVec3::new(8.0, 6.0, 0.0)), last_snap: None, last_direction: None };
        let base = DVec3::ZERO;
        // Moving along red, the cursor passes under the corner: the point stops at x = 8.
        let t = target(&model, screen, at(screen, 8.1, 0.05), &query(base, &hints), |_| false, ground);
        assert_eq!(t.direction.map(|d| d.along), Some(Along::Axis(Axis::X)));
        assert!(t.point.abs_diff_eq(DVec3::new(8.0, 0.0, 0.0), 1e-9), "{}", t.point);
        assert_eq!(t.from, Some((DVec3::new(8.0, 6.0, 0.0), Axis::Y)));
        // Free on the ground, level with the corner along red.
        let t = target(&model, screen, at(screen, 3.0, 6.1), &query(DVec3::new(-5.0, -5.0, 0.0), &hints), |_| false, ground);
        assert!((t.point.y - 6.0).abs() < 1e-9, "{}", t.point);
        assert_eq!(t.from.map(|f| f.1), Some(Axis::X));
    }

    #[test]
    fn a_lock_wins_over_everything() {
        let model = Model::default();
        let camera = top();
        let screen = Screen::new(&camera, SIZE);
        let hints = Hints::default();
        let base = DVec3::ZERO;
        let lock = Some(Direction::axis(Axis::Y));
        let q = Query { lock, ..query(base, &hints) };
        let t = target(&model, screen, at(screen, 7.0, 3.0), &q, |_| false, ground);
        // Exactly on the green axis; how far along depends on the perspective.
        assert!(t.point.x.abs() < 1e-9 && t.point.z.abs() < 1e-9 && (t.point.y - 3.0).abs() < 0.05, "{}", t.point);
    }
}
