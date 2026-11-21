//! The Default Tray down the right-hand side, as SketchUp lays it out: a column of panels, each
//! with a header bar naming it that folds when clicked and closes with its ✕, all of them listed
//! in Window > Default Tray so a closed one can be brought back.
//!
//! Which panels stand there is kept between runs ([`Tray`], in eframe's settings). Three answers
//! per panel rather than two, because some panels are about what's selected and used to appear by
//! themselves when it applied: [`Shows::Auto`] keeps that, and the Window menu says
//! [`Shows::Always`] or [`Shows::Never`] when someone means it.

use std::collections::BTreeMap;

use eframe::egui;
use serde::{Deserialize, Serialize};
use woodsketch_i18n::t;

use crate::icons;

/// A panel of the tray, in the order they stand in it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum Panel {
    Board,
    EntityInfo,
    Joinery,
    TemplateSizes,
    EdgeProfile,
    Templates,
    Materials,
    Tags,
    Outliner,
}

impl Panel {
    /// Every panel, top to bottom: the tray draws them in this order and the Window menu lists
    /// them in it.
    pub const ALL: [Self; 9] =
        [Self::Board, Self::EntityInfo, Self::Joinery, Self::TemplateSizes, Self::EdgeProfile, Self::Templates, Self::Materials, Self::Tags, Self::Outliner];

    /// What egui remembers the fold by. English and never translated: egui keys a fold by its id,
    /// so a translated one would forget whether the panel was folded on switching language.
    fn salt(self) -> &'static str {
        match self {
            Self::Board => "Board",
            Self::EntityInfo => "Entity Info",
            Self::Joinery => "Joinery",
            Self::TemplateSizes => "Template sizes",
            Self::EdgeProfile => "Edge profile",
            Self::Templates => "Templates",
            Self::Materials => "Materials",
            Self::Tags => "Tags",
            Self::Outliner => "Outliner",
        }
    }

    /// Its name, for the header bar and the Window menu. A match rather than a key built from the
    /// panel, because `t!` keys have to be literals for `i18n`'s guard test to find them.
    pub fn name(self) -> String {
        match self {
            Self::Board => t!("tray-board"),
            Self::EntityInfo => t!("tray-entity-info"),
            Self::Joinery => t!("tray-joinery"),
            Self::TemplateSizes => t!("tray-template-sizes-name"),
            Self::EdgeProfile => t!("tray-edge-profile"),
            Self::Templates => t!("tray-templates"),
            Self::Materials => t!("tray-materials"),
            Self::Tags => t!("tray-tags"),
            Self::Outliner => t!("tray-outliner"),
        }
    }

    /// What to select for a panel that works on a selection, said in its body while there's
    /// nothing for it to work on. `None` for the panels that always have something to show --
    /// Board among them: it picks the stock the Board tool will use, which is worth seeing before
    /// reaching for the tool as well as after.
    pub fn needs(self) -> Option<String> {
        match self {
            Self::Joinery => Some(t!("tray-joinery-needs")),
            Self::TemplateSizes => Some(t!("tray-template-sizes-needs")),
            Self::EdgeProfile => Some(t!("tray-edge-profile-needs")),
            _ => None,
        }
    }

    /// Whether it starts folded. Templates does: it's for making a new cabinet or table, which is
    /// a thing you go looking for, not something to read while working on what's there.
    fn folded(self) -> bool {
        matches!(self, Self::Templates)
    }
}

/// Whether a panel stands in the tray.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum Shows {
    /// When it has something to say: Joinery with two parts selected, Board with the Board tool
    /// in hand. The panels that always have something to say always show.
    #[default]
    Auto,
    Always,
    Never,
}

/// Which panels stand in the tray, kept between runs.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Tray {
    /// Only the panels somebody has said something about; the rest are [`Shows::Auto`].
    said: BTreeMap<Panel, Shows>,
}

impl Tray {
    pub fn shows(&self, panel: Panel) -> Shows {
        self.said.get(&panel).copied().unwrap_or_default()
    }

    /// Whether to draw it now. `applies` is whether it has anything to say (see [`Shows::Auto`]).
    pub fn showing(&self, panel: Panel, applies: bool) -> bool {
        match self.shows(panel) {
            Shows::Auto => applies,
            Shows::Always => true,
            Shows::Never => false,
        }
    }

    pub fn set(&mut self, panel: Panel, shows: Shows) {
        self.said.insert(panel, shows);
    }

    /// Window > Default Tray's Show All / Hide All / Automatic.
    pub fn set_all(&mut self, shows: Shows) {
        if shows == Shows::Auto {
            self.said.clear();
            return;
        }
        for panel in Panel::ALL {
            self.set(panel, shows);
        }
    }
}

