//! Help > Report a Problem / Send Feedback: a short form, the details that go with it shown in
//! full, and a button that opens a new GitHub issue filled in (`woodsketch_doc::feedback`). Nothing
//! is sent from the app; the user reads it over and submits it on GitHub.

use eframe::egui;
use woodsketch_doc::Document;
use woodsketch_doc::feedback::{Drawing, Kind, System, details, issue_url, scrub};

/// Where issues go.
pub const REPO: &str = "https://github.com/FMastro/woodsketch";

#[derive(Default)]
pub struct FeedbackWindow {
    pub open: bool,
    kind: Kind,
    title: String,
    what: String,
    /// Counts from the open drawing go with it (on by default; no names).
    skip_drawing: bool,
    /// Focus the title when the form opens.
    focus_title: bool,
}

impl FeedbackWindow {
    pub fn start(&mut self, kind: Kind) {
        *self = Self { open: true, kind, focus_title: true, ..Self::default() };
    }

    /// The form. `graphics` is the card and driver; `last_error` the last error shown.
    pub fn show(&mut self, ctx: &egui::Context, doc: &Document, graphics: &str, last_error: Option<&str>) {
        if !self.open {
            return;
        }
        let mut open = true;
        let mut sent = false;
        let heading = match self.kind {
            Kind::Problem => "Report a Problem",
            Kind::Idea => "Send Feedback",
        };
        egui::Window::new(heading).open(&mut open).collapsible(false).resizable(false).default_width(460.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.kind, Kind::Problem, "Something went wrong");
                ui.selectable_value(&mut self.kind, Kind::Idea, "An idea or other feedback");
            });
            ui.add_space(4.0);
            ui.label("Title");
            // Tab and Shift+Tab move between the title and the description (left to egui, Tab
            // would go to the toolbar behind the window). Taken before the fields see it.
            let (title_id, what_id) = (ui.make_persistent_id("feedback_title"), ui.make_persistent_id("feedback_what"));
            let focused = |id| ui.memory(|m| m.has_focus(id));
            let (in_title, in_what) = (focused(title_id), focused(what_id));
            if (in_title || in_what) && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab) || i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab)) {
                ui.memory_mut(|m| m.request_focus(if in_title { what_id } else { title_id }));
            }
            let title = ui.add(egui::TextEdit::singleline(&mut self.title).id(title_id).lock_focus(true).desired_width(f32::INFINITY).hint_text("A few words"));
            if std::mem::take(&mut self.focus_title) {
                title.request_focus();
            }
            // Enter in the title goes on to the description.
            if title.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                ui.memory_mut(|m| m.request_focus(what_id));
            }
            ui.label(match self.kind {
                Kind::Problem => "What happened? (what you did, what you expected)",
                Kind::Idea => "Your idea",
            });
            ui.add(egui::TextEdit::multiline(&mut self.what).id(what_id).lock_focus(true).desired_width(f32::INFINITY).desired_rows(5));
            let mut with_drawing = !self.skip_drawing;
            ui.checkbox(&mut with_drawing, "Include counts from the open drawing (no names)")
                .on_hover_text("How many parts, faces, joints and materials of each kind: helps reproduce a problem");
            self.skip_drawing = !with_drawing;

            // Everything that goes with it, as it'll be sent.
            let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok();
            let user = std::env::var("USERNAME").or_else(|_| std::env::var("USER")).ok();
            let system = System::here(crate::updates::VERSION, graphics, woodsketch_doc::units::units().name());
            let drawing = with_drawing.then(|| Drawing::of(doc.scene()));
            let error = last_error.filter(|_| self.kind == Kind::Problem).map(|e| scrub(e, home.as_deref(), user.as_deref()));
            let text = details(&system, drawing.as_ref(), error.as_deref());
            ui.add_space(4.0);
            ui.label("Sent with it:");
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.add(egui::Label::new(egui::RichText::new(&text).monospace().small()).wrap());
            });
            ui.weak("It opens on GitHub (a free account is needed), where you can add more, attach a drawing if you like, and submit. Issues there are public. Nothing is sent until you submit.");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let ready = !self.what.trim().is_empty();
                if ui.add_enabled(ready, egui::Button::new("Open on GitHub")).clicked() {
                    let title = scrub(self.title.trim(), home.as_deref(), user.as_deref());
                    let what = scrub(self.what.trim(), home.as_deref(), user.as_deref());
                    ui.ctx().open_url(egui::OpenUrl::new_tab(issue_url(REPO, self.kind, &title, &what, &text)));
                    sent = true;
                }
                if ui.button("Cancel").clicked() {
                    self.open = false;
                }
            });
        });
        if !open || sent {
            *self = Self::default();
        }
    }
}
