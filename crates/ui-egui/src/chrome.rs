//! The window around the viewport, laid out the way SketchUp users expect on Windows: a menu
//! bar, a toolbar of standard and view buttons, the two-column Large Tool Set down the left,
//! the Default Tray of collapsible panels on the right, and the status bar with the
//! Measurements box at the bottom right.

use eframe::egui;
use woodsketch_doc::DefinitionKind;
use woodsketch_doc::export::ExportUnits;
use woodsketch_tools::{ToolId, Tools};
use woodsketch_view::StandardView;

use crate::{WoodSketchApp, files, icons, outliner, tags};

const BUTTON_SIZE: f32 = 32.0;

/// The Large Tool Set, two columns, grouped as in SketchUp.
const TOOL_SET: &[&[ToolId]] = &[
    &[ToolId::Select, ToolId::Eraser, ToolId::PaintBucket],
    &[
        ToolId::Line,
        ToolId::Arc,
        ToolId::CenterArc,
        ToolId::ThreePointArc,
        ToolId::Pie,
        ToolId::Rectangle,
        ToolId::RotatedRectangle,
        ToolId::Circle,
        ToolId::Polygon,
        ToolId::Board,
    ],
    &[ToolId::PushPull, ToolId::Move, ToolId::Rotate, ToolId::Scale, ToolId::Offset],
    &[ToolId::TapeMeasure, ToolId::Protractor, ToolId::Dimension, ToolId::Text],
    &[ToolId::Orbit, ToolId::Pan, ToolId::Zoom, ToolId::Walk],
];

const VIEWS: [(&str, StandardView); 4] = [
    ("Iso", StandardView::Iso),
    ("Top", StandardView::Top),
    ("Front", StandardView::Front),
    ("Right", StandardView::Right),
];

/// A light theme in the spirit of SketchUp's Windows interface: pale grey chrome, white fields,
/// a soft blue selection.
pub fn visuals(dark: bool) -> egui::Visuals {
    if dark {
        // Dark mode: soft charcoal panels (not black), the same blue selection, dimmed.
        let mut v = egui::Visuals::dark();
        v.panel_fill = egui::Color32::from_gray(36);
        v.window_fill = egui::Color32::from_gray(42);
        v.extreme_bg_color = egui::Color32::from_gray(24);
        v.faint_bg_color = egui::Color32::from_gray(46);
        v.selection.bg_fill = egui::Color32::from_rgb(38, 72, 110);
        v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(140, 190, 240));
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(64));
        return v;
    }
    let mut v = egui::Visuals::light();
    v.panel_fill = egui::Color32::from_gray(240);
    v.window_fill = egui::Color32::from_gray(248);
    v.extreme_bg_color = egui::Color32::WHITE;
    v.faint_bg_color = egui::Color32::from_gray(232);
    v.selection.bg_fill = egui::Color32::from_rgb(204, 228, 247);
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 84, 153));
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(210));
    v
}

/// View > Theme's choices: the preference, its name, and how it's saved.
pub const THEMES: [(egui::ThemePreference, &str, &str); 3] = [
    (egui::ThemePreference::Light, "Light", "light"),
    (egui::ThemePreference::Dark, "Dark", "dark"),
    (egui::ThemePreference::System, "Follow System", "system"),
];