/// One panel in the tray: a header bar with its name, folded by clicking the name or the arrow,
/// closed by the ✕. Returns whether the ✕ was clicked, since what to do about that belongs to
/// whoever holds the [`Tray`].
#[must_use = "a panel closed by its ✕ has to be written down, or it comes back next frame"]
pub fn section(ui: &mut egui::Ui, panel: Panel, title: &str, body: impl FnOnce(&mut egui::Ui)) -> bool {
    let id = ui.make_persistent_id(panel.salt());
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, !panel.folded());
    let mut close = false;
    egui::Frame::new()
        .fill(crate::chrome::panel_header_fill(ui.visuals()))
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(4, 2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                state.show_toggle_button(ui, egui::collapsing_header::paint_default_icon);
                let name = egui::Label::new(egui::RichText::new(title).strong()).sense(egui::Sense::click());
                if ui.add(name).clicked() {
                    state.toggle(ui);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Named after its panel: nine buttons all called "✕" would tell a screen
                    // reader (and a test looking for one of them) nothing.
                    let shut = ui.small_button(icons::CLOSE);
                    let named = t!("tray-close-panel", "panel" => title);
                    shut.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &named));
                    if shut.on_hover_text(t!("tray-close-panel-hint")).clicked() {
                        close = true;
                    }
                    // And a way into the guide, for the panels that have a page. Named after
                    // its panel for the same reason the ✕ is: nine of them all called "?"
                    // would tell a screen reader nothing.
                    if let Some(page) = crate::help::for_panel(panel) {
                        let ask = ui.small_button(icons::HELP);
                        let named = t!("tray-panel-help", "panel" => title);
                        ask.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &named));
                        if ask.on_hover_text(t!("tray-panel-help-hint")).clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(page));
                        }
                    }
                });
            });
        });
    // The body sits a little in from its header, so a long column reads as panels with contents
    // rather than as one list of everything.
    state.show_body_unindented(ui, |ui| {
        egui::Frame::new().inner_margin(egui::Margin { left: 6, right: 2, top: 3, bottom: 3 }).show(ui, |ui| {
            ui.set_width(ui.available_width());
            body(ui);
        });
    });
    ui.add_space(4.0);
    close
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nothing said: the panels that always have something to say stand there, and the ones about
    /// a selection appear when it applies -- which is how the tray behaved before it could be
    /// closed panel by panel.
    #[test]
    fn panels_show_by_themselves_until_somebody_says_otherwise() {
        let tray = Tray::default();
        assert!(tray.showing(Panel::Outliner, true), "the Outliner always has something to say");
        assert!(!tray.showing(Panel::Joinery, false), "Joinery waits for two parts");
        assert!(tray.showing(Panel::Joinery, true), "and appears when they're selected");
    }

    /// Closing one panel says nothing about the rest, and the Window menu brings that one back.
    #[test]
    fn closing_a_panel_keeps_it_closed_until_it_is_asked_back() {
        let mut tray = Tray::default();
        tray.set(Panel::Materials, Shows::Never);
        assert!(!tray.showing(Panel::Materials, true));
        assert!(tray.showing(Panel::Tags, true), "the others are untouched");
        tray.set(Panel::Materials, Shows::Always);
        assert!(tray.showing(Panel::Materials, true));
    }

    /// A panel asked for stands there even with nothing selected, so the Window menu's tick is
    /// never a lie: that's what the body's "select two parts" line is for.
    #[test]
    fn a_panel_asked_for_shows_even_with_nothing_to_say() {
        let mut tray = Tray::default();
        tray.set(Panel::Joinery, Shows::Always);
        assert!(tray.showing(Panel::Joinery, false));
        assert!(Panel::Joinery.needs().is_some(), "and says what to select");
    }

    /// Show All, Hide All, and Automatic back to where everyone starts.
    #[test]
    fn show_all_hide_all_and_back_to_automatic() {
        let mut tray = Tray::default();
        tray.set_all(Shows::Never);
        assert!(Panel::ALL.iter().all(|&p| !tray.showing(p, true)), "all closed");
        tray.set_all(Shows::Always);
        assert!(Panel::ALL.iter().all(|&p| tray.showing(p, false)), "all open, selection or not");
        tray.set_all(Shows::Auto);
        assert!(tray.showing(Panel::Tags, true) && !tray.showing(Panel::EdgeProfile, false), "automatic again");
    }

    /// Every panel is in `ALL` once, since the Window menu and the tray are both built from it.
    #[test]
    fn every_panel_is_listed_once() {
        let mut all = Panel::ALL.to_vec();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), Panel::ALL.len());
    }
}
