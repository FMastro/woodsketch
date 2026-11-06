//! The Start page: New, Open, Import, Recent and Samples.
//!
//! SketchUp opens on a Welcome window you have to get past before the app exists. SolidCraft
//! opens the app on an empty drawing and puts a Start *page* over it, with "Back to Untitled"
//! and Esc to leave. We follow SolidCraft: a drawing is always already there, so nothing here
//! has to be cleared before the app can be tried, and leaving the page costs one key.

use eframe::egui;
use woodsketch_i18n::t;
use woodsketch_doc::samples::SAMPLES;
use woodsketch_doc::units::Units;

use crate::files::Discard;
use crate::thumbs::{Of, Thumbnails};

/// What a card shows above its title.
enum Picture<'a> {
    /// Nothing, and no room kept for anything (the New row).
    None,
    /// Being drawn: the room is kept now, so the page doesn't jump when it arrives.
    Coming,
    Ready(&'a egui::TextureHandle),
}

/// What a card measures, so the rows line up down the page. The height is the room the text
/// needs; a card with a picture adds [`thumbs::SIZE`](crate::thumbs::SIZE) above it.
const CARD_WIDTH: f32 = 248.0;
const CARD_HEIGHT: f32 = 34.0;
const SAMPLE_CARD_HEIGHT: f32 = 48.0;

/// What someone picked on the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Picked {
    /// Something that puts a different drawing on screen, so unsaved work is asked about first.
    Instead(Discard),
    /// A SketchUp model added to the drawing that's already open, which discards nothing.
    Import,
}

#[derive(Default)]
pub struct StartPage {
    /// Whether the page is over the drawing. Open at startup, unless a drawing was named on the
    /// command line.
    pub open: bool,
}

impl StartPage {
    /// Draw the page and say what was picked. Takes `recent` rather than the whole app so the
    /// page stays a view over the little it needs.
    pub fn show(&mut self, ui: &mut egui::Ui, drawing: &str, recent: &[std::path::PathBuf], thumbs: &mut Thumbnails) -> Option<Picked> {
        let mut picked = None;
        let mut import = false;
        egui::CentralPanel::default().show(ui, |ui| {
            // "Back to <drawing>" first: the way out is visible before anything on the page.
            ui.horizontal(|ui| {
                if ui.button(t!("start-back-to", "drawing" => drawing)).on_hover_text(t!("start-back-to-hint")).clicked() {
                    self.open = false;
                }
                // The key's own name, the same on every keyboard, so it is not in the catalog.
                ui.weak("Esc");
            });
            ui.add_space(12.0);

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.heading(t!("start-title"));
                ui.label(t!("start-intro"));
                ui.add_space(18.0);

                ui.strong(t!("start-new"));
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    // Inches first: WoodSketch's model unit, and what its users measure in.
                    if card(ui, CARD_WIDTH, CARD_HEIGHT, &t!("start-new-drawing"), &t!("start-inches"), None, Picture::None) {
                        picked = Some(Picked::Instead(Discard::NewIn(Units::Inches)));
                    }
                    if card(ui, CARD_WIDTH, CARD_HEIGHT, &t!("start-new-drawing"), &t!("start-millimeters"), None, Picture::None) {
                        picked = Some(Picked::Instead(Discard::NewIn(Units::Millimeters)));
                    }
                    if card(ui, CARD_WIDTH, CARD_HEIGHT, &t!("start-open"), &t!("start-open-under"), None, Picture::None) {
                        picked = Some(Picked::Instead(Discard::Open));
                    }
                    if card(ui, CARD_WIDTH, CARD_HEIGHT, &t!("start-import"), &t!("start-import-under"), None, Picture::None) {
                        import = true;
                    }
                });
                ui.add_space(22.0);

                ui.strong(t!("start-recent"));
                ui.add_space(6.0);
                if recent.is_empty() {
                    ui.weak(t!("start-recent-empty"));
                } else {
                    ui.horizontal_wrapped(|ui| {
                        for (i, path) in recent.iter().enumerate() {
                            let name = path.file_stem().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
                            let folder = path.parent().map(folder_name).unwrap_or_default();
                            let picture = thumbs.get(Of::Drawing(path.clone())).cloned();
                            let picture = picture.as_ref().map_or(Picture::Coming, Picture::Ready);
                            if card(ui, CARD_WIDTH, CARD_HEIGHT, &name, &folder, Some(&path.display().to_string()), picture) {
                                picked = Some(Picked::Instead(Discard::OpenRecent(i)));
                            }
                        }
                    });
                }
                ui.add_space(22.0);

                ui.strong(t!("start-samples"));
                ui.add_space(2.0);
                ui.weak(t!("start-samples-note"));
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    for (i, sample) in SAMPLES.iter().enumerate() {
                        let picture = thumbs.get(Of::Sample(i)).cloned();
                        let picture = picture.as_ref().map_or(Picture::Coming, Picture::Ready);
                        if card(ui, CARD_WIDTH, SAMPLE_CARD_HEIGHT, sample.name, sample.shows, None, picture) {
                            picked = Some(Picked::Instead(Discard::Sample(i)));
                        }
                    }
                });
                ui.add_space(24.0);
            });
        });

        // Esc leaves, as the button says. Checked after the page is drawn so a menu or a dialog
        // over it gets the key first.
        if self.open && ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            self.open = false;
        }
        if import {
            picked = Some(Picked::Import);
        }
        // The page does *not* close here, on the click. Opening and importing put up a file
        // dialog that can be cancelled, and closing first meant cancelling dropped you onto the
        // empty drawing underneath, which reads as "it made me a new document". New drawings
        // and samples can be interrupted too, by "Save changes?". The page closes when a
        // drawing actually arrives (`replace_document`), or when an import lands.
        picked
    }
}

