//! Joints between parts that meet at an angle (a knee brace into a post, a rafter into a ridge):
//! parts turned to each other about one axis they share, like their thickness.
//!
//! Each part is then a flat shape (its profile across that axis) stretched through its
//! thickness, so the joint is worked out in 2D, layer by layer: the plane is cut into convex
//! cells by every edge line of both profiles (and the tenon's shoulder lines), the shared axis
//! into slabs at every face's height, and each part is rebuilt from the cells and slabs it fills.
//!
//! - **Mortise and tenon**: the overlap of the tenoned part inside the other is trimmed back to
//!   the other's face (the shoulders, at the angle), except a tenon (a slice through the middle of
//!   its thickness, set in from its edges) that goes on into the other, which gets a mortise of
//!   exactly the tenon's shape.
//! - **Housing**: the whole overlap is cut out of the other part; the first keeps its shape.
//!
//! The overlap, as drawn, sets the joint's depth. Each joint is one undo step, recorded in
//! [`crate::Scene::joints`] with the parts' shapes before it, so it can be taken out again.

use std::collections::HashMap;

use glam::{DAffine3, DVec2, DVec3};

use crate::joinery::{JoineryError, Joint, JointKind};
use crate::units::format_length;
use crate::{DefinitionId, Document, Entity, InstanceId, Model};

/// Closer than this counts as the same (1/1000").
const TOL: f64 = 1e-3;

/// What to make where two parts meet at an angle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AngledJoint {
    /// A tenon `thickness` thick through the middle, set in `shoulder` from each edge.
    MortiseAndTenon { thickness: f64, shoulder: f64 },
    /// The whole overlap let into the other part.
    Housing,
}

/// How two parts meet at an angle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AngledFit {
    /// Between their lengths, in degrees (0-90).
    pub angle: f64,
    /// Each part's thickness along the shared axis.
    pub thickness: (f64, f64),
    /// Whether they overlap (a joint needs one part's end drawn into the other).
    pub overlap: bool,
}

/// One part seen across the shared axis: its surface (for inside tests, in the container's
/// coordinates), its walls' footprints, the heights of its faces square to the axis, and the range
/// it spans along it.
struct Flat {
    triangles: Vec<[DVec3; 3]>,
    walls: Vec<(DVec2, DVec2)>,
    levels: Vec<f64>,
    y: (f64, f64),
    /// The part's own length direction, in 2D.
    along: DVec2,
    /// Its walls' ends, for its extent.
    corners: Vec<DVec2>,
    /// Its faces with their own paint, as triangles with their side (the rebuilt faces on them
    /// take it back).
    painted: Vec<([DVec3; 3], DVec3, crate::MaterialId)>,
}

/// The frame the joint is worked out in: 2D across the shared axis `a`, `y` along it.
struct Frame {
    u: DVec3,
    a: DVec3,
    w: DVec3,
}

impl Frame {
    fn flat(&self, p: DVec3) -> (DVec2, f64) {
        (DVec2::new(self.u.dot(p), self.w.dot(p)), self.a.dot(p))
    }

    fn point(&self, p: DVec2, y: f64) -> DVec3 {
        self.u * p.x + self.a * y + self.w * p.y
    }
}

impl Document {
    /// How `a` and `b` (instances in the container being edited) meet at an angle.
    pub fn angled_fit(&self, a: InstanceId, b: InstanceId) -> Result<AngledFit, JoineryError> {
        let (frame, fa, fb) = self.flatten(a, b)?;
        let cos = fa.along.dot(fb.along).abs().clamp(0.0, 1.0);
        let (cells, levels) = layout(&fa, &fb, &[], &[]);
        let (in_a, in_b) = (fa.occupancy(&frame, &cells, &levels), fb.occupancy(&frame, &cells, &levels));
        let overlap = in_a.iter().flatten().zip(in_b.iter().flatten()).any(|(a, b)| *a && *b);
        Ok(AngledFit { angle: cos.acos().to_degrees(), thickness: (fa.y.1 - fa.y.0, fb.y.1 - fb.y.0), overlap })
    }

