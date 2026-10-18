//! Marks on the drawing: the Dimensions tool (click two points, then move out to place the
//! dimension line, click or type its distance out) and the Text tool (click a point, then where
//! the words go; it starts with something useful, like the part's name or an edge's length, and
//! typing right away replaces it). Both are kept with the guides (`Guide::Dimension`,
//! `Guide::Text`): selected, moved, copied and erased like them.

use glam::{DVec3, Vec2};
use woodsketch_doc::units::{format_length, parse_length};
use woodsketch_doc::{Document, EPS, Entity, Guide};

use crate::Overlay;
use crate::pick::{Screen, Snap, SnapKind, pick_entity, snap};

#[derive(Default)]
pub struct DimensionTool {
    hover: Option<Snap>,
    stage: DimensionStage,
}

#[derive(Default)]
enum DimensionStage {
    #[default]
    First,
    /// The first point picked; the cursor is the second.
    Second { a: DVec3, b: Option<DVec3> },
    /// Both points picked; moving out places the dimension line.
    Place { a: DVec3, b: DVec3, offset: DVec3 },
}

impl DimensionTool {
    pub fn is_busy(&self) -> bool {
        !matches!(self.stage, DimensionStage::First)
    }

    pub fn cancel(&mut self) {
        self.stage = DimensionStage::First;
    }

    pub fn hover(&mut self, doc: &Document, screen: Screen<'_>, cursor: Vec2) {
        self.hover = snap(doc.view().model(), screen, cursor, None);
        match &mut self.stage {
            DimensionStage::First => {}
            DimensionStage::Second { b, .. } => *b = self.hover.map(|s| s.point),
            DimensionStage::Place { a, b, offset } => {
                if let Some(o) = place_offset(screen, cursor, *a, *b) {
                    *offset = o;
                }
            }
        }
    }

    pub fn click(&mut self, doc: &mut Document, screen: Screen<'_>, cursor: Vec2) -> Result<(), String> {
        self.hover(doc, screen, cursor);
        match self.stage {
            DimensionStage::First => {
                let Some(hover) = self.hover else { return Ok(()) };
                // Clicking along an edge (not at its ends or middle) dimensions the whole edge.
                if hover.kind == SnapKind::OnEdge
                    && let Some(Entity::Edge(edge)) = pick_entity(doc.view().model(), screen, cursor)
                    && let Some(e) = doc.view().model().edge(edge)
                {
                    let [a, b] = doc.view().model().edge_points(e);
                    let offset = place_offset(screen, cursor, a, b).unwrap_or(DVec3::ZERO);
                    self.stage = DimensionStage::Place { a, b, offset };
                    return Ok(());
                }
                self.stage = DimensionStage::Second { a: hover.point, b: None };
                Ok(())
            }
            DimensionStage::Second { a, b } => {
                let Some(b) = b.filter(|b| b.distance(a) > EPS) else {
                    return Err("Pick a second point away from the first".into());
                };
                let offset = place_offset(screen, cursor, a, b).unwrap_or(DVec3::ZERO);
                self.stage = DimensionStage::Place { a, b, offset };
                Ok(())
            }
            DimensionStage::Place { a, b, offset } => self.finish(doc, a, b, offset),
        }
    }

    /// Typed while placing: how far out the dimension line goes (the way the mouse is).
    pub fn enter(&mut self, doc: &mut Document, typed: &str) -> Result<(), String> {
        let DimensionStage::Place { a, b, offset } = self.stage else {
            return Err("Pick two points, then type how far out the dimension goes".into());
        };
        let distance = parse_length(typed).map_err(|e| e.to_string())?;
        let out = if offset.length() > EPS { offset.normalize() } else { DVec3::Z };
        self.finish(doc, a, b, out * distance)
    }

    fn finish(&mut self, doc: &mut Document, a: DVec3, b: DVec3, offset: DVec3) -> Result<(), String> {
        doc.add_note(Guide::Dimension { a, b, offset });
        self.stage = DimensionStage::First;
        Ok(())
    }

