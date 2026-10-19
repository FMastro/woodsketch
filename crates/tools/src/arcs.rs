//! SketchUp's other arc tools. Center Arc: click the center, click the start (or type the
//! radius), then turn and click (or type the angle). Pie: the same, closed back to the center
//! with two straight sides, so it makes a face. 3-Point Arc: click the start, a point the arc
//! passes through, then the end. `24s` sets the number of sides (12 by default).

use std::f64::consts::TAU;

use glam::{DVec3, Vec2};
use woodsketch_doc::curves::{ARC_SEGMENTS, angle_about, circle_through, sweep_points};
use woodsketch_doc::units::{format_length, parse_length};
use woodsketch_doc::{Document, EPS};

use crate::circle::parse_segments;
use crate::pick::{Screen, Snap, snap};
use crate::{Axis, Overlay};

/// Within this many degrees of a full turn, the sweep closes into a full circle.
const CLOSE_DEGREES: f64 = 6.0;

/// The sweep after the cursor turns to `turned` (0..TAU from the start): it keeps going the way it
/// was (past half a turn too), stops at a full turn either way instead of starting over, and closes
/// when it's nearly there.
fn sweep_to(current: f64, turned: f64) -> f64 {
    let nearest = [turned - TAU, turned, turned + TAU].into_iter().min_by(|a, b| (a - current).abs().total_cmp(&(b - current).abs())).unwrap_or(turned);
    let angle = nearest.clamp(-TAU, TAU);
    if angle.abs() > TAU - CLOSE_DEGREES.to_radians() { TAU.copysign(angle) } else { angle }
}

/// Center Arc, or Pie with `pie`.
pub struct CenterArcTool {
    pie: bool,
    stage: CenterStage,
    hover: Option<Snap>,
    segments: u32,
    lock: Option<Axis>,
}

#[derive(Clone, Copy)]
enum CenterStage {
    Center,
    /// The center placed; the cursor sets where the arc starts (and its radius).
    Start { center: DVec3, normal: DVec3, start: DVec3 },
    /// Turning from `start` by `angle` (radians, either way, up to a full turn).
    Sweep { center: DVec3, normal: DVec3, start: DVec3, angle: f64 },
}

impl CenterArcTool {
    pub fn new(pie: bool) -> Self {
        Self { pie, stage: CenterStage::Center, hover: None, segments: ARC_SEGMENTS, lock: None }
    }

    pub fn is_busy(&self) -> bool {
        !matches!(self.stage, CenterStage::Center)
    }

    pub fn cancel(&mut self) {
        self.stage = CenterStage::Center;
    }

    /// Arrow keys: lay it in the plane across red, green or blue; again to unlock.
    pub fn lock_axis(&mut self, axis: Axis) {
        self.lock = if self.lock == Some(axis) { None } else { Some(axis) };
        if let CenterStage::Start { normal, .. } = &mut self.stage {
            *normal = self.lock.map_or(DVec3::Z, Axis::dir);
        }
    }

    fn plane(&self) -> Option<(DVec3, DVec3)> {
        match self.stage {
            CenterStage::Center => None,
            CenterStage::Start { center, normal, .. } | CenterStage::Sweep { center, normal, .. } => Some((center, normal)),
        }
    }

    pub fn hover(&mut self, doc: &Document, screen: Screen<'_>, cursor: Vec2) {
        self.hover = snap(doc.view().model(), screen, cursor, self.plane());
        let Some(point) = self.hover.map(|h| h.point) else { return };
        match &mut self.stage {
            CenterStage::Center => {}
            CenterStage::Start { start, .. } => *start = point,
            CenterStage::Sweep { center, normal, start, angle } => {
                *angle = sweep_to(*angle, angle_about(*start - *center, point - *center, *normal));
            }
        }
    }

