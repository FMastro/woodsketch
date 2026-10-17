//! The Tags panel (SketchUp's renamed layers) and the Entity Info tag picker.

use eframe::egui;
use egui_phosphor::regular as ph;
use woodsketch_doc::{Document, TagId};
use woodsketch_tools::Tools;

#[derive(Default)]
pub struct TagsPanel {
    /// The tag being renamed, and the text typed so far.
    renaming: Option<(TagId, String)>,
    /// Whether the rename box got its initial focus.
    focused: bool,
    /// Folders folded shut.
    closed: std::collections::HashSet<TagId>,
}

/// What a row asked for, done after drawing the tree.
enum Change {
    Visible(TagId, bool),
    Rename(TagId, String),
    Delete(TagId),
    Move(TagId, Option<TagId>),
}

impl TagsPanel {
    pub fn show(&mut self, ui: &mut egui::Ui, doc: &mut Document) {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let start_renaming = |this: &mut Self, doc: &Document, tag: TagId| {
                    let name = doc.scene().tag(tag).map(|t| t.name.clone()).unwrap_or_default();
                    this.renaming = Some((tag, name));
                    this.focused = false;
                };
                if ui.button(ph::PLUS).on_hover_text("Add a tag").clicked() {
                    let tag = doc.add_tag();
                    start_renaming(self, doc, tag);
                }
                if ui.button(ph::FOLDER_PLUS).on_hover_text("Add a folder (right-click a tag to move it into one)").clicked() {
                    let folder = doc.add_tag_folder();
                    start_renaming(self, doc, folder);
                }
            });
        });

        // Untagged is always there and always visible, as in SketchUp. A tag dropped on it goes to
        // the top level.
        let mut changes = Vec::new();
        let untagged = ui.horizontal(|ui| {
            ui.add_enabled(false, egui::Button::new(ph::EYE).frame(false));
            ui.label("Untagged");
        });
        drop_target(ui, &untagged.response, None, &mut changes);

        self.rows(ui, doc, None, 0, true, &mut changes);
        for change in changes {
            match change {
                Change::Visible(tag, visible) => doc.set_tag_visible(tag, visible),
                Change::Rename(tag, name) => doc.rename_tag(tag, &name),
                Change::Delete(tag) => doc.delete_tag(tag),
                Change::Move(tag, folder) => {
                    doc.move_tag(tag, folder);
                    if let Some(f) = folder {
                        self.closed.remove(&f);
                    }
                }
            }
        }
    }

    /// The tags and folders in `folder`: folders first, then tags, each by name. `shown` is
    /// whether the folders around them are visible (hidden ones dim what's inside).
    fn rows(&mut self, ui: &mut egui::Ui, doc: &Document, folder: Option<TagId>, depth: usize, shown: bool, changes: &mut Vec<Change>) {
        let scene = doc.scene();
        let mut here: Vec<(TagId, String, bool, bool)> =
            scene.tags().filter(|(_, t)| t.parent == folder).map(|(id, t)| (id, t.name.clone(), t.visible, t.folder)).collect();
        here.sort_by_key(|(_, name, _, is_folder)| (!is_folder, name.to_lowercase()));
        for (id, name, visible, is_folder) in here {
            // The whole row drags onto a folder to move it there: sensed under the row's buttons and
            // name, so they still click.
            let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
            let handle = ui.interact(egui::Rect::from_min_size(ui.cursor().min, size), egui::Id::new(("tag_row_drag", id)), egui::Sense::drag());
            handle.dnd_set_drag_payload(Dragged(id));
            if handle.dragged() {
                dragging_label(ui, &name);
            }
            // Whether the pointer is on one of the row's buttons (they click; the rest of the row drags).
            let mut on_button = false;
            let row = ui.horizontal(|ui| {
                ui.add_space(16.0 * depth as f32);
                if is_folder {
                    let open = !self.closed.contains(&id);
                    let caret = if open { ph::CARET_DOWN } else { ph::CARET_RIGHT };
                    let fold = ui.add(egui::Button::new(caret).frame(false));
                    on_button |= fold.hovered();
                    if fold.on_hover_text(if open { "Fold" } else { "Unfold" }).clicked() {
                        if open {
                            self.closed.insert(id);
                        } else {
                            self.closed.remove(&id);
                        }
                    }
                }
                let eye = if visible { ph::EYE } else { ph::EYE_SLASH };
                let what = if is_folder { "everything in this folder" } else { "everything with this tag" };
                let hint = if visible { format!("Hide {what}") } else { "Show it again".to_owned() };
                let eye = ui.add(egui::Button::new(eye).frame(false));
                on_button |= eye.hovered();
                if eye.on_hover_text(hint).clicked() {
                    changes.push(Change::Visible(id, !visible));
                }

                match &mut self.renaming {
                    Some((renaming, text)) if *renaming == id => {
                        let edit = ui.add(egui::TextEdit::singleline(text).desired_width(110.0));
                        if !self.focused {
                            edit.request_focus();
                            self.focused = true;
                        }
                        let cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));
                        let done = edit.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if cancel || done {
                            if !cancel {
                                changes.push(Change::Rename(id, text.clone()));
                            }
                            // Hand the keyboard back to the drawing.
                            edit.surrender_focus();
                            self.renaming = None;
                        }
                    }
                    _ => {
                        let text = if is_folder { format!("{} {name}", ph::FOLDER) } else { name.clone() };
                        let label = if visible && shown { egui::RichText::new(text) } else { egui::RichText::new(text).weak() };
                        let response = ui.add(egui::Label::new(label).selectable(false).sense(egui::Sense::click()));
                        let response = response.on_hover_text("Double-click to rename; drag the row onto a folder (or right-click) to move it");
                        if response.double_clicked() {
                            self.renaming = Some((id, name.clone()));
                            self.focused = false;
                        }
                        response.context_menu(|ui| move_menu(ui, scene, id, changes));
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let hint = if is_folder { "Delete folder (what's in it moves up)" } else { "Delete tag (its groups become Untagged)" };
                    let trash = ui.add(egui::Button::new(ph::TRASH).frame(false));
                    on_button |= trash.hovered();
                    if trash.on_hover_text(hint).clicked() {
                        changes.push(Change::Delete(id));
                    }
                });
            });
            // An open hand over the row (not its buttons): it can be picked up.
            if ui.rect_contains_pointer(handle.rect) && !handle.dragged() && !on_button && self.renaming.is_none() && ui.ctx().dragged_id().is_none() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
            // Dropped on a folder: into it. On a tag: beside it, in its folder.
            drop_target(ui, &row.response, if is_folder { Some(id) } else { folder }, changes);
            if is_folder && !self.closed.contains(&id) {
                self.rows(ui, doc, Some(id), depth + 1, shown && visible, changes);
            }
        }
    }
}