    pub fn status(&self) -> &'static str {
        match self.stage {
            DimensionStage::First => "Pick the first point to measure from, or click an edge to dimension it.",
            DimensionStage::Second { .. } => "Pick the second point.",
            DimensionStage::Place { .. } => "Move out to place the dimension, then click, or type how far out.",
        }
    }

    pub fn measurement(&self) -> String {
        match self.stage {
            DimensionStage::Second { a, b: Some(b) } | DimensionStage::Place { a, b, .. } => format_length(a.distance(b)),
            _ => String::new(),
        }
    }

    pub fn overlay(&self, overlay: &mut Overlay) {
        overlay.snap = self.hover;
        match self.stage {
            DimensionStage::First => {}
            DimensionStage::Second { a, b } => {
                if let Some(b) = b {
                    overlay.guides.push((Guide::Dimension { a, b, offset: DVec3::ZERO }, true));
                }
            }
            DimensionStage::Place { a, b, offset } => overlay.guides.push((Guide::Dimension { a, b, offset }, true)),
        }
    }
}

/// How far out, and which way, the cursor puts a dimension of `a`..`b`: across the line from
/// it, in the plane through it that faces the camera most.
fn place_offset(screen: Screen<'_>, cursor: Vec2, a: DVec3, b: DVec3) -> Option<DVec3> {
    let along = (b - a).normalize_or_zero();
    let toward_eye = -screen.forward();
    // The plane holding the line and turned as square to the camera as it can be.
    let across = along.cross(toward_eye).normalize_or_zero();
    let normal = along.cross(across).normalize_or_zero();
    if normal == DVec3::ZERO {
        return None;
    }
    let p = screen.ray(cursor).hit_plane(a, normal)?;
    let from = p - a;
    Some(from - along * from.dot(along))
}

#[derive(Default)]
pub struct TextTool {
    hover: Option<Snap>,
    /// The point the note marks, while placing its words.
    at: Option<(DVec3, String)>,
    /// Where the words would go.
    offset: DVec3,
    /// The note just placed, whose words typing replaces (once).
    last: Option<Placed>,
}

#[derive(Clone, Copy)]
struct Placed {
    at: DVec3,
    offset: DVec3,
    model_version: u64,
}

impl TextTool {
    pub fn is_busy(&self) -> bool {
        // Waiting for words counts too, so Esc ends that.
        self.at.is_some() || self.last.is_some()
    }

    pub fn cancel(&mut self) {
        self.at = None;
        self.last = None;
    }

    /// Letters typed now are the note's words (not tool shortcuts).
    pub fn takes_words(&self, doc: &Document) -> bool {
        self.last.as_ref().is_some_and(|l| l.model_version == doc.model_version())
    }

    pub fn hover(&mut self, doc: &Document, screen: Screen<'_>, cursor: Vec2) {
        self.hover = snap(doc.view().model(), screen, cursor, None);
        if let Some((at, _)) = &self.at
            && let Some(p) = screen.ray(cursor).hit_plane(*at, -screen.forward())
        {
            self.offset = p - *at;
        }
    }

    pub fn click(&mut self, doc: &mut Document, screen: Screen<'_>, cursor: Vec2) -> Result<(), String> {
        self.hover(doc, screen, cursor);
        match &self.at {
            None => {
                let Some(hover) = self.hover else { return Ok(()) };
                let words = default_words(doc, screen, cursor);
                self.at = Some((hover.point, words));
                self.offset = DVec3::ZERO;
                self.last = None;
                Ok(())
            }
            Some((at, words)) => {
                let (at, offset, text) = (*at, self.offset, words.clone());
                doc.add_note(Guide::Text { at, offset, text });
                self.at = None;
                self.last = Some(Placed { at, offset, model_version: doc.model_version() });
                Ok(())
            }
        }
    }

    /// Typed right after placing a note: its words, instead.
    pub fn enter(&mut self, doc: &mut Document, typed: &str) -> Result<(), String> {
        let Some(last) = self.last.take().filter(|l| l.model_version == doc.model_version()) else {
            return Err("Place a note (click a point, then where the words go), then type its words".into());
        };
        doc.undo();
        doc.add_note(Guide::Text { at: last.at, offset: last.offset, text: typed.trim().to_owned() });
        // Done: letters are shortcuts again (Entity Info changes the words later).
        Ok(())
    }

    pub fn status(&self) -> &'static str {
        match (&self.at, &self.last) {
            (Some(_), _) => "Click where the words go (the same spot: no leader line).",
            (None, Some(_)) => "Type the note's words and press Enter, or pick a point for another note.",
            (None, None) => "Pick the point the note is about.",
        }
    }

    pub fn overlay(&self, overlay: &mut Overlay) {
        overlay.snap = self.hover;
        if let Some((at, words)) = &self.at {
            overlay.guides.push((Guide::Text { at: *at, offset: self.offset, text: words.clone() }, true));
        }
    }
}