    pub fn click(&mut self, doc: &mut Document, screen: Screen<'_>, cursor: Vec2) -> Result<(), String> {
        self.hover(doc, screen, cursor);
        match self.stage {
            CenterStage::Center => {
                let Some(hover) = self.hover else { return Ok(()) };
                let normal = self.lock.map_or(hover.normal, Axis::dir);
                self.stage = CenterStage::Start { center: hover.point, normal, start: hover.point };
                Ok(())
            }
            CenterStage::Start { center, normal, start } => self.set_start(center, normal, start),
            CenterStage::Sweep { center, normal, start, angle } => self.draw(doc, center, normal, start, angle),
        }
    }

    fn set_start(&mut self, center: DVec3, normal: DVec3, start: DVec3) -> Result<(), String> {
        // In the plane: off it, the start counts where it projects.
        let start = start - normal * (start - center).dot(normal);
        if start.distance(center) < EPS {
            return Err("Pick a start point away from the center".into());
        }
        self.stage = CenterStage::Sweep { center, normal, start, angle: 0.0 };
        Ok(())
    }

    /// Typed: the radius (toward the cursor) while placing the start, else the angle in degrees.
    pub fn enter(&mut self, doc: &mut Document, typed: &str) -> Result<(), String> {
        if let Some(segments) = parse_segments(typed) {
            self.segments = segments?;
            return Ok(());
        }
        match self.stage {
            CenterStage::Center => Err("Click the center first".into()),
            CenterStage::Start { center, normal, start } => {
                let radius = parse_length(typed).map_err(|e| e.to_string())?.abs();
                let toward = (start - center - normal * (start - center).dot(normal)).try_normalize().unwrap_or_else(|| woodsketch_doc::curves::plane_axes(normal).0);
                self.set_start(center, normal, center + toward * radius)
            }
            CenterStage::Sweep { center, normal, start, angle } => {
                let degrees: f64 = typed.trim().trim_end_matches('°').parse().map_err(|_| format!("\"{}\" isn't an angle", typed.trim()))?;
                let sign = if angle < 0.0 { -1.0 } else { 1.0 };
                self.draw(doc, center, normal, start, sign * degrees.abs().to_radians().min(TAU))
            }
        }
    }

    fn draw(&mut self, doc: &mut Document, center: DVec3, normal: DVec3, start: DVec3, angle: f64) -> Result<(), String> {
        if angle.abs() < 1e-6 {
            return Err("Turn away from the start to sweep the arc".into());
        }
        let points = self.points(center, normal, start, angle);
        doc.edit(|m| m.draw_polyline(&points, normal));
        self.stage = CenterStage::Center;
        Ok(())
    }

    /// The arc, and for a pie its sides back to the center.
    fn points(&self, center: DVec3, normal: DVec3, start: DVec3, angle: f64) -> Vec<DVec3> {
        let mut points = sweep_points(center, start, normal, angle, self.segments);
        // A full turn closes exactly on its start (a circle, ready to push into a hole).
        if angle.abs() >= TAU - 1e-9 {
            *points.last_mut().expect("at least two") = start;
        }
        if self.pie {
            points.insert(0, center);
            points.push(center);
        }
        points
    }

    pub fn status(&self) -> String {
        let sides = self.segments;
        match self.stage {
            CenterStage::Center => format!("Select the center. Sides: {sides} (type 24s to change). Arrow keys lay it across an axis."),
            CenterStage::Start { .. } => "Select where the arc starts, or type the radius.".into(),
            CenterStage::Sweep { .. } => "Turn and click where it ends, or type the angle.".into(),
        }
    }

    pub fn measurement_label(&self) -> &'static str {
        match self.stage {
            CenterStage::Sweep { .. } => "Angle",
            _ => "Radius",
        }
    }

    pub fn measurement(&self) -> String {
        match self.stage {
            CenterStage::Center => String::new(),
            CenterStage::Start { center, start, .. } => format_length(center.distance(start)),
            CenterStage::Sweep { angle, .. } => format!("{:.1}°", angle.to_degrees()),
        }
    }

    pub fn overlay(&self, overlay: &mut Overlay) {
        overlay.snap = self.hover;
        match self.stage {
            CenterStage::Center => {}
            CenterStage::Start { center, start, .. } => overlay.lines.push([center, start]),
            CenterStage::Sweep { center, normal, start, angle } => {
                let points = self.points(center, normal, start, angle);
                overlay.lines.extend(points.windows(2).map(|w| [w[0], w[1]]));
                if !self.pie {
                    overlay.lines.push([center, start]);
                }
            }
        }
    }
}