impl WoodSketchApp {
    pub(crate) fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                self.file_menu(ui);
                self.edit_menu(ui);
                self.view_menu(ui);
                self.camera_menu(ui);
                tool_menu(ui, &mut self.tools, "Draw", &[ToolId::Line, ToolId::Arc, ToolId::CenterArc, ToolId::ThreePointArc, ToolId::Pie, ToolId::Rectangle, ToolId::RotatedRectangle, ToolId::Circle, ToolId::Polygon, ToolId::Board]);
                self.tools_menu(ui);
                self.window_menu(ui);
                self.help_menu(ui);
            });
        });
    }

    fn file_menu(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.menu_button("File", |ui| {
            if ui.add(egui::Button::new("New").shortcut_text("Ctrl+N")).clicked() {
                ui.close();
                self.request(&ctx, files::Discard::New);
            }
            if ui.add(egui::Button::new("Open...").shortcut_text("Ctrl+O")).clicked() {
                ui.close();
                self.request(&ctx, files::Discard::Open);
            }
            ui.add_enabled_ui(!self.recent.is_empty(), |ui| {
                ui.menu_button("Open Recent", |ui| {
                    let mut open = None;
                    for (i, path) in self.recent.iter().enumerate() {
                        let name = path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
                        if ui.button(format!("{} {name}", i + 1)).on_hover_text(path.display().to_string()).clicked() {
                            open = Some(i);
                        }
                    }
                    if let Some(i) = open {
                        ui.close();
                        self.request(&ctx, files::Discard::OpenRecent(i));
                    }
                });
            });
            ui.separator();
            if ui.add(egui::Button::new("Save").shortcut_text("Ctrl+S")).clicked() {
                ui.close();
                self.save(false);
            }
            if ui.add(egui::Button::new("Save As...").shortcut_text("Ctrl+Shift+S")).clicked() {
                ui.close();
                self.save(true);
            }
            ui.separator();
            if ui.button("Import SketchUp Model...").on_hover_text("Adds a .skp (SketchUp 2021 or later) to this drawing").clicked() {
                ui.close();
                if let Some(path) = files::import_dialog(self.files.path.as_deref())
                    && let Err(e) = self.import_sketchup(&path)
                {
                    self.files.show_error(e);
                }
            }
            if ui.button("Print Build Sheets...").on_hover_text("A page per part with its steps, then how it all goes together, with exploded drawings; opens in your browser to print").clicked() {
                ui.close();
                self.cut_list.export_build_sheets(&self.doc);
            }
            ui.menu_button("Export", |ui| {
                let formats = [
                    ("OBJ...", files::ExportFormat::Obj, "The 3D model with its material colors, for renderers and other modelers"),
                    ("STL...", files::ExportFormat::Stl, "The 3D model, for 3D printing"),
                    ("DXF for CNC...", files::ExportFormat::Dxf, "Each part flat, seen from its face: the outline, through holes, and pockets on layers by depth"),
                    ("SVG for CNC...", files::ExportFormat::Svg, "Each part flat, seen from its face: the outline black, holes white, pockets grey"),
                ];
                for (label, format, hint) in formats {
                    if format == files::ExportFormat::Dxf {
                        ui.separator();
                    }
                    if ui.button(label).on_hover_text(hint).clicked() {
                        ui.close();
                        if let Some(path) = files::export_dialog(self.files.path.as_deref(), format)
                            && let Err(error) = self.export_to(&path, format)
                        {
                            self.files.show_error(error);
                        }
                    }
                }
                ui.separator();
                ui.label("Units");
                ui.radio_value(&mut self.export_units, ExportUnits::Inches, "Inches");
                ui.radio_value(&mut self.export_units, ExportUnits::Millimeters, "Millimeters");
            });
            ui.separator();
            if ui.button("Exit").clicked() {
                ui.close();
                self.request(&ctx, files::Discard::Close);
            }
        });
    }

    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Edit", |ui| {
            if ui.add(egui::Button::new("Undo").shortcut_text("Ctrl+Z")).clicked() {
                ui.close();
                self.undo();
            }
            if ui.add(egui::Button::new("Redo").shortcut_text("Ctrl+Y")).clicked() {
                ui.close();
                self.redo();
            }
            ui.separator();
            let selected = !self.doc.selection().is_empty();
            if ui.add_enabled(selected, egui::Button::new("Cut").shortcut_text("Ctrl+X")).clicked() {
                ui.close();
                self.tools.cut(&mut self.doc);
                crate::keys::note_copy(ui.ctx(), &self.tools);
            }
            if ui.add_enabled(selected, egui::Button::new("Copy").shortcut_text("Ctrl+C")).clicked() {
                ui.close();
                self.tools.copy(&self.doc);
                crate::keys::note_copy(ui.ctx(), &self.tools);
            }
            let can_paste = self.tools.can_paste();
            if ui.add_enabled(can_paste, egui::Button::new("Paste").shortcut_text("Ctrl+V")).on_hover_text("Follows the mouse; click to put it down").clicked() {
                ui.close();
                self.tools.paste(&mut self.doc);
            }
            if ui.add_enabled(can_paste, egui::Button::new("Paste in Place").shortcut_text("Ctrl+Shift+V")).clicked() {
                ui.close();
                self.tools.paste_in_place(&mut self.doc);
            }
            if ui.add_enabled(selected, egui::Button::new("Duplicate").shortcut_text("Ctrl+D")).on_hover_text("A copy right beside it").clicked() {
                ui.close();
                self.tools.duplicate(&mut self.doc);
            }
            ui.separator();
            if ui.add(egui::Button::new("Delete").shortcut_text("Delete")).clicked() {
                ui.close();
                self.tools.cancel();
                self.doc.erase_selection();
            }
            if ui.button("Delete Guides").clicked() {
                ui.close();
                self.tools.delete_guides(&mut self.doc);
            }
            ui.separator();
            if ui.add(egui::Button::new("Select All").shortcut_text("Ctrl+A")).clicked() {
                ui.close();
                self.doc.select_all();
            }
            if ui.button("Select None").clicked() {
                ui.close();
                self.doc.selection_mut().clear();
            }
            ui.separator();
            if ui.add(egui::Button::new("Make Group").shortcut_text("Ctrl+G")).clicked() {
                ui.close();
                self.tools.make_group(&mut self.doc, DefinitionKind::Group);
            }
            if ui.add(egui::Button::new("Make Component...").shortcut_text("G")).clicked() {
                ui.close();
                self.tools.make_group(&mut self.doc, DefinitionKind::Component);
            }
            if ui.button("Explode").clicked() {
                ui.close();
                self.tools.explode(&mut self.doc);
            }
        });
    }

    fn view_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("View", |ui| {
            ui.checkbox(&mut self.viewport.show_grid, "Grid");
            ui.checkbox(&mut self.viewport.show_movement, "Wood Movement")
                .on_hover_text("How much each solid-wood part moves across its grain: red where a part it meets holds it, green where it's free");
            ui.menu_button("Theme", |ui| {
                let current = ui.ctx().options(|o| o.theme_preference);
                for (theme, name, _) in THEMES {
                    if ui.radio(current == theme, name).on_hover_text("Dark is easier on the eyes at night; Follow System matches your computer").clicked() {
                        ui.ctx().set_theme(theme);
                        ui.close();
                    }
                }
            });
            ui.menu_button("Units", |ui| {
                use woodsketch_doc::units::{Units, set_units, units};
                for choice in Units::ALL {
                    if ui.radio(units() == choice, choice.name()).on_hover_text("How lengths are shown, and what a number typed without a unit means").clicked() {
                        set_units(choice);
                        ui.close();
                    }
                }
            });
            let mut guides = !self.doc.scene().guides_hidden;
            if ui.checkbox(&mut guides, "Guides").on_hover_text("Show or hide every guide (hidden ones aren't snapped to)").changed() {
                self.doc.set_guides_hidden(!guides);
            }
            let mut dimensions = !self.doc.scene().dimensions_hidden;
            if ui.checkbox(&mut dimensions, "Dimensions").on_hover_text("Show or hide every dimension").changed() {
                self.doc.set_dimensions_hidden(!dimensions);
            }
            let mut text = !self.doc.scene().text_hidden;
            if ui.checkbox(&mut text, "Text").on_hover_text("Show or hide every text note").changed() {
                self.doc.set_text_hidden(!text);
            }
            ui.checkbox(&mut self.show_tray, "Default Tray");
        });
    }

    fn camera_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Camera", |ui| {
            ui.menu_button("Standard Views", |ui| {
                for (name, view) in VIEWS {
                    if ui.button(name).clicked() {
                        ui.close();
                        self.viewport.camera.set_view(view);
                    }
                }
            });
            ui.separator();
            for tool in [ToolId::Orbit, ToolId::Pan, ToolId::Zoom, ToolId::Walk] {
                tool_item(ui, &mut self.tools, tool);
            }
            if ui.add(egui::Button::new("Zoom Extents").shortcut_text("Shift+Z")).clicked() {
                ui.close();
                self.viewport.request_zoom_extents();
            }
        });
    }

    fn tools_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Tools", |ui| {
            for tool in [ToolId::Select, ToolId::Eraser, ToolId::PaintBucket] {
                tool_item(ui, &mut self.tools, tool);
            }
            ui.separator();
            for tool in [ToolId::Move, ToolId::Rotate, ToolId::Scale, ToolId::PushPull, ToolId::Offset] {
                tool_item(ui, &mut self.tools, tool);
            }
            ui.separator();
            tool_item(ui, &mut self.tools, ToolId::TapeMeasure);
            tool_item(ui, &mut self.tools, ToolId::Protractor);
            tool_item(ui, &mut self.tools, ToolId::Dimension);
            tool_item(ui, &mut self.tools, ToolId::Text);
            ui.separator();
            if ui.button("Cut List...").clicked() {
                ui.close();
                self.cut_list.open = true;
            }
        });
    }

    fn window_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Window", |ui| {
            ui.checkbox(&mut self.show_tray, "Default Tray");
            ui.checkbox(&mut self.cut_list.open, "Cut List");
        });
    }

    fn help_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Help", |ui| {
            if ui.button("Check for Updates...").clicked() {
                ui.close();
                self.check_for_updates();
            }
            ui.checkbox(&mut self.check_updates, "Check for updates at startup")
                .on_hover_text("A quick look at WoodSketch's releases on GitHub when it opens; new versions download in the background");
            ui.separator();
            if ui.button("What's New...").on_hover_text("What the last few updates brought, newest first").clicked() {
                ui.close();
                self.read_the_changes();
            }
            if ui.button("Release Notes").on_hover_text("Every version's notes, on GitHub").clicked() {
                ui.close();
                ui.ctx().open_url(egui::OpenUrl::new_tab(crate::updates::RELEASES_URL));
            }
            ui.separator();
            if ui.button("Report a Problem...").on_hover_text("Tell us what went wrong: opens a GitHub issue with details that help, nothing private").clicked() {
                ui.close();
                self.feedback.start(woodsketch_doc::feedback::Kind::Problem);
            }
            if ui.button("Send Feedback...").on_hover_text("An idea or anything else: opens a GitHub issue").clicked() {
                ui.close();
                self.feedback.start(woodsketch_doc::feedback::Kind::Idea);
            }
            ui.separator();
            if ui.button("Problems and Ideas").on_hover_text("Everything people have reported or suggested, and what's been fixed, on GitHub").clicked() {
                ui.close();
                ui.ctx().open_url(egui::OpenUrl::new_tab(crate::updates::ISSUES_URL));
            }
            if ui.button("Discussions").on_hover_text("Ask how to do something, say what WoodSketch should handle next, or show a piece you've made").clicked() {
                ui.close();
                ui.ctx().open_url(egui::OpenUrl::new_tab(crate::updates::DISCUSSIONS_URL));
            }
            ui.separator();
            if ui.button("About WoodSketch").clicked() {
                ui.close();
                self.show_about = true;
            }
        });
    }

    pub(crate) fn about_window(&mut self, ctx: &egui::Context) {
        // `open` holds a borrow of the flag, so What's New is asked for through a local.
        let mut show_about = self.show_about;
        let mut changes = false;
        egui::Window::new("About WoodSketch")
            .open(&mut show_about)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("WoodSketch {}", crate::updates::VERSION));
                    changes = ui.link("What's new").clicked();
                });
                ui.label("A lightweight 3D modeler for woodworking, in the spirit of SketchUp.");
                ui.label("Not affiliated with or endorsed by Trimble.");
            });
        self.show_about = show_about;
        if changes {
            self.read_the_changes();
        }
    }

    /// What the last few updates brought, newest first. Releases are small and frequent, so one
    /// version on its own reads like nothing happened; this gathers them, each under its version,
    /// from the notes built into the app, so it works with no connection.
    pub(crate) fn changes_window(&mut self, ctx: &egui::Context) {
        if !self.show_changes {
            return;
        }
        let releases = self.changes_to_show();
        let since = self.updated_from.clone();
        let mut open = true;
        let mut read = false;
        egui::Window::new("What's New in WoodSketch")
            .open(&mut open)
            .collapsible(false)
            .default_width(520.0)
            .default_height(440.0)
            .show(ctx, |ui| {
                match (&since, releases.len()) {
                    (Some(from), 1) => ui.label(format!("One update since {from}:")),
                    (Some(from), n) => ui.label(format!("{n} updates since {from}:")),
                    (None, _) => ui.label("The most recent updates:"),
                };
                ui.separator();
                // Leave room for the buttons below: an unbounded scroll area takes the whole
                // window and pushes them off the bottom, out of reach.
                let room = (ui.available_height() - 36.0).max(80.0);
                egui::ScrollArea::vertical().max_height(room).auto_shrink([false, false]).show(ui, |ui| {
                    for release in &releases {
                        ui.horizontal(|ui| {
                            ui.heading(&release.version);
                            if release.version == crate::updates::VERSION {
                                ui.label("(this one)");
                            }
                        });
                        for item in notes_items(&release.notes) {
                            ui.label(format!("{} {item}", egui_phosphor::regular::DOT_OUTLINE));
                        }
                        ui.add_space(10.0);
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    // "Done" rather than "Close": it marks the notes read, which is why the
                    // status bar stops offering them afterwards.
                    read = ui.button("Done").on_hover_text("Marks these as read").clicked();
                    ui.hyperlink_to("Every version on GitHub", crate::updates::RELEASES_URL);
                });
            });
        if read || !open {
            self.show_changes = false;
            // Read, so the status bar needn't keep offering it.
            self.updated_from = None;
        }
    }

    /// Standard and view buttons under the menu bar.
    pub(crate) fn toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let ctx = ui.ctx().clone();
                if icon_button(ui, icons::NEW, false, true, "New (Ctrl+N)") {
                    self.request(&ctx, files::Discard::New);
                }
                if icon_button(ui, icons::OPEN, false, true, "Open (Ctrl+O)") {
                    self.request(&ctx, files::Discard::Open);
                }
                if icon_button(ui, icons::SAVE, false, true, "Save (Ctrl+S)") {
                    self.save(false);
                }
                ui.separator();
                if icon_button(ui, icons::UNDO, false, true, "Undo (Ctrl+Z)") {
                    self.undo();
                }
                if icon_button(ui, icons::REDO, false, true, "Redo (Ctrl+Y)") {
                    self.redo();
                }
                ui.separator();
                if icon_button(ui, icons::ZOOM_EXTENTS, false, true, "Zoom Extents (Shift+Z)") {
                    self.viewport.request_zoom_extents();
                }
                for (name, view) in VIEWS {
                    let label = if view == StandardView::Iso { format!("{} {name}", icons::ISO) } else { name.into() };
                    let response = ui
                        .add(egui::Button::selectable(false, label).min_size(egui::vec2(0.0, BUTTON_SIZE)))
                        .on_hover_text(format!("{name} view"));
                    if response.clicked() {
                        response.surrender_focus();
                        self.viewport.camera.set_view(view);
                    }
                }
                ui.separator();
                let open = self.cut_list.open;
                if icon_button(ui, icons::CUT_LIST, open, true, "Cut List: parts and cutting diagrams") {
                    self.cut_list.open = !open;
                }
            });
        });
    }

    /// SketchUp's Large Tool Set: every tool, two to a row.
    pub(crate) fn tool_set(&mut self, ui: &mut egui::Ui) {
        // Two buttons and the gap between them, plus the panel margins.
        let width = BUTTON_SIZE * 2.0 + 2.0 + 16.0;
        egui::Panel::left("tool set").resizable(false).exact_size(width).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
            for (i, group) in TOOL_SET.iter().enumerate() {
                if i > 0 {
                    ui.separator();
                }
                for pair in group.chunks(2) {
                    ui.horizontal(|ui| {
                        for &tool in pair {
                            let hint = format!("{} ({})", tool.name(), tool.shortcut_label());
                            let hint = if tool.is_available() { hint } else { format!("{hint}: coming soon") };
                            let selected = self.tools.active() == tool;
                            if icon_button(ui, icons::tool(tool), selected, tool.is_available(), &hint) {
                                self.tools.activate(tool);
                            }
                        }
                    });
                }
            }
        });
    }

    /// SketchUp's Default Tray: Entity Info, Materials, Tags and Outliner, each collapsible.
    pub(crate) fn tray(&mut self, ui: &mut egui::Ui) {
        // A text note double-clicked: its words, in Entity Info, ready to type over.
        if let Some(note) = self.tools.take_edit_note() {
            self.show_tray = true;
            ui.memory_mut(|m| m.request_focus(tags::note_field_id(note)));
        }
        if !self.show_tray {
            return;
        }
        egui::Panel::right("default tray").default_size(250.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Default Tray");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(icons::CLOSE).on_hover_text("Hide the tray (Window > Default Tray)").clicked() {
                        self.show_tray = false;
                    }
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                if self.tools.active() == ToolId::Board {
                    tray_section(ui, "Board", |ui| crate::board::show(ui, &mut self.doc, &mut self.tools));
                }
                tray_section(ui, "Entity Info", |ui| tags::entity_info(ui, &mut self.doc, &mut self.tools));
                if let Some(pair) = crate::joinery::selected_pair(&self.doc) {
                    tray_section(ui, "Joinery", |ui| crate::joinery::show(ui, &mut self.doc, pair));
                }
                if let [woodsketch_doc::Entity::Instance(part)] = self.doc.selection().iter().collect::<Vec<_>>()[..] {
                    if let Some(made) = self.doc.template_of(part) {
                        tray_section(ui, &format!("{} sizes", made.template.name()), |ui| crate::template::change(ui, &mut self.doc, part, made));
                    }
                    tray_section(ui, "Edge profile", |ui| crate::profile::show(ui, &mut self.doc, part));
                }
                // Folded until wanted: making a new cabinet, drawer, face frame or table.
                egui::CollapsingHeader::new(egui::RichText::new("Templates").strong())
                    .id_salt("Templates")
                    .default_open(false)
                    .show(ui, |ui| {
                        if crate::template::new(ui, &mut self.doc) {
                            self.viewport.request_zoom_extents();
                        }
                    });
                tray_section(ui, "Materials", |ui| self.materials.show(ui, &mut self.doc, &mut self.tools));
                tray_section(ui, "Tags", |ui| self.tags.show(ui, &mut self.doc));
                tray_section(ui, "Outliner", |ui| outliner::show(ui, &mut self.doc, &mut self.tools));
            });
        });
    }

    pub(crate) fn status_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    measurements_box(ui, &self.tools);
                    ui.label(self.tools.measurement_label());
                    ui.separator();
                    // Reloaded because the file changed outside (an assistant working through
                    // woodsketch-cli, a synced folder): say so briefly, so it isn't a mystery.
                    if self.reloaded_recently() {
                        ui.label(format!("{} Reloaded: the file changed outside WoodSketch", egui_phosphor::regular::ARROWS_CLOCKWISE))
                            .on_hover_text("Ctrl+Z goes back to the drawing as it was before the reload");
                        ui.separator();
                    }
                    // Just updated: say so once, with everything the update brought.
                    if self.updated() {
                        if ui.small_button(egui_phosphor::regular::X).on_hover_text("Dismiss").clicked() {
                            self.updated_from = None;
                        }
                        if ui.link("What's new").on_hover_text("What came in this update, and in any you skipped").clicked() {
                            self.read_the_changes();
                        }
                        ui.label(format!("{} Updated to {}", egui_phosphor::regular::CHECK_CIRCLE, crate::updates::VERSION));
                        ui.separator();
                    }
                    // A new version downloading, then downloaded: offer the restart (green, so it's seen),
                    // never force it.
                    if let Some(crate::updates::UpdateState::Downloading(version)) = self.update_state() {
                        ui.label(format!("Downloading WoodSketch {version}..."));
                        ui.add(egui::Spinner::new());
                        ui.separator();
                    }
                    if let Some(crate::updates::UpdateState::Ready(version)) = self.update_state() {
                        let label = egui::RichText::new(format!("{} Restart to update to {version}", egui_phosphor::regular::ARROW_CIRCLE_UP)).color(egui::Color32::WHITE).strong();
                        let restart = ui
                            .add(egui::Button::new(label).fill(egui::Color32::from_rgb(34, 139, 34)))
                            .on_hover_text("Installs the new version and reopens WoodSketch (you'll be asked to save first)");
                        if restart.clicked() {
                            let ctx = ui.ctx().clone();
                            self.restart_to_update(&ctx);
                        }
                        ui.separator();
                    }
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(icons::HELP).color(egui::Color32::from_rgb(0, 102, 204)));
                        ui.add(egui::Label::new(self.tools.status()).truncate());
                    });
                });
            });
        });
    }

    fn undo(&mut self) {
        self.tools.cancel();
        self.doc.undo();
    }

    fn redo(&mut self) {
        self.tools.cancel();
        self.doc.redo();
    }
}