    /// Make `joint` between `part` (tenoned, or let in) and `into`, which meet at an angle and
    /// overlap. One undo step.
    pub fn angled_joint(&mut self, part: InstanceId, into: InstanceId, joint: AngledJoint) -> Result<(), JoineryError> {
        let (frame, p, q) = self.flatten(part, into)?;
        // The tenon: a slice through the middle of the part's thickness, set in from its edges.
        let across = DVec2::new(-p.along.y, p.along.x);
        let (lo, hi) = extent(&p.corners, across);
        let (strip, ty) = match joint {
            AngledJoint::MortiseAndTenon { thickness, shoulder } => {
                let mid = (p.y.0 + p.y.1) / 2.0;
                let ty = (mid - thickness / 2.0, mid + thickness / 2.0);
                if thickness <= TOL || ty.0 < p.y.0 - TOL || ty.1 > p.y.1 + TOL || ty.0 < q.y.0 - TOL || ty.1 > q.y.1 + TOL {
                    return Err(JoineryError::Size("The tenon must be thinner than both parts."));
                }
                if hi - lo <= 2.0 * shoulder + TOL {
                    return Err(JoineryError::Size("The shoulders are wider than the part."));
                }
                (Some((lo + shoulder, hi - shoulder)), Some(ty))
            }
            AngledJoint::Housing => (None, None),
        };
        // The shoulder lines cut the plane too, and the tenon's faces the slabs.
        let mut extra = Vec::new();
        if let Some((s0, s1)) = strip {
            for s in [s0, s1] {
                let point = across * s;
                extra.push((point, point + p.along));
            }
        }
        let extra_levels: Vec<f64> = ty.map(|t| vec![t.0, t.1]).unwrap_or_default();
        let (cells, levels) = layout(&p, &q, &extra, &extra_levels);
        let (in_p, in_q) = (p.occupancy(&frame, &cells, &levels), q.occupancy(&frame, &cells, &levels));
        if !in_p.iter().flatten().zip(in_q.iter().flatten()).any(|(a, b)| *a && *b) {
            return Err(JoineryError::Size("Draw the end of one part into the other where the joint goes: the overlap is the joint."));
        }
        let center = |c: &[DVec2]| c.iter().copied().sum::<DVec2>() / c.len() as f64;
        let in_strip = |c: DVec2| strip.is_some_and(|(s0, s1)| (s0..=s1).contains(&c.dot(across)));
        let in_tenon = |i: usize, k: usize| {
            let y = (levels[k] + levels[k + 1]) / 2.0;
            in_strip(center(&cells[i])) && ty.is_some_and(|t| y > t.0 && y < t.1)
        };
        let slabs = levels.len() - 1;
        let mut part_fills = vec![vec![false; slabs]; cells.len()];
        let mut into_fills = vec![vec![false; slabs]; cells.len()];
        let mut tenon_cells = Vec::new();
        for i in 0..cells.len() {
            for k in 0..slabs {
                let both = in_p[i][k] && in_q[i][k];
                let tenon = both && in_tenon(i, k);
                // The part keeps its tenon of the overlap (housed: all of it); the other loses it.
                part_fills[i][k] = in_p[i][k] && (!both || tenon || ty.is_none());
                into_fills[i][k] = in_q[i][k] && !(both && (tenon || ty.is_none()));
                if tenon {
                    tenon_cells.push(i);
                }
            }
        }
        let detail = match ty {
            Some(t) => {
                if tenon_cells.is_empty() {
                    return Err(JoineryError::Size("There's no room for a tenon in the overlap."));
                }
                let corners: Vec<DVec2> = tenon_cells.iter().flat_map(|&i| cells[i].iter().copied()).collect();
                let depth = extent(&corners, p.along);
                format!("{} tenon, {} long, at an angle", format_length(t.1 - t.0), format_length(depth.1 - depth.0))
            }
            None => "housed, at an angle".to_owned(),
        };

        let new_part = solid(&frame, &cells, &levels, &part_fills, &p.painted).ok_or(JoineryError::Size("That joint couldn't be made here."))?;
        let new_into = solid(&frame, &cells, &levels, &into_fills, &q.painted).ok_or(JoineryError::Size("That joint couldn't be made here."))?;

        let container = self.container();
        let (ip, iq) = (container.instance(part).ok_or(JoineryError::NotParts)?, container.instance(into).ok_or(JoineryError::NotParts)?);
        let (dp, dq) = (ip.definition, iq.definition);
        let to_p = ip.transform.inverse();
        let to_q = iq.transform.inverse();
        let model_p = transformed(&new_part, &to_p);
        let model_q = transformed(&new_into, &to_q);
        let scene = self.scene();
        let before: Vec<(DefinitionId, Model)> = [dp, dq]
            .iter()
            .filter_map(|&d| scene.definition(d).map(|def| (d, def.container.model.clone())))
            .collect();
        let kind = match joint {
            AngledJoint::MortiseAndTenon { .. } => JointKind::MortiseAndTenon,
            AngledJoint::Housing => JointKind::Notch,
        };
        self.edit_scene(|scene, path| {
            // A copy of a shared component gets its own: the others (perhaps mirrored, like the
            // other side of an arch) don't take this joint.
            let dp = scene.make_unique(path, part).unwrap_or(dp);
            let dq = scene.make_unique(path, into).unwrap_or(dq);
            let before: Vec<(DefinitionId, Model)> = before.into_iter().zip([dp, dq]).map(|((_, model), new)| (new, model)).collect();
            if let Some(d) = scene.definition_mut(dp) {
                d.container.model = model_p;
            }
            if let Some(d) = scene.definition_mut(dq) {
                d.container.model = model_q;
            }
            scene.joints.push(Joint { kind, cut: dp, into: dq, detail, hardware: None, ops: Vec::new(), before, sizes: Vec::new() });
        });
        Ok(())
    }

