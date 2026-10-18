//! Rotated Rectangle (SketchUp's): a rectangle at any angle. Click the first corner, click the
//! second (the first side: its direction and length, inferred like a line, or typed), then move
//! across and click (or type) the other side.

use glam::{DVec3, Vec2};
use woodsketch_doc::units::{format_length, parse_length};
use woodsketch_doc::{Document, EPS};

use crate::infer::{self, Hints, Query, Target};
use crate::move_tool::parse_offset;
use crate::pick::{Screen, Snap, snap};
use crate::{Guide, Overlay};

#[derive(Default)]
pub struct RotatedRectangleTool {
    stage: Stage,
    hover: Option<Snap>,
    hints: Hints,
    target: Option<Target>,
}

#[derive(Clone, Copy, Default)]
enum Stage {
    #[default]
    First,
    /// The first corner placed; the cursor sets the first side's end.
    Side { a: DVec3, normal: DVec3, b: DVec3 },
    /// The first side placed; the cursor sets how far across (signed, along `across`).
    Across { a: DVec3, b: DVec3, normal: DVec3, height: f64 },
}

/// The direction in the plane square to the side `a`..`b`.
fn across(a: DVec3, b: DVec3, normal: DVec3) -> DVec3 {
    normal.cross(b - a).normalize_or_zero()
}

fn corners(a: DVec3, b: DVec3, normal: DVec3, height: f64) -> [DVec3; 4] {
    let up = across(a, b, normal) * height;
    [a, b, b + up, a + up]
}

impl RotatedRectangleTool {
    pub fn is_busy(&self) -> bool {
        !matches!(self.stage, Stage::First)
    }

    pub fn cancel(&mut self) {
        self.stage = Stage::First;
    }

    pub fn hover(&mut self, doc: &Document, screen: Screen<'_>, cursor: Vec2) {
        let model = doc.view().model();
        match &mut self.stage {
            Stage::First => {
                self.hover = snap(model, screen, cursor, None);
                self.hints.note(self.hover.as_ref());
            }
            Stage::Side { a, normal, b } => {
                let (base, n) = (*a, *normal);
                let query = Query { base, hints: &self.hints, lock: None, plane_normal: n, current: *b };
                let mut target = infer::target(model, screen, cursor, &query, |_| false, |ray| ray.hit_plane(base, n).map(|p| (p, None)));
                self.hints.note(target.snap.as_ref());
                if target.direction.is_some_and(|d| d.dir.dot(n).abs() > 1e-6) {
                    target = Target { point: screen.ray(cursor).hit_plane(base, n).unwrap_or(*b), ..Target::default() };
                }
                *b = target.point - n * (target.point - base).dot(n);
                self.hover = target.snap;
                self.target = Some(target);
            }
            Stage::Across { a, b, normal, height } => {
                self.hover = snap(model, screen, cursor, Some((*a, *normal)));
                if let Some(h) = self.hover {
                    *height = (h.point - *a).dot(across(*a, *b, *normal));
                }
            }
        }
    }

    pub fn click(&mut self, doc: &mut Document, screen: Screen<'_>, cursor: Vec2) -> Result<(), String> {
        self.hover(doc, screen, cursor);
        match self.stage {
            Stage::First => {
                if let Some(h) = self.hover {
                    self.stage = Stage::Side { a: h.point, normal: h.normal, b: h.point };
                    self.target = None;
                }
                Ok(())
            }
            Stage::Side { a, normal, b } => self.set_side(a, normal, b),
            Stage::Across { a, b, normal, height } => self.draw(doc, a, b, normal, height),
        }
    }

    fn set_side(&mut self, a: DVec3, normal: DVec3, b: DVec3) -> Result<(), String> {
        if a.distance(b) < EPS {
            return Err("Pick the second corner away from the first".into());
        }
        self.stage = Stage::Across { a, b, normal, height: 0.0 };
        Ok(())
    }

    /// Typed: the first side's length (the way the mouse is), or `[x,y,z]` / `<x,y,z>` for the
    /// second corner; then the other side's length.
    pub fn enter(&mut self, doc: &mut Document, typed: &str) -> Result<(), String> {
        match self.stage {
            Stage::First => Err("Click the first corner".into()),
            Stage::Side { a, normal, b } => {
                let offset = parse_offset(typed, b - a, a)?;
                let b = a + offset - normal * offset.dot(normal);
                self.set_side(a, normal, b)
            }
            Stage::Across { a, b, normal, height } => {
                let length = parse_length(typed).map_err(|e| e.to_string())?.abs();
                let sign = if height < 0.0 { -1.0 } else { 1.0 };
                self.draw(doc, a, b, normal, sign * length)
            }
        }
    }

    fn draw(&mut self, doc: &mut Document, a: DVec3, b: DVec3, normal: DVec3, height: f64) -> Result<(), String> {
        if height.abs() < EPS {
            return Err("Move across the first side to give the rectangle its width".into());
        }
        let points = corners(a, b, normal, height);
        let facing = if normal.z.abs() > 1e-9 { normal * normal.z.signum() } else { normal };
        doc.edit(|m| m.draw_polygon(&points, facing));
        self.stage = Stage::First;
        Ok(())
    }

    pub fn status(&self) -> &'static str {
        match self.stage {
            Stage::First => "Select the first corner.",
            Stage::Side { .. } => "Select the second corner (the first side), or type its length.",
            Stage::Across { .. } => "Move across and click, or type the other side's length.",
        }
    }

    pub fn measurement(&self) -> String {
        match self.stage {
            Stage::First => String::new(),
            Stage::Side { a, b, .. } => format_length(a.distance(b)),
            Stage::Across { height, .. } => format_length(height.abs()),
        }
    }

    pub fn overlay(&self, overlay: &mut Overlay) {
        overlay.snap = self.hover;
        match self.stage {
            Stage::First => {}
            Stage::Side { a, b, .. } => {
                let target = self.target.unwrap_or_default();
                overlay.guide = Some(Guide { from: a, to: b, along: target.direction.map(|d| d.along) });
                overlay.alignment = target.from.map(|(anchor, axis)| (anchor, b, axis));
            }
            Stage::Across { a, b, normal, height } => {
                let c = corners(a, b, normal, height);
                for i in 0..4 {
                    overlay.lines.push([c[i], c[(i + 1) % 4]]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rectangle_at_45_degrees() {
        let mut doc = Document::default();
        let mut tool = RotatedRectangleTool { stage: Stage::Side { a: DVec3::ZERO, normal: DVec3::Z, b: DVec3::ZERO }, ..Default::default() };
        tool.enter(&mut doc, "[3,3,0]").unwrap();
        if let Stage::Across { height, .. } = &mut tool.stage {
            *height = 1.0;
        }
        tool.enter(&mut doc, "2").unwrap();
        let model = doc.model();
        assert_eq!(model.faces().count(), 1);
        let (_, face) = model.faces().next().unwrap();
        assert!(face.normal.abs_diff_eq(DVec3::Z, 1e-9), "faces up");
        let expected = DVec3::new(-1.0, 1.0, 0.0).normalize() * 2.0;
        assert!(model.vertices().any(|(_, p)| p.abs_diff_eq(expected, 1e-9)), "the far side 2\" across");
        assert!(model.vertices().any(|(_, p)| p.abs_diff_eq(DVec3::new(3.0, 3.0, 0.0), 1e-9)));
    }
}
