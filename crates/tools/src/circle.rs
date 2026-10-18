//! The Circle tool (C), and Polygon (the same with straight sides you see, 6 to start). Click the center (on a face, the circle lies on it), move out and click,
//! or type the radius. Type `32s` to use 32 sides instead of 24, before or right after drawing;
//! typing a radius right after drawing resizes the circle just drawn.

use glam::{DVec3, Vec2};
use woodsketch_doc::curves::{CIRCLE_SEGMENTS, circle_points};
use woodsketch_doc::units::{format_length, parse_length};
use woodsketch_doc::{Document, EPS};

use crate::Overlay;
use crate::pick::{Screen, Snap, snap};

pub struct CircleTool {
    center: Option<(DVec3, DVec3)>,
    hover: Option<Snap>,
    radius: f64,
    segments: u32,
    last: Option<Drawn>,
    /// The Polygon tool: said as a polygon's sides and inscribed radius.
    polygon: bool,
    /// Locked with an arrow key: the circle faces along that axis (it lies on the face under the
    /// cursor, else the ground, otherwise).
    lock: Option<crate::Axis>,
    /// Polygon with Ctrl: the radius reaches the middle of a side (circumscribed about a circle
    /// that size) instead of a corner.
    circumscribed: bool,
}

#[derive(Clone, Copy)]
struct Drawn {
    center: DVec3,
    normal: DVec3,
    radius: f64,
    model_version: u64,
}

impl Default for CircleTool {
    fn default() -> Self {
        Self { center: None, hover: None, radius: 0.0, segments: CIRCLE_SEGMENTS, last: None, polygon: false, lock: None, circumscribed: false }
    }
}

/// `24s`: a number of sides (segments) for circles and arcs.
pub(crate) fn parse_segments(typed: &str) -> Option<Result<u32, String>> {
    let number = typed.trim().strip_suffix(['s', 'S'])?;
    Some(match number.trim().parse::<u32>() {
        Ok(n) if (3..=999).contains(&n) => Ok(n),
        _ => Err("Sides must be a whole number from 3 to 999, like 24s".into()),
    })
}

impl CircleTool {
    /// The Polygon tool: a hexagon to start; type `8s` for an octagon.
    pub fn polygon() -> Self {
        Self { segments: 6, polygon: true, ..Self::default() }
    }

    pub fn is_busy(&self) -> bool {
        self.center.is_some()
    }

    pub fn cancel(&mut self) {
        self.center = None;
        self.last = None;
    }

    /// Polygon's Ctrl: the radius to the middle of a side, or to a corner.
    pub fn toggle_circumscribed(&mut self) {
        if self.polygon {
            self.circumscribed = !self.circumscribed;
        }
    }

    /// How far the corners are for a `radius` as given.
    fn corner_radius(&self, radius: f64) -> f64 {
        if self.polygon && self.circumscribed { radius / (std::f64::consts::PI / f64::from(self.segments)).cos() } else { radius }
    }

    /// Arrow keys: face the circle along red (Right), green (Left) or blue (Up, Down); again to
    /// unlock.
    pub fn lock_axis(&mut self, axis: crate::Axis) {
        self.lock = if self.lock == Some(axis) { None } else { Some(axis) };
        if let Some((center, _)) = self.center {
            self.center = Some((center, self.lock.map_or(DVec3::Z, crate::Axis::dir)));
        }
    }

    pub fn hover(&mut self, doc: &Document, screen: Screen<'_>, cursor: Vec2) {
        let model = doc.view().model();
        self.hover = snap(model, screen, cursor, self.center);
        if let (Some((center, _)), Some(hover)) = (self.center, self.hover) {
            self.radius = hover.point.distance(center);
        }
    }

    pub fn click(&mut self, doc: &mut Document, screen: Screen<'_>, cursor: Vec2) -> Result<(), String> {
        self.hover(doc, screen, cursor);
        match self.center {
            None => {
                let Some(hover) = self.hover else { return Ok(()) };
                let normal = self.lock.map_or(hover.normal, crate::Axis::dir);
                self.center = Some((hover.point, normal));
                self.radius = 0.0;
                self.last = None;
                Ok(())
            }
            Some(_) => self.draw(doc, self.radius),
        }
    }