    /// Both parts flattened across the axis they share; an error unless they share one and are
    /// plain boards along it (every face along that axis or square to it, one profile).
    fn flatten(&self, a: InstanceId, b: InstanceId) -> Result<(Frame, Flat, Flat), JoineryError> {
        let container = self.container();
        let (ia, ib) = (container.instance(a).ok_or(JoineryError::NotParts)?, container.instance(b).ok_or(JoineryError::NotParts)?);
        let scene = self.scene();
        let (da, db) = (scene.definition(ia.definition).ok_or(JoineryError::NotParts)?, scene.definition(ib.definition).ok_or(JoineryError::NotParts)?);
        let axes = |t: &DAffine3| [t.matrix3.x_axis, t.matrix3.y_axis, t.matrix3.z_axis].map(DVec3::normalize);
        let (aa, ab) = (axes(&ia.transform), axes(&ib.transform));
        let mut last = JoineryError::Size("These parts don't share an axis (a compound angle), so they can't be joined here yet.");
        for x in aa {
            if !ab.iter().any(|y| x.dot(*y).abs() > 1.0 - 1e-6) {
                continue;
            }
            let u = if x.x.abs() < 0.9 { DVec3::X } else { DVec3::Y };
            let u = (u - x * x.dot(u)).normalize();
            let frame = Frame { u, a: x, w: x.cross(u) };
            match (flat_part(&frame, &da.container.model, &ia.transform), flat_part(&frame, &db.container.model, &ib.transform)) {
                (Ok(fa), Ok(fb)) => return Ok((frame, fa, fb)),
                (Err(e), _) | (_, Err(e)) => last = e,
            }
        }
        Err(last)
    }
}

/// A part seen across the frame's axis, if every face runs along the axis (a wall) or square to it
/// (a cap): its walls' footprints, its caps' heights, and its surface for inside tests.
fn flat_part(frame: &Frame, model: &Model, transform: &DAffine3) -> Result<Flat, JoineryError> {
    let not_square = JoineryError::Size("Angled joints need parts whose faces all run along the axis they share, or square to it.");
    let mut flat = Flat {
        triangles: Vec::new(),
        walls: Vec::new(),
        levels: Vec::new(),
        y: (f64::INFINITY, f64::NEG_INFINITY),
        along: DVec2::X,
        corners: Vec::new(),
        painted: Vec::new(),
    };
    for (_, face) in model.faces() {
        let n = transform.transform_vector3(face.normal).normalize();
        let points: Vec<DVec3> = face.outer.iter().map(|&v| transform.transform_point3(model.vertex(v))).collect();
        let along = n.dot(frame.a);
        if along.abs() < 1e-6 {
            // A wall: its footprint, end to end along its width.
            let d = frame.a.cross(n);
            let (lo, hi) = points.iter().map(|p| p.dot(d)).fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| (lo.min(x), hi.max(x)));
            let (a, b) = (points.iter().find(|p| (p.dot(d) - lo).abs() < 1e-9), points.iter().find(|p| (p.dot(d) - hi).abs() < 1e-9));
            if let (Some(&a), Some(&b)) = (a, b) {
                let (a, b) = (frame.flat(a).0, frame.flat(b).0);
                if a.distance(b) > TOL {
                    flat.walls.push((a, b));
                    flat.corners.extend([a, b]);
                }
            }
        } else if along.abs() > 1.0 - 1e-6 {
            flat.levels.push(frame.flat(points[0]).1);
        } else {
            return Err(not_square);
        }
        for p in &points {
            let y = frame.flat(*p).1;
            flat.y = (flat.y.0.min(y), flat.y.1.max(y));
        }
        for [a, b, c] in model.triangulate(face) {
            let triangle = [a, b, c].map(|p| transform.transform_point3(p));
            flat.triangles.push(triangle);
            if let Some(material) = face.material {
                flat.painted.push((triangle, n, material));
            }
        }
    }
    if flat.walls.is_empty() || flat.y.1 - flat.y.0 <= TOL {
        return Err(not_square);
    }
    flat.levels.extend([flat.y.0, flat.y.1]);
    // Its length runs along its longest wall.
    if let Some(&(a, b)) = flat.walls.iter().max_by(|x, y| x.0.distance(x.1).total_cmp(&y.0.distance(y.1))) {
        flat.along = (b - a).normalize();
    }
    Ok(flat)
}

