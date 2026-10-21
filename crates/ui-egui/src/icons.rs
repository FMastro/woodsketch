//! Toolbar icons from the Phosphor icon font (MIT), drawn as text glyphs. The toolbars draw them in
//! two colors, as SketchUp's are (its navy and red): the filled glyph in amber behind, the outline
//! in walnut over it, the logo's wood colors (see [`paint`]).

use eframe::egui;
use egui_phosphor::regular as ph;
use woodsketch_tools::ToolId;

/// Register the icon font as a fallback for egui's default fonts.
pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    // The filled glyphs, by name only: the second color under the outline.
    egui_phosphor::add_font_bytes_as_family(&mut fonts, FILL, egui_phosphor::Variant::Fill.font_bytes());
    ctx.set_fonts(fonts);
}

/// The font family of the filled glyphs.
const FILL: &str = "phosphor-fill";

/// Walnut and amber, from the logo (lighter for the dark theme).
const OUTLINE: egui::Color32 = egui::Color32::from_rgb(0x3a, 0x27, 0x18);
const OUTLINE_DARK: egui::Color32 = egui::Color32::from_rgb(0xe9, 0xc5, 0x8f);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(0xd0, 0x8a, 0x2e);

/// Paint `icon` centered in `rect`, `size` points: its filled shape in amber, its outline over it
/// in walnut. Greyed when not `enabled`.
pub fn paint(ui: &egui::Ui, rect: egui::Rect, icon: &str, size: f32, enabled: bool) {
    let dark = ui.visuals().dark_mode;
    let (outline, accent) = if !enabled {
        let gray = ui.visuals().weak_text_color();
        (gray, gray.gamma_multiply(0.35))
    } else if dark {
        (OUTLINE_DARK, ACCENT.gamma_multiply(0.75))
    } else {
        (OUTLINE, ACCENT.gamma_multiply(0.6))
    };
    let painter = ui.painter();
    let at = rect.center();
    painter.text(at, egui::Align2::CENTER_CENTER, icon, egui::FontId::new(size, egui::FontFamily::Name(FILL.into())), accent);
    painter.text(at, egui::Align2::CENTER_CENTER, icon, egui::FontId::proportional(size), outline);
}

pub fn tool(tool: ToolId) -> &'static str {
    match tool {
        ToolId::Select => ph::CURSOR,
        ToolId::Eraser => ph::ERASER,
        ToolId::PaintBucket => ph::PAINT_BUCKET,
        ToolId::Line => ph::LINE_SEGMENT,
        ToolId::Arc => ph::BEZIER_CURVE,
        ToolId::Rectangle => ph::RECTANGLE,
        ToolId::RotatedRectangle => ph::DIAMOND,
        ToolId::Circle => ph::CIRCLE,
        ToolId::Polygon => ph::HEXAGON,
        ToolId::CenterArc => ph::CIRCLE_HALF,
        ToolId::ThreePointArc => ph::ARROW_ARC_RIGHT,
        ToolId::Pie => ph::CHART_PIE_SLICE,
        ToolId::Protractor => ph::ANGLE,
        ToolId::Dimension => ph::ARROWS_HORIZONTAL,
        ToolId::Text => ph::TEXT_T,
        ToolId::Board => ph::LOG,
        ToolId::PushPull => ph::ARROW_FAT_LINES_UP,
        ToolId::Move => ph::ARROWS_OUT_CARDINAL,
        ToolId::Rotate => ph::ARROW_CLOCKWISE,
        ToolId::Scale => ph::RESIZE,
        ToolId::Offset => ph::FRAME_CORNERS,
        ToolId::TapeMeasure => ph::RULER,
        ToolId::Orbit => ph::ARROWS_CLOCKWISE,
        ToolId::Pan => ph::HAND,
        ToolId::Zoom => ph::MAGNIFYING_GLASS,
        ToolId::Walk => ph::PERSON_SIMPLE_WALK,
    }
}

pub const ZOOM_EXTENTS: &str = ph::CORNERS_OUT;
pub const ISO: &str = ph::CUBE;
pub const UNDO: &str = ph::ARROW_U_UP_LEFT;
pub const REDO: &str = ph::ARROW_U_UP_RIGHT;
pub const CUT_LIST: &str = ph::TABLE;
pub const NEW: &str = ph::FILE;
pub const OPEN: &str = ph::FOLDER_OPEN;
pub const SAVE: &str = ph::FLOPPY_DISK;
pub const CLOSE: &str = ph::X;
pub const HELP: &str = ph::QUESTION;
