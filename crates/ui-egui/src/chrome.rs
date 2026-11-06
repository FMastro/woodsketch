//! The window around the viewport, laid out the way SketchUp users expect on Windows: a menu
//! bar, a toolbar of standard and view buttons, the two-column Large Tool Set down the left,
//! the Default Tray of collapsible panels on the right, and the status bar with the
//! Measurements box at the bottom right.

use eframe::egui;
use woodsketch_doc::DefinitionKind;
use woodsketch_doc::export::ExportUnits;
use woodsketch_tools::{ToolId, Tools};
use woodsketch_view::StandardView;

use woodsketch_i18n::t;

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

const VIEWS: [StandardView; 4] = [StandardView::Iso, StandardView::Top, StandardView::Front, StandardView::Right];

/// What a standard view is called. A match rather than a string beside each view, so every key
/// stays a literal where `t!` can see it and the test that checks the catalog can find it.
fn view_name(view: StandardView) -> String {
    match view {
        StandardView::Iso => t!("camera-view-iso"),
        StandardView::Top => t!("camera-view-top"),
        StandardView::Front => t!("camera-view-front"),
        StandardView::Right => t!("camera-view-right"),
    }
}

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
pub const THEMES: [(egui::ThemePreference, &str); 3] = [
    (egui::ThemePreference::Light, "light"),
    (egui::ThemePreference::Dark, "dark"),
    (egui::ThemePreference::System, "system"),
];

/// Which language the app speaks. Only shown once there is a choice: with English alone it would
/// be a menu with one answer, and it appears by itself the day a translation is added. The choice
/// is written to the settings by `WoodSketchApp::save`, which reads it back from here.
fn language_menu(ui: &mut egui::Ui) {
    if woodsketch_i18n::languages().count() < 2 {
        return;
    }
    ui.menu_button(t!("view-language"), |ui| {
        let current = woodsketch_i18n::language();
        for (tag, endonym) in woodsketch_i18n::languages() {
            // Each named in its own language, so a reader looking for theirs recognises it
            // without being able to read the one it would otherwise be listed under.
            if ui.radio(current == tag, endonym).on_hover_text(t!("view-language-hint")).clicked() {
                woodsketch_i18n::set_language(tag);
                ui.close();
            }
        }
    });
}

/// What a theme is called on screen, by the same reasoning as `view_name`.
fn theme_name(theme: egui::ThemePreference) -> String {
    match theme {
        egui::ThemePreference::Light => t!("view-theme-light"),
        egui::ThemePreference::Dark => t!("view-theme-dark"),
        egui::ThemePreference::System => t!("view-theme-system"),
    }
}