impl Flat {
    /// Which cells (at each slab's middle height) it fills.
    fn occupancy(&self, frame: &Frame, cells: &[Vec<DVec2>], levels: &[f64]) -> Vec<Vec<bool>> {
        cells
            .iter()
            .map(|cell| {
                let c = cell.iter().copied().sum::<DVec2>() / cell.len() as f64;
                (0..levels.len() - 1).map(|k| inside(&self.triangles, frame.point(c, (levels[k] + levels[k + 1]) / 2.0))).collect()
            })
            .collect()
    }
}

/// Whether `p` is inside the closed surface made of `triangles`: an odd number of them crossed by
/// a ray from it (in a direction no drawn edge is likely to line up with).
fn inside(triangles: &[[DVec3; 3]], p: DVec3) -> bool {
    let dir = DVec3::new(0.5377, 0.6219, 0.5693).normalize();
    let mut crossings = 0;
    for [a, b, c] in triangles {
        let (e1, e2) = (*b - *a, *c - *a);
        let h = dir.cross(e2);
        let det = e1.dot(h);
        if det.abs() < 1e-12 {
            continue;
        }
        let s = p - *a;
        let u = s.dot(h) / det;
        if !(0.0..=1.0).contains(&u) {
            continue;
        }
        let q = s.cross(e1);
        let v = dir.dot(q) / det;
        if v < 0.0 || u + v > 1.0 {
            continue;
        }
        if e2.dot(q) / det > 1e-9 {
            crossings += 1;
        }
    }
    crossings % 2 == 1
}

/// The convex cells and the slab heights for joining `p` and `q`: the plane cut by every wall's
/// line (and `extra` lines), the shared axis at every cap (and `extra_levels`).
fn layout(p: &Flat, q: &Flat, extra: &[(DVec2, DVec2)], extra_levels: &[f64]) -> (Vec<Vec<DVec2>>, Vec<f64>) {
    let lines: Vec<(DVec2, DVec2)> = p.walls.iter().chain(&q.walls).chain(extra).copied().collect();
    let cells = arrangement(&[&p.corners[..], &q.corners[..]], &lines);
    let mut levels: Vec<f64> = p.levels.iter().chain(&q.levels).chain(extra_levels).copied().collect();
    levels.sort_by(f64::total_cmp);
    levels.dedup_by(|a, b| (*a - *b).abs() < TOL);
    (cells, levels)
}

fn signed_area(poly: &[DVec2]) -> f64 {
    (0..poly.len()).map(|i| poly[i].perp_dot(poly[(i + 1) % poly.len()])).sum::<f64>() / 2.0
}

fn area(poly: &[DVec2]) -> f64 {
    signed_area(poly).abs()
}

/// The smallest and largest of `poly`'s points along `dir`.
fn extent(poly: &[DVec2], dir: DVec2) -> (f64, f64) {
    poly.iter().map(|p| p.dot(dir)).fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), d| (lo.min(d), hi.max(d)))
}