    pub fn enter(&mut self, doc: &mut Document, typed: &str) -> Result<(), String> {
        if let Some(segments) = parse_segments(typed) {
            self.segments = segments?;
            return match self.last {
                Some(last) if last.model_version == doc.model_version() => self.redraw(doc, last, last.radius),
                _ => Ok(()),
            };
        }
        // `[x,y,z]` before the first click: the center there (facing up, or along a locked axis).
        if self.center.is_none() && typed.trim().starts_with('[') {
            let center = crate::move_tool::parse_offset(typed, DVec3::ZERO, DVec3::ZERO)?;
            self.center = Some((center, self.lock.map_or(DVec3::Z, crate::Axis::dir)));
            self.radius = 0.0;
            self.last = None;
            return Ok(());
        }
        let radius = parse_length(typed).map_err(|e| e.to_string())?.abs();
        if self.center.is_some() {
            return self.draw(doc, radius);
        }
        match self.last {
            Some(last) if last.model_version == doc.model_version() => self.redraw(doc, last, radius),
            _ => Err("Click the center, then type the radius".into()),
        }
    }

    pub fn status(&self) -> String {
        let sides = self.segments;
        if self.center.is_some() {
            let what = if self.polygon { "polygon" } else { "circle" };
            format!("Select a corner of the {what} or enter its radius. Sides: {sides} (type 8s to change).")
        } else {
            let ctrl = match (self.polygon, self.circumscribed) {
                (false, _) => "",
                (true, false) => " Radius to a corner (tap Ctrl: to a side).",
                (true, true) => " Radius to a side's middle (tap Ctrl: to a corner).",
            };
            format!("Select center point (or type [x,y,z]). Sides: {sides} (type {}s to change). Arrow keys face it along an axis.{ctrl}", if self.polygon { 8 } else { 32 })
        }
    }

    pub fn measurement(&self) -> String {
        match (self.center, self.last) {
            (Some(_), _) => format_length(self.radius),
            (None, Some(last)) => format_length(last.radius),
            _ => String::new(),
        }
    }

    pub fn overlay(&self, overlay: &mut Overlay) {
        overlay.snap = self.hover;
        let Some((center, normal)) = self.center else { return };
        if self.radius > EPS {
            let points = circle_points(center, normal, self.corner_radius(self.radius), self.segments);
            for i in 0..points.len() {
                overlay.lines.push([points[i], points[(i + 1) % points.len()]]);
            }
            if let Some(hover) = self.hover {
                overlay.guide = Some(crate::Guide { from: center, to: hover.point, along: None });
            }
        }
    }

    fn draw(&mut self, doc: &mut Document, radius: f64) -> Result<(), String> {
        let Some((center, normal)) = self.center else { return Ok(()) };
        if radius < EPS {
            return Err("Move away from the center, or type the radius".into());
        }
        let corner = self.corner_radius(radius);
        doc.edit(|m| m.draw_circle(center, normal, corner, self.segments));
        self.center = None;
        self.last = Some(Drawn { center, normal, radius, model_version: doc.model_version() });
        Ok(())
    }

    fn redraw(&mut self, doc: &mut Document, last: Drawn, radius: f64) -> Result<(), String> {
        if radius < EPS {
            return Err("Radius must be greater than zero".into());
        }
        doc.undo();
        self.center = Some((last.center, last.normal));
        self.draw(doc, radius)
    }
}

#[cfg(test)]
mod tests {
    use woodsketch_view::{OrbitCamera, StandardView};

    use super::*;

    const SIZE: Vec2 = Vec2::new(1000.0, 800.0);

    #[test]
    fn a_typed_center_then_a_typed_radius() {
        let mut doc = Document::default();
        let mut tool = CircleTool::default();
        tool.enter(&mut doc, "[10,0,0]").unwrap();
        tool.enter(&mut doc, "2").unwrap();
        let model = doc.model();
        assert_eq!(model.faces().count(), 1);
        assert!(model.vertices().all(|(_, p)| (p.distance(DVec3::X * 10.0) - 2.0).abs() < 1e-9 && p.z.abs() < 1e-12));
    }

