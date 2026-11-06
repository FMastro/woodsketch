//! The Cut List window: parts grouped by material with the stock each is cut from, cutting
//! diagrams with a shopping list, and the saw kerf (each material's standard stock is set in the
//! Materials panel). Exports a CSV, or a cut sheet to print.

use eframe::egui;
use woodsketch_i18n::t;
use egui_phosphor::regular as ph;
use woodsketch_doc::cutlist::{self, CutList, MaterialParts};
use woodsketch_doc::cutplan::{self, Plan, Settings, match_stock};
use woodsketch_doc::units::{format_length, parse_length};
use woodsketch_doc::{Document, MaterialKind, cutsheet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Parts,
    Diagrams,
    Movement,
    Settings,
}

#[derive(Default)]
pub struct CutListWindow {
    pub open: bool,
    pub tab: Tab,
    /// The kerf as typed (and the drawing version it was read at).
    kerf_text: Option<(u64, String)>,
    /// The list and plans, for the drawing version they were made from.
    cache: Option<Cache>,
    message: Option<String>,
}

struct Cache {
    model_version: u64,
    list: CutList,
    plans: Vec<Option<Plan>>,
}

impl CutListWindow {
    /// The current cut list and plans, recomputed when the drawing changes.
    pub fn refresh(&mut self, doc: &Document) -> (&CutList, &[Option<Plan>]) {
        if self.cache.as_ref().is_none_or(|c| c.model_version != doc.model_version()) {
            let list = cutlist::cut_list(doc.scene());
            let plans = cutplan::plan_all(doc.scene(), &list);
            self.cache = Some(Cache { model_version: doc.model_version(), list, plans });
        }
        let cache = self.cache.as_ref().expect("just filled");
        (&cache.list, &cache.plans)
    }