/// The part of convex `poly` where `p · normal >= offset`.
fn halfplane(poly: &[DVec2], normal: DVec2, offset: f64) -> Vec<DVec2> {
    let mut out = Vec::new();
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let (da, db) = (a.dot(normal) - offset, b.dot(normal) - offset);
        if da >= 0.0 {
            out.push(a);
        }
        if (da >= 0.0) != (db >= 0.0) {
            out.push(a + (b - a) * (da / (da - db)));
        }
    }
    out
}

/// The convex cells the bounding box of `shapes` is cut into by every line (each given by two of
/// its points).
fn arrangement(shapes: &[&[DVec2]], lines: &[(DVec2, DVec2)]) -> Vec<Vec<DVec2>> {
    let (mut lo, mut hi) = (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY));
    for p in shapes.iter().flat_map(|s| s.iter()) {
        lo = lo.min(*p);
        hi = hi.max(*p);
    }
    let (lo, hi) = (lo - 1.0, hi + 1.0);
    let mut cells = vec![vec![lo, DVec2::new(hi.x, lo.y), hi, DVec2::new(lo.x, hi.y)]];
    let mut seen: Vec<(DVec2, f64)> = Vec::new();
    for &(a, b) in lines {
        if a.distance(b) < TOL {
            continue;
        }
        let dir = (b - a).normalize();
        let normal = DVec2::new(-dir.y, dir.x);
        let offset = normal.dot(a);
        // The same line twice (or reversed) cuts nothing new.
        if seen.iter().any(|&(n, o)| (n.dot(normal).abs() > 1.0 - 1e-9) && (o - offset * n.dot(normal).signum()).abs() < TOL) {
            continue;
        }
        seen.push((normal, offset));
        cells = cells
            .into_iter()
            .flat_map(|cell| {
                let (left, right) = (halfplane(&cell, normal, offset), halfplane(&cell, -normal, -offset));
                // A line through a corner leaves it doubled: drop the repeat.
                [left, right].into_iter().map(|mut c| {
                    c.dedup_by(|a, b| a.distance(*b) < TOL);
                    if c.len() > 1 && c[0].distance(c[c.len() - 1]) < TOL {
                        c.pop();
                    }
                    c
                })
                .filter(|c| c.len() >= 3 && area(c) > TOL * TOL)
            })
            .collect();
    }
    cells
}

/// A 2D point rounded to [`TOL`], to match cell corners.
type Key = (i64, i64);

