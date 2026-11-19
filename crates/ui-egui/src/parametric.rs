//! The numbers of a component built from a recipe, in the tray beside a template's sizes: change
//! one and the piece is built again from it (`doc::parametric`, #9).
//!
//! A parameter whose value is a plain number is a box to type in. One worked out from the others
//! (`width - 2*stile`) is shown as what it says rather than as a number, because typing over it
//! would be typing over the rule. Where a parameter lists choices, it is a dropdown.

use eframe::egui;
use woodsketch_doc::recipe::{Length, ParameterSpec, Recipe};
use woodsketch_doc::units::{format_length, parse_length};
use woodsketch_i18n::t;

use woodsketch_doc::{Document, InstanceId};

/// What a parameter looks like on screen.
enum Shown {
    /// A number to type over.
    Typed(String),
    /// One of a list.
    Chosen(String),
    /// Worked out from the others, so it is read and not typed.
    WorkedOut(String),
}

/// The panel for a selected parametric component. Nothing is built until Change size is clicked.
pub fn change(ui: &mut egui::Ui, doc: &mut Document, instance: InstanceId) {
    let Some(made) = doc.parametric_of(instance).cloned() else { return };
    let id = egui::Id::new(("parametric_change", instance));
    // What is being typed, kept between frames, against the component it was read from: if the
    // component is built again or undone under the panel, the boxes start from what it is now.
    // Anything else (counting the parameters, say) throws away the very edit being made.
    let (built, mut recipe) = ui.data(|d| d.get_temp::<(u64, Recipe)>(id)).unwrap_or((made.as_built, made.recipe.clone()));
    if built != made.as_built {
        recipe = made.recipe.clone();
    }
    let mut good = true;
    egui::Grid::new(("parametric_grid", instance)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        for i in 0..recipe.parameters.len() {
            let label = recipe.parameters[i].label.clone().unwrap_or_else(|| recipe.parameters[i].name.clone());
            ui.label(&label);
            let how = if recipe.parameters[i].choices.is_empty() {
                shown(&recipe.parameters[i].value)
            } else {
                Shown::Chosen(said(&recipe.parameters[i].value))
            };
            match how {
                Shown::WorkedOut(text) => {
                    ui.weak(t!("component-worked-out", "sum" => text.as_str()));
                }
                Shown::Chosen(now) => {
                    let choices: Vec<String> = recipe.parameters[i].choices.iter().map(said).collect();
                    let mut picked = now;
                    egui::ComboBox::from_id_salt(("parametric_choice", instance, i)).selected_text(&picked).show_ui(ui, |ui| {
                        for choice in &choices {
                            ui.selectable_value(&mut picked, choice.clone(), choice);
                        }
                    });
                    recipe.parameters[i].value = Length::Text(picked);
                }
                Shown::Typed(now) => {
                    let field = egui::Id::new(("parametric_field", instance, i));
                    let mut text = ui.data(|d| d.get_temp::<String>(field)).unwrap_or(now);
                    ui.add(egui::TextEdit::singleline(&mut text).desired_width(90.0));
                    match parse_length(&text) {
                        Ok(_) => recipe.parameters[i].value = Length::Text(text.clone()),
                        Err(_) => good = false,
                    }
                    ui.data_mut(|d| d.insert_temp(field, text));
                }
            }
            ui.end_row();
        }
    });
    add_parameter(ui, instance, &mut recipe);
    numbers(ui, instance, &mut recipe);
    // One design in several places: changing it here gives this one a design of its own, which
    // is what the cut list needs anyway, but it is better said than discovered.
    let copies = doc.parametric_copies(instance);
    if copies > 1 {
        ui.weak(t!("component-copies", "count" => copies));
    }
    let by_hand = doc.parametric_changed_by_hand(instance);
    if by_hand {
        ui.colored_label(ui.visuals().warn_fg_color, t!("component-changed-by-hand"));
    }
    let changed = recipe.parameters != made.recipe.parameters;
    let label = if by_hand { t!("component-change-anyway") } else { t!("template-change-size") };
    let button = ui.add_enabled(good && changed, egui::Button::new(label));
    if button.on_hover_text(t!("template-change-size-hint")).clicked() {
        match doc.rebuild_parametric(instance, &recipe, by_hand) {
            Ok(_) => {
                ui.data_mut(|d| d.remove::<(u64, Recipe)>(id));
                for i in 0..recipe.parameters.len() {
                    ui.data_mut(|d| d.remove::<String>(egui::Id::new(("parametric_field", instance, i))));
                }
            }
            Err(why) => {
                ui.colored_label(ui.visuals().error_fg_color, why);
            }
        }
        return;
    }
    ui.data_mut(|d| d.insert_temp(id, (made.as_built, recipe)));
}