/// Right-click on a tag or folder: move it into a folder, or out to the top.
fn move_menu(ui: &mut egui::Ui, scene: &woodsketch_doc::Scene, tag: TagId, changes: &mut Vec<Change>) {
    let current = scene.tag(tag).and_then(|t| t.parent);
    // Any folder but itself or one inside it.
    let mut folders: Vec<(TagId, String)> = scene
        .tags()
        .filter(|(id, t)| t.folder && *id != tag && scene.tag_ancestors(*id).all(|a| a != tag))
        .map(|(id, _)| (id, scene.tag_path(id)))
        .collect();
    folders.sort_by_key(|(_, path)| path.to_lowercase());
    ui.label(egui::RichText::new("Move to").weak());
    if current.is_some() && ui.button("Top level").clicked() {
        changes.push(Change::Move(tag, None));
        ui.close();
    }
    for (folder, path) in folders {
        if Some(folder) != current && ui.button(format!("{} {path}", ph::FOLDER)).clicked() {
            changes.push(Change::Move(tag, Some(folder)));
            ui.close();
        }
    }
    if scene.tags().all(|(id, t)| !t.folder || id == tag) {
        ui.weak("No folders yet: add one with the folder button.");
    }
}

/// SketchUp's Entity Info, for now just the tag of the selected groups and components.
pub fn entity_info(ui: &mut egui::Ui, doc: &mut Document, tools: &mut Tools) {
    let count = doc.selection().len();
    if count == 0 {
        ui.weak("Nothing selected.");
        return;
    }
    let instances = doc.selection().iter().filter(|e| matches!(e, woodsketch_doc::Entity::Instance(_))).count();
    let single = doc.selection().iter().find_map(|e| if let woodsketch_doc::Entity::Instance(i) = e { Some(i) } else { None });
    match (count, instances, single) {
        (1, 1, Some(instance)) => name_field(ui, doc, instance),
        (n, 0, _) => {
            ui.label(format!("{n} loose entities"));
        }
        (n, i, _) => {
            ui.label(format!("{n} selected ({i} groups/components)"));
        }
    }
    crate::materials::entity_info_material(ui, doc, tools);
    // One group or component: its part details (size, stock, grain, cut).
    if let (1, 1, Some(instance)) = (count, instances, single) {
        crate::part::show(ui, doc, instance);
        crate::joinery::part_joints(ui, doc, instance);
        crate::profile::part_profiles(ui, doc, instance);
    }
    if instances == 0 {
        ui.weak("Tags go on groups and components. Group this first (Ctrl+G).");
        return;
    }

    // Tags with their folders (`Deck / Joist`); folders themselves can't be put on anything.
    let name_of = |doc: &Document, tag: Option<TagId>| match tag.filter(|t| doc.scene().tag(*t).is_some()) {
        Some(t) => doc.scene().tag_path(t),
        None => "Untagged".to_owned(),
    };
    let current = doc.selection_tag();
    let shown = match current {
        Some(tag) => name_of(doc, tag),
        None => "(mixed)".to_owned(),
    };
    let mut choices: Vec<(Option<TagId>, String)> = vec![(None, "Untagged".to_owned())];
    let mut tags: Vec<(Option<TagId>, String)> =
        doc.scene().tags().filter(|(_, t)| !t.folder).map(|(id, _)| (Some(id), doc.scene().tag_path(id))).collect();
    tags.sort_by_key(|(_, path)| path.to_lowercase());
    choices.extend(tags);

    let mut picked = None;
    ui.horizontal(|ui| {
        ui.label(format!("{} Tag", ph::TAG));
        egui::ComboBox::from_id_salt("entity_info_tag").selected_text(shown).show_ui(ui, |ui| {
            for (tag, name) in &choices {
                if ui.selectable_label(current == Some(*tag), name).clicked() {
                    picked = Some(*tag);
                }
            }
        });
    });
    if let Some(tag) = picked {
        tools.cancel();
        doc.tag_selection(tag);
    }
}