/// What a note starts saying about what's under the cursor: a part's name, an edge's length,
/// else "Note".
fn default_words(doc: &Document, screen: Screen<'_>, cursor: Vec2) -> String {
    match crate::pick_resolved(doc, screen, cursor) {
        Some(Entity::Instance(id)) => doc.instance_name(id).unwrap_or("Note").to_owned(),
        Some(Entity::Edge(id)) => {
            let model = doc.model();
            model.edge(id).map_or_else(|| "Note".to_owned(), |e| {
                let [a, b] = model.edge_points(e);
                format_length(a.distance(b))
            })
        }
        _ => "Note".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use woodsketch_view::{OrbitCamera, StandardView};

    use super::*;

    const SIZE: Vec2 = Vec2::new(1000.0, 800.0);

    fn camera() -> OrbitCamera {
        let mut camera = OrbitCamera { target: glam::Vec3::new(10.0, 10.0, 0.0), distance: 60.0, ..Default::default() };
        camera.set_view(StandardView::Top);
        camera
    }

    fn at(screen: Screen<'_>, x: f64, y: f64) -> Vec2 {
        screen.project(DVec3::new(x, y, 0.0)).unwrap()
    }

    #[test]
    fn dimension_two_points_and_place_it_out() {
        let mut doc = Document::default();
        let camera = camera();
        let screen = Screen::new(&camera, SIZE);
        let mut tool = DimensionTool::default();
        tool.click(&mut doc, screen, at(screen, 0.0, 0.0)).unwrap();
        tool.hover(&doc, screen, at(screen, 20.0, 0.0));
        assert!(tool.measurement().starts_with("20\"") || tool.measurement().contains("20"), "{}", tool.measurement());
        tool.click(&mut doc, screen, at(screen, 20.0, 0.0)).unwrap();
        tool.hover(&doc, screen, at(screen, 10.0, -3.0));
        tool.enter(&mut doc, "2").unwrap();
        let notes: Vec<Guide> = doc.model().guides().map(|(_, g)| g.clone()).collect();
        let [Guide::Dimension { a, b, offset }] = notes[..] else { panic!("{notes:?}") };
        assert!((a.distance(b) - 20.0).abs() < 0.3, "{a} {b}");
        assert!((offset.length() - 2.0).abs() < 1e-9 && offset.y < 0.0, "2\" out toward the cursor: {offset}");
        assert!((offset.dot(b - a)).abs() < 1e-6, "square to the line");
    }

    #[test]
    fn clicking_along_an_edge_dimensions_it() {
        let mut doc = Document::default();
        doc.edit(|m| m.add_line(DVec3::new(0.0, 0.0, 0.0), DVec3::new(16.0, 0.0, 0.0)));
        let camera = camera();
        let screen = Screen::new(&camera, SIZE);
        let mut tool = DimensionTool::default();
        tool.click(&mut doc, screen, at(screen, 5.0, 0.0)).unwrap();
        assert_eq!(tool.measurement(), format_length(16.0), "the whole edge");
        tool.click(&mut doc, screen, at(screen, 8.0, -2.0)).unwrap();
        let [Guide::Dimension { a, b, .. }] = doc.model().guides().map(|(_, g)| g.clone()).collect::<Vec<_>>()[..] else { panic!() };
        assert!((a.distance(b) - 16.0).abs() < 1e-9);
    }

    #[test]
    fn a_note_starts_with_its_default_words_and_typing_replaces_them() {
        let mut doc = Document::default();
        let camera = camera();
        let screen = Screen::new(&camera, SIZE);
        let mut tool = TextTool::default();
        tool.click(&mut doc, screen, at(screen, 5.0, 5.0)).unwrap();
        tool.click(&mut doc, screen, at(screen, 9.0, 8.0)).unwrap();
        let text = |doc: &Document| doc.model().guides().find_map(|(_, g)| if let Guide::Text { text, .. } = g { Some(text.clone()) } else { None });
        assert_eq!(text(&doc).as_deref(), Some("Note"));
        assert!(tool.takes_words(&doc));
        tool.enter(&mut doc, "Glue only the middle").unwrap();
        assert_eq!(text(&doc).as_deref(), Some("Glue only the middle"));
        assert_eq!(doc.model().guides().count(), 1, "replaced, not another");
        doc.undo();
        assert_eq!(doc.model().guides().count(), 0, "one undo takes the note away");
    }
}
