//! Pictures of the tray panels for the User Guide.
//!
//! The panels are the one part of the app a page cannot describe from the outside: what a panel
//! offers depends on what is selected, and the only honest picture of it is the panel itself,
//! drawn by the app with something real to work on. So this drives the real app with
//! `egui_kittest`, puts each panel in the state that makes it useful, and crops the tray down to
//! that panel alone.
//!
//! Each panel's rectangle comes from its own header: the accessibility tree gives the header
//! label's rect, and a panel runs from there to the top of the next one. Nothing is measured by
//! hand, so the crops follow the tray if its layout changes.
//!
//! Light and dark, because mdBook has themes of both kinds and a pale screenshot glares on a
//! dark page; `assets/diagrams.css` shows whichever matches.
//!
//! Like the rendered joint pictures this needs a GPU, so CI never runs it. Run it by hand after
//! a change to the tray, and commit what it writes:
//!
//!     cargo test -p woodsketch-ui-egui --test guide_panels -- --ignored --nocapture
//!
//! `tests/guide.rs` checks every panel has a picture, so a new panel cannot reach the guide
//! without one.

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use glam::DVec3;
use woodsketch_doc::board::Board;
use woodsketch_doc::template::{Cabinet, Template};
use woodsketch_doc::{Document, Entity, MaterialKind};
use woodsketch_ui_egui::WoodSketchApp;
use woodsketch_ui_egui::tray::Panel;

/// One at a time on the GPU, as the other harness tests do.
static GPU: std::sync::Mutex<()> = std::sync::Mutex::new(());

const WINDOW: [f32; 2] = [1200.0, 800.0];
/// The tray's column. Taken from the window rather than measured: the tray is the right-hand
/// strip, and a panel's header tells us where its own top is.
const TRAY_RIGHT: u32 = 1198;
/// From a header's label back to the edge of the tray: the fold arrow and the frame's margin.
/// Measured from the header rather than fixed, because the tray widens to fit its content --
/// the Sizes panel's label column is wider than anything else's, and a fixed edge either
/// clipped it or let a sliver of the viewport into every other picture.
const HEADER_INSET: f32 = 26.0;
/// How far below its header a panel is shown when nothing follows it.
const TAIL: u32 = 250;
/// The Sizes panel's header while the cabinet this test places is selected.
const SIZES_TITLE: &str = "Cabinet Sizes";

fn pictures() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/guide/src/images/panels")
}

/// A board standing on its length, so the drawing has parts for the panels to work on.
fn board(doc: &mut Document, origin: DVec3, length: f64, width: f64) -> Entity {
    let wood = doc.add_material(MaterialKind::SolidWood);
    let board = Board { origin, length_dir: DVec3::Z, width_dir: DVec3::Y, length, width, thickness: 0.75 };
    Entity::Instance(doc.add_board(&board, Some(wood)).expect("a board"))
}

struct Shot {
    harness: Harness<'static, WoodSketchApp>,
    dark: bool,
}

impl Shot {
    fn new(dark: bool) -> Self {
        let theme = if dark { egui::Theme::Dark } else { egui::Theme::Light };
        let mut harness = Harness::builder().with_size(WINDOW).with_step_dt(1.0 / 60.0).with_theme(theme).wgpu().build_eframe(|cc| WoodSketchApp::new(cc));
        harness.run();
        Self { harness, dark }
    }

    /// Where each panel that is on screen begins, top to bottom.
    fn tops(&mut self) -> Vec<(Panel, f32)> {
        let mut found: Vec<(Panel, f32)> = Panel::ALL
            .into_iter()
            .filter_map(|panel| {
                // Every header is the panel's name, except Sizes: that one is titled after
                // what is selected (`tray-template-sizes`, "{ $template } Sizes"), so for the
                // cabinet this test places it reads "Cabinet Sizes". Matched exactly rather
                // than by searching for "sizes", which also finds the Board panel's hint about
                // framing sizes and the Templates panel's body.
                let name = panel.name();
                let label = match panel {
                    Panel::TemplateSizes => SIZES_TITLE.to_owned(),
                    _ => name,
                };
                let top = self.harness.query_by_label(label.as_str()).map(|node| node.rect().min.y);
                top.map(|top| (panel, top))
            })
            .collect();
        found.sort_by(|a, b| a.1.total_cmp(&b.1));
        found
    }

    /// Where this panel's header label begins, which is how far in the tray's edge is.
    fn header_x(&mut self, panel: Panel) -> f32 {
        let label = if panel == Panel::TemplateSizes { SIZES_TITLE.to_owned() } else { panel.name() };
        self.harness.query_by_label(label.as_str()).map_or(990.0, |node| node.rect().min.x)
    }