/// A tray panel: a bar with the panel's name that folds it open and closed.
/// A version's notes split into its bullets, each with its wrapped lines joined into one, so
/// egui wraps them to the window's width instead of keeping the file's line breaks.
fn notes_items(notes: &str) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    for line in notes.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- ") {
            items.push(rest.to_owned());
        } else if !trimmed.is_empty() {
            match items.last_mut() {
                // A wrapped continuation of the bullet above.
                Some(item) => {
                    item.push(' ');
                    item.push_str(trimmed);
                }
                // Notes not written as bullets still show.
                None => items.push(trimmed.to_owned()),
            }
        }
    }
    items
}

fn tray_section(ui: &mut egui::Ui, title: &str, contents: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(egui::RichText::new(title).strong())
        .id_salt(title)
        .default_open(true)
        .show(ui, contents);
}

/// A menu of tools, each with its shortcut.
fn tool_menu(ui: &mut egui::Ui, tools: &mut Tools, title: &str, list: &[ToolId]) {
    ui.menu_button(title, |ui| {
        for &tool in list {
            tool_item(ui, tools, tool);
        }
    });
}

fn tool_item(ui: &mut egui::Ui, tools: &mut Tools, tool: ToolId) {
    let button = egui::Button::new(tool.name()).shortcut_text(tool.shortcut_label());
    if ui.add_enabled(tool.is_available(), button).clicked() {
        ui.close();
        tools.activate(tool);
    }
}