/// 3-Point Arc: start, a point on the arc, end.
pub struct ThreePointArcTool {
    stage: ThreeStage,
    hover: Option<Snap>,
    segments: u32,
}

#[derive(Clone, Copy)]
enum ThreeStage {
    First,
    Second { a: DVec3, normal: DVec3, b: DVec3 },
    Third { a: DVec3, b: DVec3, c: Option<DVec3> },
}

impl Default for ThreePointArcTool {
    fn default() -> Self {
        Self { stage: ThreeStage::First, hover: None, segments: ARC_SEGMENTS }
    }
}

impl ThreePointArcTool {
    pub fn is_busy(&self) -> bool {
        !matches!(self.stage, ThreeStage::First)
    }

    pub fn cancel(&mut self) {
        self.stage = ThreeStage::First;
    }

    pub fn hover(&mut self, doc: &Document, screen: Screen<'_>, cursor: Vec2) {
        let plane = match self.stage {
            ThreeStage::Second { a, normal, .. } => Some((a, normal)),
            _ => None,
        };
        self.hover = snap(doc.view().model(), screen, cursor, plane);
        let Some(point) = self.hover.map(|h| h.point) else { return };
        match &mut self.stage {
            ThreeStage::First => {}
            ThreeStage::Second { b, .. } => *b = point,
            ThreeStage::Third { c, .. } => *c = Some(point),
        }
    }

    pub fn click(&mut self, doc: &mut Document, screen: Screen<'_>, cursor: Vec2) -> Result<(), String> {
        self.hover(doc, screen, cursor);
        match self.stage {
            ThreeStage::First => {
                if let Some(h) = self.hover {
                    self.stage = ThreeStage::Second { a: h.point, normal: h.normal, b: h.point };
                }
                Ok(())
            }
            ThreeStage::Second { a, b, .. } => {
                if a.distance(b) < EPS {
                    return Err("Pick a second point away from the first".into());
                }
                self.stage = ThreeStage::Third { a, b, c: None };
                Ok(())
            }
            ThreeStage::Third { a, b, c } => {
                let Some(points) = c.and_then(|c| self.points(a, b, c)) else {
                    return Err("The three points are in a line: pick an end off it".into());
                };
                let normal = circle_through(a, b, c.expect("checked")).expect("checked").1;
                doc.edit(|m| m.draw_polyline(&points, normal));
                self.stage = ThreeStage::First;
                Ok(())
            }
        }
    }

    pub fn enter(&mut self, _doc: &mut Document, typed: &str) -> Result<(), String> {
        match parse_segments(typed) {
            Some(segments) => {
                self.segments = segments?;
                Ok(())
            }
            None => Err("Click three points: the start, one on the arc, the end. Type 24s for 24 sides.".into()),
        }
    }

    /// The arc from `a` through `b` to `c`.
    fn points(&self, a: DVec3, b: DVec3, c: DVec3) -> Option<Vec<DVec3>> {
        let (center, normal) = circle_through(a, b, c)?;
        // Counter-clockwise about `normal`, a reaches b before c.
        let angle = angle_about(a - center, c - center, normal);
        let mut points = sweep_points(center, a, normal, angle, self.segments);
        *points.last_mut().expect("at least two") = c;
        Some(points)
    }

    pub fn status(&self) -> String {
        match self.stage {
            ThreeStage::First => format!("Select the arc's start. Sides: {} (type 24s to change).", self.segments),
            ThreeStage::Second { .. } => "Select a point the arc passes through.".into(),
            ThreeStage::Third { .. } => "Select the arc's end.".into(),
        }
    }