    /// Show the window. Returns a material to open in the Materials panel ("Edit material").
    pub fn show(&mut self, ctx: &egui::Context, doc: &mut Document) -> Option<woodsketch_doc::MaterialId> {
        if !self.open {
            return None;
        }
        let mut edit = None;
        let mut open = self.open;
        egui::Window::new(format!("{} {}", ph::TABLE, t!("cutlist-title")))
            .id(egui::Id::new("cut_list"))
            .open(&mut open)
            .default_size([760.0, 540.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (tab, label) in [
                        (Tab::Parts, t!("cutlist-tab-parts")),
                        (Tab::Diagrams, t!("cutlist-tab-diagrams")),
                        (Tab::Movement, t!("cutlist-tab-movement")),
                        (Tab::Settings, t!("cutlist-tab-settings")),
                    ] {
                        if ui.selectable_label(self.tab == tab, label).clicked() {
                            self.tab = tab;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let print = ui
                            .button(format!("{} {}", ph::PRINTER, t!("cutlist-print-cut-sheet")))
                            .on_hover_text(t!("cutlist-print-cut-sheet-hint"));
                        if print.clicked() {
                            self.export_sheet(doc);
                        }
                        let build = ui
                            .button(format!("{} {}", ph::NOTEBOOK, t!("cutlist-print-build-sheets")))
                            .on_hover_text("A page per part for the shop: its size, drawings, joints and profiles, in order");
                        if build.clicked() {
                            self.export_build_sheets(doc);
                        }
                        if ui.button(format!("{} {}", ph::FILE_CSV, t!("cutlist-export-csv"))).clicked() {
                            self.export_csv(doc);
                        }
                    });
                });
                if let Some(message) = &self.message {
                    ui.colored_label(ui.visuals().warn_fg_color, message);
                }
                ui.separator();
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.tab {
                    Tab::Parts => {
                        let (list, _) = self.refresh(doc);
                        parts_tab(ui, list, doc, &mut edit);
                    }
                    Tab::Diagrams => {
                        let (list, plans) = self.refresh(doc);
                        diagrams_tab(ui, list, plans, doc);
                    }
                    Tab::Movement => movement_tab(ui, doc),
                    Tab::Settings => self.settings_tab(ui, doc),
                });
            });
        self.open = open;
        edit
    }

    fn export_csv(&mut self, doc: &Document) {
        let (list, _) = self.refresh(doc);
        let csv = cutlist::to_csv(list);
        let Some(path) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).set_file_name("cut list.csv").save_file() else {
            return;
        };
        self.message = match std::fs::write(&path, csv) {
            Ok(()) => None,
            Err(e) => Some(t!("cutlist-save-failed", "path" => path.display().to_string(), "error" => e.to_string())),
        };
    }

    /// Save the printable cut sheet and open it in the browser to print.
    fn export_sheet(&mut self, doc: &Document) {
        let html = self.cut_sheet(doc, &t!("cutlist-sheet-title"));
        let Some(path) = rfd::FileDialog::new().add_filter("Web page", &["html"]).set_file_name("cut sheet.html").save_file() else {
            return;
        };
        self.message = match std::fs::write(&path, html) {
            Ok(()) => open_with_system(&path).err().map(|e| t!("cutlist-saved-not-opened", "error" => e.to_string())),
            Err(e) => Some(t!("cutlist-save-failed", "path" => path.display().to_string(), "error" => e.to_string())),
        };
    }

    pub fn export_build_sheets(&mut self, doc: &Document) {
        let html = self.build_sheets(doc, &t!("cutlist-build-sheets-title"));
        let Some(path) = rfd::FileDialog::new().add_filter("Web page", &["html"]).set_file_name("build sheets.html").save_file() else {
            return;
        };
        self.message = match std::fs::write(&path, html) {
            Ok(()) => open_with_system(&path).err().map(|e| t!("cutlist-saved-not-opened", "error" => e.to_string())),
            Err(e) => Some(t!("cutlist-save-failed", "path" => path.display().to_string(), "error" => e.to_string())),
        };
    }

    /// The build sheets as a web page: one page per part.
    pub fn build_sheets(&mut self, doc: &Document, title: &str) -> String {
        let (list, _) = self.refresh(doc);
        woodsketch_doc::buildsheet::to_html(doc.scene(), list, title)
    }

    /// The cut sheet as a web page.
    pub fn cut_sheet(&mut self, doc: &Document, title: &str) -> String {
        let (list, plans) = self.refresh(doc);
        cutsheet::to_html(doc.scene(), list, plans, title)
    }

    fn settings_tab(&mut self, ui: &mut egui::Ui, doc: &mut Document) {
        // Retyped from the drawing whenever it changes underneath (undo, open).
        let version = doc.model_version();
        if self.kerf_text.as_ref().is_none_or(|(v, _)| *v != version) {
            self.kerf_text = Some((version, format_length(doc.scene().cut_settings.kerf)));
        }
        let (_, text) = self.kerf_text.as_mut().expect("just filled");
        ui.label(t!("cutlist-kerf-note"));
        ui.weak(t!("cutlist-stock-note"));
        ui.add_space(6.0);
        let changed = ui
            .horizontal(|ui| {
                ui.label(t!("cutlist-saw-kerf"));
                ui.add(egui::TextEdit::singleline(text).desired_width(100.0).hint_text("1/8\"")).changed()
            })
            .inner;
        if changed {
            match parse_length(text).ok().filter(|k| *k >= 0.0) {
                Some(kerf) => {
                    doc.set_cut_settings(Settings { kerf });
                    self.message = None;
                }
                None => self.message = Some(t!("cutlist-kerf-unreadable")),
            }
            // Keep what's typed: this edit isn't a change from elsewhere.
            if let Some((v, _)) = &mut self.kerf_text {
                *v = doc.model_version();
            }
        }
    }
}

/// Open a file with the system's default program (the browser, for a web page).
fn open_with_system(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(windows)]
    let mut command = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]).arg(path);
        c
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut c = std::process::Command::new("open");
        c.arg(path);
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(path);
        c
    };
    command.spawn().map(|_| ())
}