    /// Crop the tray down to `panel` and write it, light or dark.
    fn take(&mut self, panel: Panel) {
        let tops = self.tops();
        let at = tops.iter().position(|(p, _)| *p == panel).unwrap_or_else(|| panic!("the {} panel isn't on screen to be pictured", panel.name()));
        let top = tops[at].1;
        let bottom = tops.get(at + 1).map_or(top + TAIL as f32, |(_, next)| *next - 6.0);

        let header_x = self.header_x(panel);
        let image = self.harness.render().expect("render the frame");
        let scale = f64::from(image.width()) / f64::from(WINDOW[0] as u32);
        let px = |v: f32| (f64::from(v) * scale).round() as u32;
        let (x, w) = (px(header_x - HEADER_INSET), px(TRAY_RIGHT as f32) - px(header_x - HEADER_INSET));
        let y = px(top - 8.0);
        let h = (px(bottom) - y).min(image.height().saturating_sub(y));

        let cropped = image::imageops::crop_imm(&image, x, y, w, h).to_image();
        let dir = pictures();
        std::fs::create_dir_all(&dir).expect("the pictures folder");
        let theme = if self.dark { "dark" } else { "light" };
        let file = dir.join(format!("{}-{theme}.png", slug(panel)));
        cropped.save(&file).expect("save the picture");
        eprintln!("  {:16} {w}x{h}", slug(panel));
    }
}

/// `Panel::EntityInfo` -> `entity-info`, the name its picture and its anchor are filed under.
fn slug(panel: Panel) -> String {
    woodsketch_ui_egui::help::panel_anchor(panel).trim_start_matches("panel-").to_owned()
}

#[test]
#[ignore = "needs a GPU and writes into docs/guide; run it by hand after changing the tray"]
fn the_panel_pictures() {
    let _gpu = GPU.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    for dark in [false, true] {
        eprintln!("{}:", if dark { "dark" } else { "light" });
        let mut shot = Shot::new(dark);

        // A drawing with two parts that meet, which is what the joinery panels want.
        let (side, shelf) = {
            let doc = shot.harness.state_mut().document_mut();
            let side = board(doc, DVec3::ZERO, 12.0, 8.0);
            let shelf = board(doc, DVec3::new(0.75, 0.0, 5.5), 9.0, 8.0);
            (side, shelf)
        };
        shot.harness.run();

        // Nothing selected: the panels that always stand there.
        for panel in [Panel::Materials, Panel::Tags, Panel::Outliner] {
            shot.take(panel);
        }

        // One part: Entity Info has something to say, and Edge Profile appears.
        shot.harness.state_mut().document_mut().selection_mut().extend([shelf]);
        shot.harness.run();
        for panel in [Panel::EntityInfo, Panel::EdgeProfile] {
            shot.take(panel);
        }

        // Two parts that meet: the Joinery panel offers the joints they suit.
        shot.harness.state_mut().document_mut().selection_mut().extend([side]);
        shot.harness.run();
        shot.take(Panel::Joinery);

        // The Board tool brings its own panel to the top of the tray.
        shot.harness.state_mut().document_mut().selection_mut().clear();
        shot.harness.run();
        type_text(&mut shot.harness, "w");
        shot.harness.run();
        shot.take(Panel::Board);

        // A cabinet in the drawing, selected: the Sizes panel holds the numbers it was built
        // from, which is the whole point of a template over a pile of boards. Taken with Select
        // in hand and Templates still folded, because this is the one panel found by searching
        // for its name inside a longer title, and both the Board panel's hint ("other framing
        // sizes") and the Templates body say "sizes" as well.
        type_text(&mut shot.harness, " ");
        let cabinet = {
            let doc = shot.harness.state_mut().document_mut();
            let wood = doc.add_material(MaterialKind::SolidWood);
            doc.add_template(&Template::Cabinet(Cabinet::default()), Some(wood)).expect("a cabinet")
        };
        shot.harness.state_mut().document_mut().selection_mut().clear();
        shot.harness.state_mut().document_mut().selection_mut().extend([Entity::Instance(cabinet)]);
        shot.harness.run();
        shot.take(Panel::TemplateSizes);

        // Templates starts folded -- it is a thing you go looking for -- so open it last.
        shot.harness.state_mut().document_mut().selection_mut().clear();
        shot.harness.run();
        shot.harness.get_by_label(Panel::Templates.name().as_str()).click();
        shot.harness.run();
        shot.take(Panel::Templates);
    }
}

/// A typed letter, which is how a tool shortcut reaches the app.
fn type_text(harness: &mut Harness<'static, WoodSketchApp>, text: &str) {
    harness.input_mut().events.push(egui::Event::Text(text.to_owned()));
    harness.run();
}