    pub fn overlay(&self, overlay: &mut Overlay) {
        overlay.snap = self.hover;
        match self.stage {
            ThreeStage::First => {}
            ThreeStage::Second { a, b, .. } => overlay.lines.push([a, b]),
            ThreeStage::Third { a, b, c } => match c.and_then(|c| self.points(a, b, c)) {
                Some(points) => overlay.lines.extend(points.windows(2).map(|w| [w[0], w[1]])),
                None => overlay.lines.push([a, b]),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pie_quarter_makes_a_face() {
        let mut doc = Document::default();
        let mut tool = CenterArcTool::new(true);
        tool.stage = CenterStage::Start { center: DVec3::ZERO, normal: DVec3::Z, start: DVec3::X };
        tool.enter(&mut doc, "10").unwrap();
        // Turning the way of +y.
        if let CenterStage::Sweep { angle, .. } = &mut tool.stage {
            *angle = 0.1;
        }
        tool.enter(&mut doc, "90").unwrap();
        let model = doc.model();
        assert_eq!(model.faces().count(), 1, "closed by its sides");
        assert!(model.vertices().any(|(_, p)| p.abs_diff_eq(DVec3::Y * 10.0, 1e-9)), "a quarter turn round to +y");
        assert!(model.vertices().all(|(_, p)| p.length() < 10.0 + 1e-9));
    }

    #[test]
    fn turning_past_a_full_circle_closes_it_instead_of_starting_over() {
        // Turning steadily round: 300 degrees, 355 (close enough: a full circle), then past it.
        let mut angle = 0.0;
        for degrees in [60.0, 120.0, 180.0, 240.0, 300.0] {
            angle = sweep_to(angle, f64::to_radians(degrees));
        }
        assert!((angle - 300f64.to_radians()).abs() < 1e-9);
        angle = sweep_to(angle, 355f64.to_radians());
        assert_eq!(angle, TAU, "closed");
        angle = sweep_to(angle, 10f64.to_radians());
        assert_eq!(angle, TAU, "past it stays closed, not back to 10 degrees");
        angle = sweep_to(angle, 300f64.to_radians());
        assert!((angle - 300f64.to_radians()).abs() < 1e-9, "backing off opens it again");
        // The other way round too.
        assert_eq!(sweep_to(-300f64.to_radians(), 2f64.to_radians()), -TAU);
    }

    #[test]
    fn a_full_center_arc_is_a_circle_with_its_face() {
        let mut doc = Document::default();
        let mut tool = CenterArcTool::new(false);
        tool.stage = CenterStage::Start { center: DVec3::ZERO, normal: DVec3::Z, start: DVec3::X };
        tool.enter(&mut doc, "1").unwrap();
        tool.enter(&mut doc, "360").unwrap();
        assert_eq!(doc.model().faces().count(), 1, "a disk, ready to push into a hole");
        assert_eq!(doc.model().vertices().count(), 12);
        // A full pie too: the circle, with its line from the center.
        let mut doc = Document::default();
        let mut tool = CenterArcTool::new(true);
        tool.stage = CenterStage::Start { center: DVec3::ZERO, normal: DVec3::Z, start: DVec3::X };
        tool.enter(&mut doc, "1").unwrap();
        tool.enter(&mut doc, "360").unwrap();
        assert_eq!(doc.model().faces().count(), 1);
    }

    #[test]
    fn three_points_make_an_arc_through_the_middle_one() {
        let tool = ThreePointArcTool::default();
        let (a, b, c) = (DVec3::new(-5.0, 0.0, 0.0), DVec3::new(0.0, 5.0, 0.0), DVec3::new(5.0, 0.0, 0.0));
        let points = tool.points(a, b, c).unwrap();
        assert_eq!(points.len(), 13);
        assert!(points[0].abs_diff_eq(a, 1e-9) && points[12].abs_diff_eq(c, 1e-9));
        assert!(points.iter().all(|p| (p.length() - 5.0).abs() < 1e-9 && p.y >= -1e-9), "the half through (0, 5)");
        assert!(points[6].abs_diff_eq(b, 1e-9));
        assert!(tool.points(a, DVec3::ZERO, c).is_none(), "in a line");
    }
}