fn kind_name(material: &MaterialParts) -> &'static str {
    material.kind.map_or("no material", MaterialKind::name)
}

/// The total that fits the material: board feet, square feet or linear feet.
fn amount(material: &MaterialParts) -> String {
    match material.kind {
        Some(MaterialKind::SheetGood) => t!("cutlist-square-feet", "amount" => format!("{:.1}", material.square_feet())),
        Some(MaterialKind::DimensionalLumber) => t!("cutlist-linear-feet", "amount" => format!("{:.1}", material.linear_feet())),
        Some(MaterialKind::SolidWood) => t!("cutlist-board-feet", "amount" => format!("{:.2}", material.board_feet())),
        _ => t!("cutlist-pieces", "count" => material.part_count()),
    }
}

fn color32(doc: &Document, material: &MaterialParts) -> egui::Color32 {
    let rgb = material.material.and_then(|m| doc.scene().material(m)).map_or([230, 230, 225], |m| m.color);
    egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

fn parts_tab(ui: &mut egui::Ui, list: &CutList, doc: &Document, edit: &mut Option<woodsketch_doc::MaterialId>) {
    if list.materials.is_empty() {
        ui.label(t!("cutlist-no-parts"));
    }
    let needs = list.needs_material();
    if needs > 0 {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t!("cutlist-needs-material", "count" => needs, "warning" => ph::WARNING),
        );
    }
    if list.loose_faces > 0 {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t!("cutlist-loose-faces", "count" => list.loose_faces),
        );
    }
    // What still needs a material first: that's what to fix.
    let ordered = list.materials.iter().filter(|m| m.material.is_none()).chain(list.materials.iter().filter(|m| m.material.is_some()));
    for material in ordered {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, color32(doc, material));
            match material.material {
                None => {
                    ui.colored_label(ui.visuals().warn_fg_color, egui::RichText::new(t!("cutlist-needs-a-material")).strong());
                    ui.weak(t!("cutlist-parts-count", "count" => material.part_count()));
                }
                Some(id) => {
                    ui.strong(&material.name);
                    ui.weak(t!("cutlist-material-line",
                        "kind" => kind_name(material),
                        "parts" => t!("cutlist-parts-count", "count" => material.part_count()),
                        "amount" => amount(material)));
                    if ui.small_button(format!("{} {}", ph::PENCIL_SIMPLE, t!("cutlist-edit-material"))).clicked() {
                        *edit = Some(id);
                    }
                }
            }
        });
        egui::Grid::new(("parts", &material.name)).striped(true).num_columns(7).spacing([16.0, 4.0]).show(ui, |ui| {
            for heading in [
                String::new(),
                t!("cutlist-column-name"),
                t!("cutlist-column-qty"),
                t!("cutlist-column-length"),
                t!("cutlist-column-width"),
                t!("cutlist-column-thickness"),
                t!("cutlist-column-stock"),
            ] {
                ui.strong(heading);
            }
            ui.end_row();
            for part in &material.parts {
                ui.strong(&part.label);
                ui.label(part.names.join(", "));
                ui.label(part.count.to_string());
                ui.label(format_length(part.length));
                ui.label(format_length(part.width));
                ui.label(format_length(part.thickness));
                // The standard stock it's cut from, or why there's none (flagged).
                match material.kind.map(|kind| match_stock(part, kind, &material.standard)) {
                    Some(Ok(stock)) => match (material.kind, cutplan::glue_up(part, &material.standard)) {
                        (Some(MaterialKind::SolidWood), Some(boards)) => {
                            let (length, width) = cutplan::glue_up_board(part, boards);
                            ui.label(t!("cutlist-glued-up", "stock" => stock.name, "boards" => boards)).on_hover_text(t!(
                                "cutlist-glued-up-hint",
                                "boards" => boards,
                                "width" => format_length(width),
                                "length" => format_length(length)
                            ));
                        }
                        _ => {
                            ui.label(stock.name);
                        }
                    },
                    Some(Err(reason)) if material.kind.is_some_and(|k| !matches!(k, MaterialKind::Hardware | MaterialKind::Other)) => {
                        ui.colored_label(ui.visuals().warn_fg_color, format!("{} {}", ph::WARNING, t!("cutlist-no-stock"))).on_hover_text(reason);
                    }
                    _ => {
                        ui.weak("—");
                    }
                }
                ui.end_row();
            }
        });
    }
    // Everything besides wood: dowels, biscuits, loose tenons, screws.
    let hardware = list.hardware(doc.scene());
    if !hardware.is_empty() {
        ui.add_space(8.0);
        ui.strong(format!("{} {}", ph::NUT, t!("cutlist-hardware")));
        egui::Grid::new("hardware").striped(true).num_columns(2).spacing([16.0, 4.0]).show(ui, |ui| {
            for (piece, count) in hardware {
                ui.label(piece);
                ui.label(count.to_string());
                ui.end_row();
            }
        });
    }
}