/// A folder as someone would write it. The recent list keeps canonical paths so the same
/// drawing is never listed twice, and on Windows that means a `\\?\` prefix, which is an
/// instruction to the filesystem rather than anything a person wants to read.
fn folder_name(path: &std::path::Path) -> String {
    let shown = path.display().to_string();
    shown.strip_prefix(r#"\\?\"#).unwrap_or(&shown).to_owned()
}

/// One card: a picture (or the room for one still being drawn), a title, and a line saying what
/// it is. The whole thing is clickable.
///
/// Painted rather than built from a `Button` (which centres its text) or from labels in a frame
/// (each of which would be a second thing a screen reader stops on). One card is one widget,
/// describing itself by its title and its line, with the text laid out top-left.
fn card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    title: &str,
    under: &str,
    hover: Option<&str>,
    picture: Picture<'_>,
) -> bool {
    const MARGIN: f32 = 12.0;
    const RADIUS: f32 = 6.0;
    // A card keeps the same shape before and after its picture arrives, so the page doesn't
    // jump as thumbnails come in: the room is kept from the first frame.
    let picture_height = match picture {
        Picture::None => 0.0,
        Picture::Coming | Picture::Ready(_) => crate::thumbs::SIZE.y,
    };
    let size = egui::vec2(width, height + picture_height) + egui::Vec2::splat(2.0 * MARGIN);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), t!("start-card-label", "title" => title, "under" => under)));

    if ui.is_rect_visible(rect) {
        let visuals = ui.visuals();
        let stroke = if response.hovered() {
            egui::Stroke::new(1.5, visuals.selection.bg_fill)
        } else {
            visuals.widgets.noninteractive.bg_stroke
        };
        let painter = ui.painter();
        painter.rect(rect, RADIUS, visuals.extreme_bg_color, stroke, egui::StrokeKind::Inside);

        let inner = rect.shrink(MARGIN);
        let mut y = inner.min.y;
        if picture_height > 0.0 {
            let at = egui::Rect::from_min_size(egui::pos2(inner.min.x, y), egui::vec2(inner.width(), picture_height));
            match picture {
                // Drawn to the panel's color, so an empty frame reads as "nothing here yet"
                // rather than as a hole in the card.
                Picture::Coming | Picture::None => {
                    painter.rect_filled(at, 4.0, visuals.faint_bg_color);
                }
                Picture::Ready(picture) => {
                    let whole = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
                    painter.image(picture.id(), at, whole, egui::Color32::WHITE);
                }
            }
            y += picture_height + 6.0;
        }
        let heading = painter.layout(title.to_owned(), egui::TextStyle::Body.resolve(ui.style()), visuals.strong_text_color(), inner.width());
        let below = y + heading.size().y + 3.0;
        painter.galley(egui::pos2(inner.min.x, y), heading, visuals.strong_text_color());
        let line = painter.layout(under.to_owned(), egui::TextStyle::Small.resolve(ui.style()), visuals.weak_text_color(), inner.width());
        painter.galley(egui::pos2(inner.min.x, below), line, visuals.weak_text_color());
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if let Some(hover) = hover {
        return response.on_hover_text(hover).clicked();
    }
    response.clicked()
}