/// The closed solid filling the `occupied` cells of each slab (`occupied[cell][slab]`), in the
/// container's coordinates. `None` if it comes out empty or open.
fn solid(frame: &Frame, cells: &[Vec<DVec2>], levels: &[f64], occupied: &[Vec<bool>], painted: &[([DVec3; 3], DVec3, crate::MaterialId)]) -> Option<Model> {
    let slabs = levels.len() - 1;
    // Which cell is across each edge (keyed by its ends, rounded).
    let key = |p: DVec2| -> Key { ((p.x / TOL).round() as i64, (p.y / TOL).round() as i64) };
    let mut edges: HashMap<(Key, Key), usize> = HashMap::new();
    for (i, cell) in cells.iter().enumerate() {
        for j in 0..cell.len() {
            edges.insert((key(cell[j]), key(cell[(j + 1) % cell.len()])), i);
        }
    }
    // Built face by face from shared corners (no merging or hole-finding: the cells already fit).
    let mut model = Model::default();
    let mut corners: HashMap<(i64, i64, i64), crate::VertexId> = HashMap::new();
    let mut sides: HashMap<(crate::VertexId, crate::VertexId), ()> = HashMap::new();
    let mut add = |model: &mut Model, points: &[DVec3]| -> Option<()> {
        let ids: Vec<crate::VertexId> = points
            .iter()
            .map(|&p| {
                let k = ((p.x / TOL).round() as i64, (p.y / TOL).round() as i64, (p.z / TOL).round() as i64);
                *corners.entry(k).or_insert_with(|| model.push_vertex(p))
            })
            .collect();
        for i in 0..ids.len() {
            let (a, b) = (ids[i], ids[(i + 1) % ids.len()]);
            if a != b && sides.insert((a.min(b), a.max(b)), ()).is_none() {
                model.push_edge(a, b);
            }
        }
        let normal = (0..points.len()).fold(DVec3::ZERO, |n, i| n + points[i].cross(points[(i + 1) % points.len()]));
        (normal.length() > 1e-12).then(|| {
            model.push_face(ids, Vec::new(), normal.normalize(), None);
        })
    };
    let mut faces = 0;
    for (i, cell) in cells.iter().enumerate() {
        for k in 0..slabs {
            if !occupied[i][k] {
                continue;
            }
            let (y0, y1) = (levels[k], levels[k + 1]);
            // Walls where the cell across is empty (or there's none).
            for j in 0..cell.len() {
                let (a, b) = (cell[j], cell[(j + 1) % cell.len()]);
                let across = edges.get(&(key(b), key(a))).copied();
                if across.is_some_and(|n| occupied[n][k]) {
                    continue;
                }
                // Counter-clockwise cells: the outward side of a -> b is to the right.
                let quad = [frame.point(a, y0), frame.point(b, y0), frame.point(b, y1), frame.point(a, y1)];
                add(&mut model, &outward(quad, frame, a, b))?;
                faces += 1;
            }
            // Caps where the slab below or above is empty.
            if k == 0 || !occupied[i][k - 1] {
                let down: Vec<DVec3> = cell.iter().rev().map(|&p| frame.point(p, y0)).collect();
                add(&mut model, &down)?;
                faces += 1;
            }
            if k + 1 == slabs || !occupied[i][k + 1] {
                let up: Vec<DVec3> = cell.iter().map(|&p| frame.point(p, y1)).collect();
                add(&mut model, &up)?;
                faces += 1;
            }
        }
    }
    if faces == 0 {
        return None;
    }
    // Merge faces lying flat against each other (the seams between cells and slabs).
    loop {
        let seam = model.edges().map(|(id, _)| id).find(|&id| {
            let f = model.edge_faces(id);
            f.len() == 2 && {
                let (n, m) = (model.face(f[0]).map(|f| f.normal), model.face(f[1]).map(|f| f.normal));
                n.zip(m).is_some_and(|(n, m)| n.dot(m) > 1.0 - 1e-9)
            }
        });
        let Some(seam) = seam else { break };
        model.erase([Entity::Edge(seam)]);
    }
    // Lines left on no face by the merging.
    let stray: Vec<Entity> = model.edges().map(|(id, _)| id).filter(|&id| model.edge_faces(id).is_empty()).map(Entity::Edge).collect();
    model.erase(stray);
    // Faces on painted faces of the part keep their paint.
    let faces: Vec<(crate::FaceId, DVec3, DVec3)> = model
        .faces()
        .filter_map(|(id, f)| model.triangulate(f).first().map(|[a, b, c]| (id, (*a + *b + *c) / 3.0, f.normal)))
        .collect();
    for (id, at, normal) in faces {
        let paint = painted.iter().find(|([a, b, c], n, _)| n.dot(normal) > 1.0 - 1e-6 && on_triangle(at, [*a, *b, *c], *n)).map(|(_, _, m)| *m);
        if paint.is_some() {
            model.set_face_material(id, paint);
        }
    }
    let closed = model.edges().all(|(id, _)| model.edge_faces(id).len() == 2);
    closed.then_some(model)
}

/// Whether `p` lies on triangle `[a, b, c]` (in its plane, inside its edges), whose side is `n`.
fn on_triangle(p: DVec3, [a, b, c]: [DVec3; 3], n: DVec3) -> bool {
    // Triangles from a face can wind either way: inside is the same side of all three edges.
    let sides = [(a, b), (b, c), (c, a)].map(|(u, v)| (v - u).cross(p - u).dot(n));
    (p - a).dot(n).abs() < TOL && (sides.iter().all(|s| *s >= -1e-9) || sides.iter().all(|s| *s <= 1e-9))
}

/// A wall quad over edge `a -> b` of a counter-clockwise cell, wound to face away from the cell.
fn outward(quad: [DVec3; 4], frame: &Frame, a: DVec2, b: DVec2) -> [DVec3; 4] {
    // Outward in 2D is to the right of a -> b.
    let right = DVec2::new((b - a).y, -(b - a).x);
    let want = frame.u * right.x + frame.w * right.y;
    let n = (quad[1] - quad[0]).cross(quad[3] - quad[0]);
    if n.dot(want) >= 0.0 { quad } else { [quad[0], quad[3], quad[2], quad[1]] }
}