fn diagrams_tab(ui: &mut egui::Ui, list: &CutList, plans: &[Option<Plan>], doc: &Document) {
    if list.materials.is_empty() {
        ui.label(t!("cutlist-nothing-to-cut"));
    }
    // Metal, glass and such (type Other) aren't cut from stock: one line for all of them.
    let not_wood: Vec<&str> =
        list.materials.iter().filter(|m| m.kind == Some(MaterialKind::Other)).map(|m| m.name.as_str()).collect();
    for (material, plan) in list.materials.iter().zip(plans) {
        if material.kind == Some(MaterialKind::Other) {
            continue;
        }
        ui.add_space(8.0);
        ui.heading(&material.name);
        let Some(plan) = plan else {
            ui.weak(match material.kind {
                Some(MaterialKind::Hardware) => t!("cutlist-hardware-not-cut"),
                None => t!("cutlist-paint-to-plan"),
                _ => t!("cutlist-type-not-cut"),
            });
            continue;
        };
        let buy: Vec<String> = plan.shopping_list().into_iter().map(|(what, n)| t!("cutlist-buy-line", "count" => n, "what" => what)).collect();
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("{} {}", ph::SHOPPING_CART, t!("cutlist-buy")));
            ui.strong(buy.join(",  "));
            ui.weak(t!("cutlist-used", "percent" => format!("{:.0}", plan.yield_fraction() * 100.0)));
        });
        for (label, reason) in &plan.unplaced {
            ui.colored_label(ui.visuals().warn_fg_color, t!("cutlist-part-problem", "label" => label.as_str(), "reason" => reason.as_str()));
        }
        let color = color32(doc, material);
        for (i, stock) in plan.stocks.iter().enumerate() {
            ui.add_space(4.0);
            ui.label(t!("cutlist-stock-line", "n" => i + 1, "description" => stock.description.as_str(), "percent" => format!("{:.0}", stock.used() * 100.0)));
            draw_stock(ui, stock, color);
        }
    }
    if !not_wood.is_empty() {
        ui.add_space(8.0);
        ui.weak(t!("cutlist-not-from-stock", "materials" => not_wood.join(", ")));
    }
}