/// A square icon button with a tooltip. Returns whether it was clicked.
fn icon_button(ui: &mut egui::Ui, icon: &str, selected: bool, enabled: bool, hint: &str) -> bool {
    // An empty button, the icon painted over it in two colors (its name kept for screen readers
    // and tests).
    let button = egui::Button::selectable(selected, "").min_size(egui::vec2(BUTTON_SIZE, BUTTON_SIZE));
    let response = ui.add_enabled(enabled, button);
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, selected, icon));
    icons::paint(ui, response.rect, icon, 20.0, enabled);
    let response = if enabled { response.on_hover_text(hint) } else { response.on_disabled_hover_text(hint) };
    if response.clicked() {
        // Don't keep keyboard focus, or Space would press the button again.
        response.surrender_focus();
        return true;
    }
    false
}

/// SketchUp's Measurements box: shows live values, and whatever is being typed.
fn measurements_box(ui: &mut egui::Ui, tools: &Tools) {
    let text = tools.measurement();
    let color = if tools.is_typing() { ui.visuals().strong_text_color() } else { ui.visuals().text_color() };
    egui::Frame::new()
        .fill(ui.visuals().extreme_bg_color)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(170)))
        .corner_radius(2.0)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.set_min_width(150.0);
            ui.label(egui::RichText::new(text).monospace().color(color));
        });
}