/// `model` with every point moved by `transform` (into a part's own coordinates).
fn transformed(model: &Model, transform: &DAffine3) -> Model {
    let mut out = Model::default();
    out.append(model, transform);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    fn board(doc: &mut Document, origin: DVec3, length_dir: DVec3, width_dir: DVec3, [l, w, t]: [f64; 3]) -> InstanceId {
        doc.add_board(&Board { origin, length_dir, width_dir, length: l, width: w, thickness: t }, None).unwrap()
    }

    fn closed(doc: &Document, part: InstanceId) -> bool {
        let def = doc.container().instance(part).unwrap().definition;
        let m = &doc.scene().definition(def).unwrap().container.model;
        m.faces().count() > 0 && m.edges().all(|(id, _)| m.edge_faces(id).len() == 2)
    }

    fn volume(doc: &Document, part: InstanceId) -> f64 {
        let def = doc.container().instance(part).unwrap().definition;
        let m = &doc.scene().definition(def).unwrap().container.model;
        // Divergence theorem over the triangulated faces, each turned to its face's side.
        m.faces()
            .flat_map(|(_, f)| m.triangulate(f).into_iter().map(move |t| (t, f.normal)))
            .map(|([a, b, c], n)| {
                let v = a.dot(b.cross(c)) / 6.0;
                if (b - a).cross(c - a).dot(n) >= 0.0 { v } else { -v }
            })
            .sum::<f64>()
    }

    /// A 3 1/2" x 1 1/2" post, and a brace at 45° across the thickness axis (Y) with its end
    /// drawn 1" into the post's side.
    fn post_and_brace(doc: &mut Document) -> (InstanceId, InstanceId) {
        let post = board(doc, DVec3::ZERO, DVec3::Z, DVec3::X, [48.0, 3.5, 1.5]);
        let dir = DVec3::new(1.0, 0.0, 1.0).normalize();
        // Its thickness runs along -Y (length x width), so it starts at y = 1 1/2".
        let brace = board(doc, DVec3::new(2.5, 1.5, 20.0), dir, DVec3::new(-1.0, 0.0, 1.0).normalize(), [20.0, 3.5, 1.5]);
        (post, brace)
    }

    #[test]
    fn parts_at_45_degrees_are_read_as_angled() {
        let mut doc = Document::default();
        let (post, brace) = post_and_brace(&mut doc);
        assert!(doc.joint_fit(brace, post).is_err(), "not square: the usual joints can't");
        let fit = doc.angled_fit(brace, post).unwrap();
        assert!((fit.angle - 45.0).abs() < 1e-6, "{fit:?}");
        assert!(fit.overlap);
    }

    #[test]
    fn an_angled_mortise_and_tenon_fits_exactly() {
        let mut doc = Document::default();
        let (post, brace) = post_and_brace(&mut doc);
        let (post_before, brace_before) = (volume(&doc, post), volume(&doc, brace));
        doc.angled_joint(brace, post, AngledJoint::MortiseAndTenon { thickness: 0.5, shoulder: 0.25 }).unwrap();
        assert!(closed(&doc, post) && closed(&doc, brace), "both closed solids");
        let (post_after, brace_after) = (volume(&doc, post), volume(&doc, brace));
        let mortise = post_before - post_after;
        assert!(mortise > 0.1, "a mortise was cut: {mortise}");
        // The brace lost its overlap but kept a tenon exactly the mortise's size.
        let trimmed = brace_before - brace_after;
        assert!(trimmed > 0.0);
        let joints = &doc.scene().joints;
        assert_eq!(joints.len(), 1);
        assert!(joints[0].detail.contains("tenon"), "{}", joints[0].detail);
        // Taken out again: both back as drawn.
        doc.remove_joint(0).unwrap();
        assert!((volume(&doc, post) - post_before).abs() < 1e-9);
        assert!((volume(&doc, brace) - brace_before).abs() < 1e-9);
    }

    #[test]
    fn painted_faces_keep_their_paint() {
        let mut doc = Document::default();
        let (post, brace) = post_and_brace(&mut doc);
        let red = doc.add_material(crate::MaterialKind::SolidWood);
        // Paint the brace's cheeks (its faces square to the shared axis).
        let def = doc.container().instance(brace).unwrap().definition;
        let cheeks: Vec<crate::FaceId> = {
            let m = &doc.scene().definition(def).unwrap().container.model;
            let t = doc.container().instance(brace).unwrap().transform;
            m.faces().filter(|(_, f)| t.transform_vector3(f.normal).y.abs() > 0.99).map(|(id, _)| id).collect()
        };
        doc.edit_scene(|scene, _| {
            let m = &mut scene.definition_mut(def).unwrap().container.model;
            for &f in &cheeks {
                m.set_face_material(f, Some(red));
            }
        });
        doc.angled_joint(brace, post, AngledJoint::MortiseAndTenon { thickness: 0.5, shoulder: 0.25 }).unwrap();
        let def = doc.container().instance(brace).unwrap().definition;
        let m = &doc.scene().definition(def).unwrap().container.model;
        let painted = m.faces().filter(|(_, f)| f.material == Some(red)).count();
        assert!(painted >= 2, "both cheeks keep their paint ({painted} painted)");
    }

    #[test]
    fn an_angled_housing_lets_the_whole_end_in() {
        let mut doc = Document::default();
        let (post, brace) = post_and_brace(&mut doc);
        let brace_before = volume(&doc, brace);
        doc.angled_joint(brace, post, AngledJoint::Housing).unwrap();
        assert!(closed(&doc, post) && closed(&doc, brace));
        assert!((volume(&doc, brace) - brace_before).abs() < 1e-6, "the brace keeps its shape");
    }

    #[test]
    fn a_copy_of_a_shared_post_gets_its_own_mortise() {
        let mut doc = Document::default();
        let (post, brace) = post_and_brace(&mut doc);
        doc.selection_mut().extend([Entity::Instance(post)]);
        let post = doc.group_selection(crate::DefinitionKind::Component).unwrap();
        let other = doc.transform_entities(&[Entity::Instance(post)], &[DAffine3::from_translation(DVec3::X * 40.0)], true).unwrap();
        let Entity::Instance(other) = other[0] else { panic!() };
        let untouched = volume(&doc, other);
        doc.angled_joint(brace, post, AngledJoint::MortiseAndTenon { thickness: 0.5, shoulder: 0.25 }).unwrap();
        assert!(volume(&doc, post) < untouched, "this post has its mortise");
        assert!((volume(&doc, other) - untouched).abs() < 1e-9, "the other copy doesn't");
    }

    #[test]
    fn parts_that_dont_overlap_say_so() {
        let mut doc = Document::default();
        let post = board(&mut doc, DVec3::ZERO, DVec3::Z, DVec3::X, [48.0, 3.5, 1.5]);
        let dir = DVec3::new(1.0, 0.0, 1.0).normalize();
        let brace = board(&mut doc, DVec3::new(10.0, 1.5, 20.0), dir, DVec3::new(-1.0, 0.0, 1.0).normalize(), [20.0, 3.5, 1.5]);
        let err = doc.angled_joint(brace, post, AngledJoint::Housing).unwrap_err();
        assert!(err.to_string().contains("overlap"), "{err}");
    }
}
#[cfg(test)]
mod real_file {
    use super::*;
    /// `WOODSKETCH_SKP=<arch.skp> cargo test -p woodsketch-doc angled::real_file -- --nocapture`
    #[test]
    fn braces_into_posts_in_a_real_file() {
        let Ok(path) = std::env::var("WOODSKETCH_SKP") else { return };
        let scene = crate::skp::read(&std::fs::read(&path).unwrap()).unwrap().scene;
        let mut doc = Document::from_scene(scene);
        let named = |doc: &Document, name: &str| -> Vec<InstanceId> {
            doc.container().instances().filter(|(_, i)| doc.scene().definition(i.definition).is_some_and(|d| d.name == name)).map(|(id, _)| id).collect()
        };
        for brace_name in ["Angle Bracket Top", "Angle Bracket Bottom"] {
            for brace in named(&doc, brace_name) {
                for post in named(&doc, "Post") {
                    match doc.angled_fit(brace, post) {
                        Ok(fit) if fit.overlap => {
                            let r = doc.angled_joint(brace, post, AngledJoint::MortiseAndTenon { thickness: 0.5, shoulder: 0.25 });
                            println!("{brace_name} {brace:?} -> post {post:?}: {:.0}° {r:?}", fit.angle);
                        }
                        Ok(fit) => println!("{brace_name} {brace:?} -> post {post:?}: {:.0}°, no overlap", fit.angle),
                        Err(e) => println!("{brace_name} {brace:?} -> post {post:?}: {e}"),
                    }
                }
            }
        }
        for j in &doc.scene().joints {
            println!("joint: {}", j.detail);
        }
    }
}
