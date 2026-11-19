//! The numbers of a component built from a recipe, in the tray beside a template's sizes: change
//! one and the piece is built again from it (`doc::parametric`, #9).
//!
//! A parameter whose value is a plain number is a box to type in. One worked out from the others
//! (`width - 2*stile`) is shown as what it says rather than as a number, because typing over it
//! would be typing over the rule. Where a parameter lists choices, it is a dropdown.

use eframe::egui;
use woodsketch_doc::recipe::{Length, Recipe};
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
    let mut recipe = ui.data(|d| d.get_temp::<Recipe>(id)).unwrap_or_else(|| made.recipe.clone());
    if recipe.parameters.len() != made.recipe.parameters.len() {
        // The drawing moved on under the panel (undo, or another component picked).
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
                ui.data_mut(|d| d.remove::<Recipe>(id));
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
    ui.data_mut(|d| d.insert_temp(id, recipe));
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
