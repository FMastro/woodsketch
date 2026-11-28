//! The fonts the app draws with, in one place: egui's own, the Phosphor icon glyphs in both
//! weights, and a Japanese face behind them.
//!
//! [`definitions`] is separate from [`install`] so a test can look at what the app would set
//! without needing a window or a graphics card. `tests/fonts.rs` uses it to check that every
//! character any catalog can put on screen is in a font that is really listed under the family
//! it would be drawn in -- a glyph missing there reaches the reader as an empty box, and nothing
//! else here would notice, since a label is compared as a string everywhere but in the drawing.

use std::sync::Arc;

use eframe::egui;

/// The font family of Phosphor's filled glyphs, the second colour under an icon's outline.
pub const FILL: &str = "phosphor-fill";

/// A Japanese face, which egui's Latin fonts have none of the glyphs for. It is 9 MB, and it is
/// in the binary for everyone rather than loaded per language, for the reason every catalog is:
/// one build, one file, nothing loose beside the app to go missing. Last in each family, so it
/// answers only for the characters the fonts before it don't have.
const JAPANESE: &str = "noto-sans-jp";

/// What the app draws with. Everything is registered here; the families below decide the order
/// each is reached in.
pub fn definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    // The filled glyphs, by name only: the second color under the outline.
    egui_phosphor::add_font_bytes_as_family(&mut fonts, FILL, egui_phosphor::Variant::Fill.font_bytes());
    fonts.font_data.insert(JAPANESE.to_owned(), Arc::new(egui::FontData::from_static(include_bytes!("../../../assets/fonts/NotoSansJP[wght].ttf"))));
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts.families.entry(family).or_default().push(JAPANESE.to_owned());
    }
    fonts
}

/// Hand them to egui. Called once, as the app starts.
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(definitions());
}