/// A piece of stock to scale, with its parts labeled. Length runs left to right.
fn draw_stock(ui: &mut egui::Ui, stock: &cutplan::Stock, color: egui::Color32) {
    let max_width = ui.available_width().min(720.0);
    let scale = max_width / stock.length as f32;
    // Keep long, narrow lumber visible.
    let height = (stock.width as f32 * scale).max(22.0);
    let y_scale = height / stock.width as f32;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(max_width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect.expand(1.0));
    painter.rect(rect, 0.0, egui::Color32::from_gray(225), egui::Stroke::new(1.0, egui::Color32::GRAY), egui::StrokeKind::Inside);
    let fill = color.gamma_multiply(0.85);
    for p in &stock.placements {
        let min = rect.min + egui::vec2(p.x as f32 * scale, p.y as f32 * y_scale);
        let part = egui::Rect::from_min_size(min, egui::vec2(p.length as f32 * scale, p.width as f32 * y_scale));
        painter.rect(part, 1.0, fill, egui::Stroke::new(1.0, egui::Color32::from_gray(40)), egui::StrokeKind::Inside);
        let text = if part.width() > 90.0 && part.height() > 26.0 {
            format!("{}\n{} x {}", p.label, format_length(p.length), format_length(p.width))
        } else {
            p.label.clone()
        };
        let font = egui::FontId::proportional(if part.height() < 18.0 { 10.0 } else { 12.0 });
        painter.text(part.center(), egui::Align2::CENTER_CENTER, text, font, egui::Color32::BLACK);
    }
}

/// `1 part`, `3 parts`.
/// Seasonal wood movement: the moisture swing to plan for, where parts will fight each other,
/// and how much each solid-wood part moves across its grain.
fn movement_tab(ui: &mut egui::Ui, doc: &mut Document) {
    use woodsketch_doc::movement;
    ui.label(t!("movement-intro"));
    let mut swing = doc.scene().moisture_swing();
    ui.horizontal(|ui| {
        ui.label(t!("movement-swing"));
        let changed = ui
            .add(egui::DragValue::new(&mut swing).range(1.0..=12.0).speed(0.1).suffix("%"))
            .on_hover_text(t!("movement-swing-hint"))
            .changed();
        if changed {
            doc.edit_scene_merging("moisture swing", |scene, _| scene.moisture_swing = Some(swing));
        }
    });
    let report = movement::check(doc.scene(), swing);
    ui.add_space(6.0);
    // Woods it can't speak for.
    if !report.unknown.is_empty() {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            t!("movement-unknown", "info" => ph::INFO, "materials" => report.unknown.join(", ")),
        );
    }
    if report.parts.is_empty() {
        ui.weak(t!("movement-none-known"));
        return;
    }
    if report.conflicts.is_empty() {
        ui.label(t!("movement-no-conflicts", "check" => ph::CHECK_CIRCLE));
    } else {
        ui.strong(format!("{} {}", ph::WARNING, t!("movement-watch")));
        for conflict in &report.conflicts {
            ui.colored_label(ui.visuals().warn_fg_color, format!("{} {}", ph::DOT_OUTLINE, conflict.describe()));
        }
        ui.weak(t!("movement-fastened-note"));
    }
    // Meetings that would fight but are free to move.
    if !report.allowed.is_empty() {
        ui.add_space(6.0);
        ui.strong(format!("{} {}", ph::CHECK_CIRCLE, t!("movement-allowed")));
        let green = if ui.visuals().dark_mode { egui::Color32::from_rgb(110, 200, 130) } else { egui::Color32::from_rgb(40, 130, 60) };
        for allowed in &report.allowed {
            ui.colored_label(green, format!("{} {}", ph::DOT_OUTLINE, allowed.describe()));
        }
    }
    ui.add_space(8.0);
    ui.strong(t!("movement-how-much"));
    egui::Grid::new("movement_parts").striped(true).num_columns(4).spacing([16.0, 4.0]).show(ui, |ui| {
        for heading in [t!("movement-column-part"), t!("movement-column-wood"), t!("movement-column-width"), t!("movement-column-moves")] {
            ui.strong(heading);
        }
        ui.end_row();
        for part in &report.parts {
            ui.label(&part.name);
            ui.label(&part.material);
            ui.label(format_length(part.width));
            let moves = ui.label(format_length(part.movement));
            if part.assumed_flat {
                moves.on_hover_text(t!("movement-assumed-flat-hint"));
            }
            ui.end_row();
        }
    });
}