impl WoodSketchApp {
    pub(crate) fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                self.file_menu(ui);
                // On the Start page, only File and Help mean anything. The rest act on the
                // drawing underneath, which nobody can see from here: undoing, painting or
                // zooming something out of sight is not a thing anyone meant to do, and a menu
                // whose items appear to do nothing reads as a broken program.
                if self.start.open {
                    self.help_menu(ui);
                    return;
                }
                self.edit_menu(ui);
                self.view_menu(ui);
                self.camera_menu(ui);
                tool_menu(ui, &mut self.tools, &t!("menu-draw"), &[ToolId::Line, ToolId::Arc, ToolId::CenterArc, ToolId::ThreePointArc, ToolId::Pie, ToolId::Rectangle, ToolId::RotatedRectangle, ToolId::Circle, ToolId::Polygon, ToolId::Board]);
                self.tools_menu(ui);
                self.window_menu(ui);
                self.help_menu(ui);
            });
        });
    }

    fn file_menu(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.menu_button(t!("menu-file"), |ui| {
            if ui.button(t!("file-start-page")).on_hover_text(t!("file-start-page-hint")).clicked() {
                ui.close();
                self.show_start_page();
            }
            ui.separator();
            if ui.add(egui::Button::new(t!("file-new")).shortcut_text("Ctrl+N")).clicked() {
                ui.close();
                self.request(&ctx, files::Discard::New);
            }
            if ui.add(egui::Button::new(t!("file-open")).shortcut_text("Ctrl+O")).clicked() {
                ui.close();
                self.request(&ctx, files::Discard::Open);
            }
            ui.add_enabled_ui(!self.recent.is_empty(), |ui| {
                ui.menu_button(t!("file-open-recent"), |ui| {
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
            if ui.add(egui::Button::new(t!("file-save")).shortcut_text("Ctrl+S")).clicked() {
                ui.close();
                self.save(false);
            }
            if ui.add(egui::Button::new(t!("file-save-as")).shortcut_text("Ctrl+Shift+S")).clicked() {
                ui.close();
                self.save(true);
            }
            ui.separator();
            if ui.button(t!("file-import-skp")).on_hover_text(t!("file-import-skp-hint")).clicked() {
                ui.close();
                if let Some(path) = files::import_dialog(self.files.path.as_deref())
                    && let Err(e) = self.import_sketchup(&path)
                {
                    self.files.show_error(e);
                }
            }
            if ui.button(t!("file-print-build-sheets")).on_hover_text(t!("file-print-build-sheets-hint")).clicked() {
                ui.close();
                self.cut_list.export_build_sheets(&self.doc);
            }
            ui.menu_button(t!("file-export"), |ui| {
                // Built here rather than as a const, so each label is a `t!` the catalog test
                // can see, and so the menu is in the language being spoken now.
                let formats = [
                    (t!("file-export-obj"), files::ExportFormat::Obj, t!("file-export-obj-hint")),
                    (t!("file-export-stl"), files::ExportFormat::Stl, t!("file-export-stl-hint")),
                    (t!("file-export-dxf"), files::ExportFormat::Dxf, t!("file-export-dxf-hint")),
                    (t!("file-export-svg"), files::ExportFormat::Svg, t!("file-export-svg-hint")),
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
                ui.label(t!("file-export-units"));
                ui.radio_value(&mut self.export_units, ExportUnits::Inches, t!("file-export-inches"));
                ui.radio_value(&mut self.export_units, ExportUnits::Millimeters, t!("file-export-millimeters"));
            });
            ui.separator();
            if ui.button(t!("file-exit")).clicked() {
                ui.close();
                self.request(&ctx, files::Discard::Close);
            }
        });
    }

    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(t!("menu-edit"), |ui| {
            if ui.add(egui::Button::new(t!("edit-undo")).shortcut_text("Ctrl+Z")).clicked() {
                ui.close();
                self.undo();
            }
            if ui.add(egui::Button::new(t!("edit-redo")).shortcut_text("Ctrl+Y")).clicked() {
                ui.close();
                self.redo();
            }
            ui.separator();
            let selected = !self.doc.selection().is_empty();
            if ui.add_enabled(selected, egui::Button::new(t!("edit-cut")).shortcut_text("Ctrl+X")).clicked() {
                ui.close();
                self.tools.cut(&mut self.doc);
                crate::keys::note_copy(ui.ctx(), &self.tools);
            }
            if ui.add_enabled(selected, egui::Button::new(t!("edit-copy")).shortcut_text("Ctrl+C")).clicked() {
                ui.close();
                self.tools.copy(&self.doc);
                crate::keys::note_copy(ui.ctx(), &self.tools);
            }
            let can_paste = self.tools.can_paste();
            if ui.add_enabled(can_paste, egui::Button::new(t!("edit-paste")).shortcut_text("Ctrl+V")).on_hover_text(t!("edit-paste-hint")).clicked() {
                ui.close();
                self.tools.paste(&mut self.doc);
            }
            if ui.add_enabled(can_paste, egui::Button::new(t!("edit-paste-in-place")).shortcut_text("Ctrl+Shift+V")).clicked() {
                ui.close();
                self.tools.paste_in_place(&mut self.doc);
            }
            if ui.add_enabled(selected, egui::Button::new(t!("edit-duplicate")).shortcut_text("Ctrl+D")).on_hover_text(t!("edit-duplicate-hint")).clicked() {
                ui.close();
                self.tools.duplicate(&mut self.doc);
            }
            ui.separator();
            if ui.add(egui::Button::new(t!("edit-delete")).shortcut_text("Delete")).clicked() {
                ui.close();
                self.tools.cancel();
                self.doc.erase_selection();
            }
            if ui.button(t!("edit-delete-guides")).clicked() {
                ui.close();
                self.tools.delete_guides(&mut self.doc);
            }
            ui.separator();
            if ui.add(egui::Button::new(t!("edit-select-all")).shortcut_text("Ctrl+A")).clicked() {
                ui.close();
                self.doc.select_all();
            }
            if ui.button(t!("edit-select-none")).clicked() {
                ui.close();
                self.doc.selection_mut().clear();
            }
            ui.separator();
            if ui.add(egui::Button::new(t!("edit-make-group")).shortcut_text("Ctrl+G")).clicked() {
                ui.close();
                self.tools.make_group(&mut self.doc, DefinitionKind::Group);
            }
            if ui.add(egui::Button::new(t!("edit-make-component")).shortcut_text("G")).clicked() {
                ui.close();
                self.tools.make_group(&mut self.doc, DefinitionKind::Component);
            }
            if ui.button(t!("edit-explode")).clicked() {
                ui.close();
                self.tools.explode(&mut self.doc);
            }
        });
    }

    fn view_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(t!("menu-view"), |ui| {
            ui.checkbox(&mut self.viewport.show_grid, t!("view-grid"));
            ui.checkbox(&mut self.viewport.show_movement, t!("view-wood-movement")).on_hover_text(t!("view-wood-movement-hint"));
            ui.menu_button(t!("view-theme"), |ui| {
                let current = ui.ctx().options(|o| o.theme_preference);
                for (theme, _) in THEMES {
                    if ui.radio(current == theme, theme_name(theme)).on_hover_text(t!("view-theme-hint")).clicked() {
                        ui.ctx().set_theme(theme);
                        ui.close();
                    }
                }
            });
            ui.menu_button(t!("view-units"), |ui| {
                use woodsketch_doc::units::Units;
                let current = self.doc.units();
                for choice in Units::ALL {
                    if ui.radio(current == choice, choice.name()).on_hover_text(t!("view-units-hint")).clicked() {
                        self.doc.set_units(choice);
                        ui.close();
                    }
                }
            });
            let mut guides = !self.doc.scene().guides_hidden;
            if ui.checkbox(&mut guides, t!("view-guides")).on_hover_text(t!("view-guides-hint")).changed() {
                self.doc.set_guides_hidden(!guides);
            }
            let mut dimensions = !self.doc.scene().dimensions_hidden;
            if ui.checkbox(&mut dimensions, t!("view-dimensions")).on_hover_text(t!("view-dimensions-hint")).changed() {
                self.doc.set_dimensions_hidden(!dimensions);
            }
            let mut text = !self.doc.scene().text_hidden;
            if ui.checkbox(&mut text, t!("view-text")).on_hover_text(t!("view-text-hint")).changed() {
                self.doc.set_text_hidden(!text);
            }
            ui.checkbox(&mut self.show_tray, t!("view-default-tray"));
            language_menu(ui);
        });
    }

    fn camera_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(t!("menu-camera"), |ui| {
            ui.menu_button(t!("camera-standard-views"), |ui| {
                for view in VIEWS {
                    if ui.button(view_name(view)).clicked() {
                        ui.close();
                        self.viewport.camera.set_view(view);
                    }
                }
            });
            ui.separator();
            for tool in [ToolId::Orbit, ToolId::Pan, ToolId::Zoom, ToolId::Walk] {
                tool_item(ui, &mut self.tools, tool);
            }
            if ui.add(egui::Button::new(t!("camera-zoom-extents")).shortcut_text("Shift+Z")).clicked() {
                ui.close();
                self.viewport.request_zoom_extents();
            }
        });
    }

    fn tools_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(t!("menu-tools"), |ui| {
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
            if ui.button(t!("tools-cut-list")).clicked() {
                ui.close();
                self.cut_list.open = true;
            }
        });
    }

    fn window_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(t!("menu-window"), |ui| {
            ui.checkbox(&mut self.show_tray, t!("window-default-tray"));
            ui.checkbox(&mut self.cut_list.open, t!("window-cut-list"));
        });
    }

    fn help_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(t!("menu-help"), |ui| {
            if ui.button(t!("help-check-for-updates")).clicked() {
                ui.close();
                self.check_for_updates();
            }
            ui.checkbox(&mut self.check_updates, t!("help-check-at-startup")).on_hover_text(t!("help-check-at-startup-hint"));
            ui.separator();
            if ui.button(t!("help-whats-new")).on_hover_text(t!("help-whats-new-hint")).clicked() {
                ui.close();
                self.read_the_changes();
            }
            if ui.button(t!("help-release-notes")).on_hover_text(t!("help-release-notes-hint")).clicked() {
                ui.close();
                ui.ctx().open_url(egui::OpenUrl::new_tab(crate::updates::RELEASES_URL));
            }
            ui.separator();
            if ui.button(t!("help-report-problem")).on_hover_text(t!("help-report-problem-hint")).clicked() {
                ui.close();
                self.feedback.start(woodsketch_doc::feedback::Kind::Problem);
            }
            if ui.button(t!("help-send-feedback")).on_hover_text(t!("help-send-feedback-hint")).clicked() {
                ui.close();
                self.feedback.start(woodsketch_doc::feedback::Kind::Idea);
            }
            ui.separator();
            if ui.button(t!("help-problems-and-ideas")).on_hover_text(t!("help-problems-and-ideas-hint")).clicked() {
                ui.close();
                ui.ctx().open_url(egui::OpenUrl::new_tab(crate::updates::ISSUES_URL));
            }
            if ui.button(t!("help-discussions")).on_hover_text(t!("help-discussions-hint")).clicked() {
                ui.close();
                ui.ctx().open_url(egui::OpenUrl::new_tab(crate::updates::DISCUSSIONS_URL));
            }
            ui.separator();
            if ui.button(t!("help-about")).clicked() {
                ui.close();
                self.show_about = true;
            }
        });
    }

    pub(crate) fn about_window(&mut self, ctx: &egui::Context) {
        // `open` holds a borrow of the flag, so What's New is asked for through a local.
        let mut show_about = self.show_about;
        let mut changes = false;
        egui::Window::new(t!("about-title"))
            .open(&mut show_about)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("about-version", "version" => crate::updates::VERSION));
                    changes = ui.link(t!("about-whats-new")).clicked();
                });
                ui.label(t!("about-what-it-is"));
                ui.label(t!("about-not-affiliated"));
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
        egui::Window::new(t!("whats-new-title"))
            .open(&mut open)
            .collapsible(false)
            .default_width(520.0)
            .default_height(440.0)
            .show(ctx, |ui| {
                match (&since, releases.len()) {
                    (Some(from), 1) => ui.label(t!("whats-new-one-since", "from" => from.as_str())),
                    (Some(from), n) => ui.label(t!("whats-new-many-since", "count" => n, "from" => from.as_str())),
                    (None, _) => ui.label(t!("whats-new-recent")),
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
                                ui.label(t!("whats-new-this-one"));
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
                    read = ui.button(t!("whats-new-done")).on_hover_text(t!("whats-new-done-hint")).clicked();
                    ui.hyperlink_to(t!("whats-new-every-version"), crate::updates::RELEASES_URL);
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
                if icon_button(ui, icons::NEW, false, true, &t!("toolbar-new-hint")) {
                    self.request(&ctx, files::Discard::New);
                }
                if icon_button(ui, icons::OPEN, false, true, &t!("toolbar-open-hint")) {
                    self.request(&ctx, files::Discard::Open);
                }
                if icon_button(ui, icons::SAVE, false, true, &t!("toolbar-save-hint")) {
                    self.save(false);
                }
                ui.separator();
                if icon_button(ui, icons::UNDO, false, true, &t!("toolbar-undo-hint")) {
                    self.undo();
                }
                if icon_button(ui, icons::REDO, false, true, &t!("toolbar-redo-hint")) {
                    self.redo();
                }
                ui.separator();
                if icon_button(ui, icons::ZOOM_EXTENTS, false, true, &t!("toolbar-zoom-extents-hint")) {
                    self.viewport.request_zoom_extents();
                }
                for view in VIEWS {
                    let name = view_name(view);
                    let label = if view == StandardView::Iso { format!("{} {name}", icons::ISO) } else { name.clone() };
                    let response = ui
                        .add(egui::Button::selectable(false, label).min_size(egui::vec2(0.0, BUTTON_SIZE)))
                        .on_hover_text(t!("camera-view-hint", "view" => name.as_str()));
                    if response.clicked() {
                        response.surrender_focus();
                        self.viewport.camera.set_view(view);
                    }
                }
                ui.separator();
                let open = self.cut_list.open;
                if icon_button(ui, icons::CUT_LIST, open, true, &t!("toolbar-cut-list-hint")) {
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
                            let hint = if tool.is_available() {
                                t!("tool-hint", "tool" => tool.name(), "key" => tool.shortcut_label())
                            } else {
                                t!("tool-hint-coming-soon", "tool" => tool.name(), "key" => tool.shortcut_label())
                            };
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
                ui.strong(t!("tray-title"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(icons::CLOSE).on_hover_text(t!("tray-hide-hint")).clicked() {
                        self.show_tray = false;
                    }
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                if self.tools.active() == ToolId::Board {
                    tray_section(ui, "Board", &t!("tray-board"), |ui| crate::board::show(ui, &mut self.doc, &mut self.tools));
                }
                tray_section(ui, "Entity Info", &t!("tray-entity-info"), |ui| tags::entity_info(ui, &mut self.doc, &mut self.tools));
                if let Some(pair) = crate::joinery::selected_pair(&self.doc) {
                    tray_section(ui, "Joinery", &t!("tray-joinery"), |ui| crate::joinery::show(ui, &mut self.doc, pair));
                }
                if let [woodsketch_doc::Entity::Instance(part)] = self.doc.selection().iter().collect::<Vec<_>>()[..] {
                    if let Some(made) = self.doc.template_of(part) {
                        tray_section(ui, "Template sizes", &t!("tray-template-sizes", "template" => made.template.name()), |ui| {
                            crate::template::change(ui, &mut self.doc, part, made)
                        });
                    }
                    tray_section(ui, "Edge profile", &t!("tray-edge-profile"), |ui| crate::profile::show(ui, &mut self.doc, part));
                }
                // Folded until wanted: making a new cabinet, drawer, face frame or table.
                egui::CollapsingHeader::new(egui::RichText::new(t!("tray-templates")).strong())
                    .id_salt("Templates")
                    .default_open(false)
                    .show(ui, |ui| {
                        if crate::template::new(ui, &mut self.doc) {
                            self.viewport.request_zoom_extents();
                        }
                    });
                tray_section(ui, "Materials", &t!("tray-materials"), |ui| self.materials.show(ui, &mut self.doc, &mut self.tools));
                tray_section(ui, "Tags", &t!("tray-tags"), |ui| self.tags.show(ui, &mut self.doc));
                tray_section(ui, "Outliner", &t!("tray-outliner"), |ui| outliner::show(ui, &mut self.doc, &mut self.tools));
            });
        });
    }

    /// What the status bar would be saying about an update, for the Start page, which stands in
    /// front of the status bar and so hid it: a drawing opened on a Start page left open would
    /// download an update and never offer the restart, until you happened to leave the page.
    ///
    /// Across the top rather than the bottom, because the page has no bottom bar to sit in and
    /// the restart is the one thing here worth interrupting for.
    pub(crate) fn start_update_banner(&mut self, ui: &mut egui::Ui) {
        let state = self.update_state();
        let busy = matches!(state, Some(crate::updates::UpdateState::Downloading(_) | crate::updates::UpdateState::Ready(_)));
        if !busy && !self.updated() {
            return;
        }
        egui::Panel::top("start update").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                // Just updated: the same note the status bar gives, so arriving on the Start
                // page after an update says as much as arriving in a drawing does.
                if self.updated() {
                    ui.label(format!("{} {}", egui_phosphor::regular::CHECK_CIRCLE, t!("status-updated", "version" => crate::updates::VERSION)));
                    if ui.link(t!("status-whats-new")).on_hover_text(t!("status-whats-new-hint")).clicked() {
                        self.read_the_changes();
                    }
                    if ui.small_button(egui_phosphor::regular::X).on_hover_text(t!("status-dismiss")).clicked() {
                        self.updated_from = None;
                    }
                }
                if let Some(crate::updates::UpdateState::Downloading(version)) = &state {
                    ui.label(t!("status-downloading", "version" => version.as_str()));
                    ui.add(egui::Spinner::new());
                }
                if let Some(crate::updates::UpdateState::Ready(version)) = &state {
                    let label = egui::RichText::new(format!("{} {}", egui_phosphor::regular::ARROW_CIRCLE_UP, t!("status-restart-to-update", "version" => version.as_str())))
                        .color(egui::Color32::WHITE)
                        .strong();
                    let restart = ui
                        .add(egui::Button::new(label).fill(egui::Color32::from_rgb(34, 139, 34)))
                        .on_hover_text(t!("status-restart-to-update-hint"));
                    if restart.clicked() {
                        let ctx = ui.ctx().clone();
                        self.restart_to_update(&ctx);
                    }
                }
            });
            ui.add_space(2.0);
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
                        ui.label(format!("{} {}", egui_phosphor::regular::ARROWS_CLOCKWISE, t!("status-reloaded")))
                            .on_hover_text(t!("status-reloaded-hint"));
                        ui.separator();
                    }
                    // Just updated: say so once, with everything the update brought.
                    if self.updated() {
                        if ui.small_button(egui_phosphor::regular::X).on_hover_text(t!("status-dismiss")).clicked() {
                            self.updated_from = None;
                        }
                        if ui.link(t!("status-whats-new")).on_hover_text(t!("status-whats-new-hint")).clicked() {
                            self.read_the_changes();
                        }
                        ui.label(format!("{} {}", egui_phosphor::regular::CHECK_CIRCLE, t!("status-updated", "version" => crate::updates::VERSION)));
                        ui.separator();
                    }
                    // A new version downloading, then downloaded: offer the restart (green, so it's seen),
                    // never force it.
                    if let Some(crate::updates::UpdateState::Downloading(version)) = self.update_state() {
                        ui.label(t!("status-downloading", "version" => version.as_str()));
                        ui.add(egui::Spinner::new());
                        ui.separator();
                    }
                    if let Some(crate::updates::UpdateState::Ready(version)) = self.update_state() {
                        let label = egui::RichText::new(format!("{} {}", egui_phosphor::regular::ARROW_CIRCLE_UP, t!("status-restart-to-update", "version" => version.as_str()))).color(egui::Color32::WHITE).strong();
                        let restart = ui
                            .add(egui::Button::new(label).fill(egui::Color32::from_rgb(34, 139, 34)))
                            .on_hover_text(t!("status-restart-to-update-hint"));
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

/// One folding panel of the tray. `salt` is what egui remembers the fold by, so it stays the
/// same in every language: keyed on the title, switching language would forget every fold.
fn tray_section(ui: &mut egui::Ui, salt: &str, title: &str, contents: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(egui::RichText::new(title).strong())
        .id_salt(salt)
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