/// The selected group's or component's name, editable: it names the part in the cut list.
fn name_field(ui: &mut egui::Ui, doc: &mut Document, instance: woodsketch_doc::InstanceId) {
    let key = egui::Id::new(("entity_info_name", instance));
    let current = doc.instance_name(instance).unwrap_or_default().to_owned();
    // While typing, keep what's typed (even blank); otherwise show the drawing's name.
    let mut text = ui.data(|d| d.get_temp::<String>(key)).unwrap_or(current);
    ui.horizontal(|ui| {
        ui.label("Name");
        let response = ui.add(egui::TextEdit::singleline(&mut text).desired_width(150.0));
        if response.changed() {
            doc.rename_instance(instance, &text);
        }
        if response.has_focus() {
            ui.data_mut(|d| d.insert_temp(key, text));
        } else {
            ui.data_mut(|d| d.remove::<String>(key));
        }
    });
}

/// A tag being dragged in the Tags panel.
#[derive(Clone, Copy)]
struct Dragged(TagId);

/// A row a dragged tag can be dropped on, moving it into `folder` (`None`: the top level). The row
/// is outlined while a tag is held over it.
fn drop_target(ui: &egui::Ui, row: &egui::Response, folder: Option<TagId>, changes: &mut Vec<Change>) {
    let Some(dragged) = row.dnd_hover_payload::<Dragged>() else { return };
    if Some(dragged.0) == folder {
        return;
    }
    let stroke = egui::Stroke::new(1.5, ui.visuals().selection.stroke.color);
    ui.painter().rect_stroke(row.rect.expand(1.0), 3.0, stroke, egui::StrokeKind::Outside);
    if let Some(dragged) = row.dnd_release_payload::<Dragged>() {
        changes.push(Change::Move(dragged.0, folder));
    }
}

/// The dragged tag's name, following the pointer.
fn dragging_label(ui: &egui::Ui, name: &str) {
    let Some(pointer) = ui.ctx().pointer_interact_pos() else { return };
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    egui::Area::new(egui::Id::new("dragged_tag")).order(egui::Order::Tooltip).fixed_pos(pointer + egui::vec2(12.0, 4.0)).show(ui.ctx(), |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.label(format!("{} {name}", ph::TAG));
        });
    });
}