/// How a parameter's value is read and written: a plain length is typed, a list is chosen from,
/// and arithmetic is left as the rule it is.
fn shown(value: &Length) -> Shown {
    match value {
        Length::Number(_) => Shown::Typed(said(value)),
        Length::Text(text) => match parse_length(text) {
            Ok(_) => Shown::Typed(text.clone()),
            Err(_) => Shown::WorkedOut(text.clone()),
        },
    }
}

/// A value as a person would write it, to fill a box or name a choice with.
fn said(value: &Length) -> String {
    match value {
        Length::Number(n) => format_length(*n),
        Length::Text(text) => text.clone(),
    }
}

/// Entity Info's offer for a group nobody has given numbers to: read it back as a recipe, so its
/// sizes can be named and changed together from then on (#9, and `doc::parametric`).
pub fn offer(ui: &mut egui::Ui, doc: &mut Document, instance: InstanceId) {
    if doc.parametric_of(instance).is_some() || doc.template_of(instance).is_some() {
        return;
    }
    let notes = egui::Id::new(("parametric_notes", instance));
    let button = ui.button(t!("component-make"));
    if button.on_hover_text(t!("component-make-hint")).clicked() {
        let said = match doc.make_parametric(instance) {
            Ok(said) => said,
            Err(why) => vec![why],
        };
        ui.data_mut(|d| d.insert_temp(notes, said));
    }
    // What the reader couldn't describe, kept until something else is selected: building it again
    // is the whole point, so a piece that wouldn't come back the same has to be said out loud.
    let said = ui.data(|d| d.get_temp::<Vec<String>>(notes)).unwrap_or_default();
    if !said.is_empty() {
        ui.colored_label(ui.visuals().warn_fg_color, t!("component-made-notes", "notes" => said.join(" ")));
    }
}

/// The numbers a component is made of: every part's size and corner, each a box that takes a
/// number or a sum over the parameters. This is how a number drawn by hand becomes `width`.
fn numbers(ui: &mut egui::Ui, instance: InstanceId, recipe: &mut Recipe) {
    let id = egui::Id::new(("parametric_numbers", instance));
    egui::CollapsingHeader::new(t!("component-numbers")).id_salt(id).show(ui, |ui| {
        egui::Grid::new(("parametric_numbers_grid", instance)).num_columns(4).spacing([6.0, 4.0]).show(ui, |ui| {
            ui.label("");
            ui.weak(t!("component-along"));
            ui.weak(t!("component-across"));
            ui.weak(t!("component-thick"));
            ui.end_row();
            for p in 0..recipe.parts.len() {
                let name = recipe.parts[p].name.clone();
                ui.label(&name).on_hover_text(t!("component-size-hint"));
                for k in 0..3 {
                    let value = &mut recipe.parts[p].size[k];
                    box_for(ui, ("parametric_size", instance, p, k), value);
                }
                ui.end_row();
                ui.weak(t!("component-corner"));
                for k in 0..3 {
                    let value = &mut recipe.parts[p].at[k];
                    box_for(ui, ("parametric_at", instance, p, k), value);
                }
                ui.end_row();
            }
        });
    });
}

/// One number, as typed. Whatever is in it goes back into the recipe as written, so `3/4"` stays
/// a size and `width - 2*stile` stays a sum; a sum that can't be worked out is reported by the
/// build, which names the part and the field it was in.
fn box_for(ui: &mut egui::Ui, key: (&str, InstanceId, usize, usize), value: &mut Length) {
    let id = egui::Id::new(key);
    let mut text = ui.data(|d| d.get_temp::<String>(id)).unwrap_or_else(|| said(value));
    ui.add(egui::TextEdit::singleline(&mut text).desired_width(80.0));
    if !text.trim().is_empty() {
        *value = Length::Text(text.clone());
    }
    ui.data_mut(|d| d.insert_temp(id, text));
}

/// A row for naming a new number, which is the one thing a drawn component always needs.
fn add_parameter(ui: &mut egui::Ui, instance: InstanceId, recipe: &mut Recipe) {
    let name_id = egui::Id::new(("parametric_new_name", instance));
    let value_id = egui::Id::new(("parametric_new_value", instance));
    let mut name = ui.data(|d| d.get_temp::<String>(name_id)).unwrap_or_default();
    let mut value = ui.data(|d| d.get_temp::<String>(value_id)).unwrap_or_default();
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut name).desired_width(70.0).hint_text(t!("component-new-name")));
        ui.add(egui::TextEdit::singleline(&mut value).desired_width(60.0).hint_text(t!("component-new-value")));
        let good = !name.trim().is_empty() && !value.trim().is_empty() && !recipe.parameters.iter().any(|p| p.name == name.trim());
        if ui.add_enabled(good, egui::Button::new(t!("component-add"))).clicked() {
            recipe.parameters.push(ParameterSpec { name: name.trim().to_owned(), value: Length::Text(value.trim().to_owned()), label: None, choices: Vec::new() });
            name.clear();
            value.clear();
        }
    });
    ui.data_mut(|d| {
        d.insert_temp(name_id, name);
        d.insert_temp(value_id, value);
    });
}