    fn screen_setup() -> OrbitCamera {
        let mut camera = OrbitCamera { target: glam::Vec3::new(5.0, 5.0, 0.0), distance: 40.0, ..Default::default() };
        camera.set_view(StandardView::Top);
        camera
    }

    fn at(screen: Screen<'_>, x: f64, y: f64) -> Vec2 {
        screen.project(DVec3::new(x, y, 0.0)).unwrap()
    }

    #[test]
    fn click_the_center_and_type_the_radius() {
        let mut doc = Document::default();
        let camera = screen_setup();
        let screen = Screen::new(&camera, SIZE);
        let mut tool = CircleTool::default();
        tool.click(&mut doc, screen, at(screen, 5.0, 5.0)).unwrap();
        tool.hover(&doc, screen, at(screen, 8.0, 5.0));
        assert!((tool.radius - 3.0).abs() < 0.1);
        tool.enter(&mut doc, "1 1/2").unwrap();
        assert_eq!(doc.model().edges().count(), 24);
        let (lo, hi) = doc.model().bounds().unwrap();
        assert!(((hi - lo).x - 3.0).abs() < 1e-9, "1 1/2\" radius");
        // More sides, then a new radius: each replaces the circle just drawn.
        tool.enter(&mut doc, "48s").unwrap();
        assert_eq!(doc.model().edges().count(), 48);
        tool.enter(&mut doc, "2").unwrap();
        assert_eq!(doc.model().edges().count(), 48);
        assert_eq!(tool.measurement(), "2\"");
        doc.undo();
        assert!(doc.model().is_empty());
    }

    #[test]
    fn a_polygon_is_a_hexagon_until_you_type_its_sides() {
        let mut doc = Document::default();
        let camera = screen_setup();
        let screen = Screen::new(&camera, SIZE);
        let mut tool = CircleTool::polygon();
        assert!(tool.status().contains("Sides: 6"));
        tool.click(&mut doc, screen, at(screen, 5.0, 5.0)).unwrap();
        tool.enter(&mut doc, "3").unwrap();
        assert_eq!(doc.model().edges().count(), 6, "a hexagon");
        assert_eq!(doc.model().faces().count(), 1);
        tool.enter(&mut doc, "8s").unwrap();
        assert_eq!(doc.model().edges().count(), 8, "now an octagon in its place");
    }

    #[test]
    fn ctrl_measures_a_polygon_to_its_sides() {
        let mut doc = Document::default();
        let camera = screen_setup();
        let screen = Screen::new(&camera, SIZE);
        let mut tool = CircleTool::polygon();
        tool.enter(&mut doc, "4s").unwrap();
        tool.toggle_circumscribed();
        assert!(tool.status().contains("to a side's middle"));
        tool.click(&mut doc, screen, at(screen, 5.0, 5.0)).unwrap();
        tool.enter(&mut doc, "1").unwrap();
        // A square around a 1" circle: its corners 1.414" out, its sides 1" from the center.
        let far = doc.model().vertices().map(|(_, p)| p.distance(DVec3::new(5.0, 5.0, 0.0))).fold(0.0, f64::max);
        assert!((far - 2f64.sqrt()).abs() < 0.02, "{far}");
    }

    #[test]
    fn an_arrow_key_stands_the_circle_up() {
        let mut doc = Document::default();
        let camera = screen_setup();
        let screen = Screen::new(&camera, SIZE);
        let mut tool = CircleTool::default();
        tool.lock_axis(crate::Axis::Y);
        tool.click(&mut doc, screen, at(screen, 5.0, 5.0)).unwrap();
        tool.enter(&mut doc, "2").unwrap();
        let (lo, hi) = doc.model().bounds().unwrap();
        assert!((hi.y - lo.y).abs() < 1e-9, "facing green: flat in y");
        assert!((hi.z - lo.z - 4.0).abs() < 0.2, "standing up: {lo} {hi}");
    }

    #[test]
    fn segment_counts_parse() {
        assert_eq!(parse_segments("32s"), Some(Ok(32)));
        assert!(matches!(parse_segments("2s"), Some(Err(_))));
        assert_eq!(parse_segments("3/4"), None);
    }
}
