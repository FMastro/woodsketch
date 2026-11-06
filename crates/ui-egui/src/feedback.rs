//! Help > Report a Problem / Send Feedback: a short form, the details that go with it shown in
//! full, then Send (woodsketch.net makes the GitHub issue, no account needed; an email left for
//! questions goes privately to the developer) or Open on GitHub (submit it there). See
//! `woodsketch_doc::feedback`.

use std::sync::{Arc, Mutex};

use eframe::egui;
use woodsketch_i18n::t;
use woodsketch_doc::Document;
use woodsketch_doc::feedback::{Crash, Drawing, Kind, RELAY, Sent, System, details, issue_url, reply, request, scrub, valid_email};

/// Where issues go.
pub const REPO: &str = "https://github.com/FMastro/woodsketch";

/// Posts a report's JSON to the relay, answering its status and body. Swapped out in tests.
pub type Post = Arc<dyn Fn(&str) -> Result<(u16, String), String> + Send + Sync>;

/// The real post: to woodsketch.net, saying it's the app.
fn post_to_relay() -> Post {
    Arc::new(|body: &str| {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .build()
            .into();
        let mut response = agent
            .post(RELAY)
            .header("User-Agent", &format!("WoodSketch/{}", crate::updates::VERSION))
            .header("Content-Type", "application/json")
            .send(body)
            .map_err(|e| t!("feedback-unreachable", "error" => e.to_string()))?;
        let status = response.status().as_u16();
        let text = response.body_mut().read_to_string().unwrap_or_default();
        Ok((status, text))
    })
}

/// Sending, and how it went.
#[derive(Clone, Default)]
enum Sending {
    #[default]
    No,
    Waiting,
    Done(Sent),
    Failed(String),
}

pub struct FeedbackWindow {
    pub open: bool,
    kind: Kind,
    title: String,
    what: String,
    email: String,
    /// Counts from the open drawing go with it (on by default; no names).
    skip_drawing: bool,
    /// Focus the title when the form opens.
    focus_title: bool,
    /// Reporting the crash the last session left: it goes with the details.
    crash: Option<Crash>,
    sending: Arc<Mutex<Sending>>,
    post: Post,
}

impl Default for FeedbackWindow {
    fn default() -> Self {
        Self {
            open: false,
            kind: Kind::default(),
            title: String::new(),
            what: String::new(),
            email: String::new(),
            skip_drawing: false,
            focus_title: false,
            crash: None,
            sending: Arc::default(),
            post: post_to_relay(),
        }
    }
}

impl FeedbackWindow {
    pub fn start(&mut self, kind: Kind) {
        let post = self.post.clone();
        *self = Self { open: true, kind, focus_title: true, post, ..Self::default() };
    }

    /// Report the crash the last session left: the form filled in, the crash in the details.
    pub fn start_crash(&mut self, crash: Crash) {
        self.start(Kind::Problem);
        self.title = crash.title();
        self.what = format!("{} ", t!("feedback-after-crash"));
        self.focus_title = false;
        self.crash = Some(crash);
    }

    /// Send reports with `post` instead of to woodsketch.net (tests).
    pub fn set_post(&mut self, post: Post) {
        self.post = post;
    }

    /// The form. `graphics` is the card and driver; `last_error` the last error shown.
    pub fn show(&mut self, ctx: &egui::Context, doc: &Document, graphics: &str, last_error: Option<&str>) {
        if !self.open {
            return;
        }
        let mut open = true;
        let mut close = false;
        let heading = match self.kind {
            Kind::Problem => t!("feedback-problem-title"),
            Kind::Idea => t!("feedback-idea-title"),
        };
        let sending = self.sending.lock().map(|s| s.clone()).unwrap_or_default();
        egui::Window::new(heading).open(&mut open).collapsible(false).resizable(false).default_width(460.0).show(ctx, |ui| {
            // Sent: the issue it made, and nothing else to do.
            if let Sending::Done(sent) = &sending {
                ui.label(t!("feedback-thanks", "check" => egui_phosphor::regular::CHECK_CIRCLE, "number" => sent.number));
                ui.hyperlink_to(&sent.url, &sent.url);
                ui.weak(if sent.emailed { t!("feedback-follow-email") } else { t!("feedback-follow") });
                if ui.button(t!("feedback-close")).clicked() {
                    close = true;
                }
                return;
            }
            let busy = matches!(sending, Sending::Waiting);
            ui.add_enabled_ui(!busy, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.kind, Kind::Problem, t!("feedback-kind-problem"));
                    ui.selectable_value(&mut self.kind, Kind::Idea, t!("feedback-kind-idea"));
                });
                ui.add_space(4.0);
                ui.label(t!("feedback-title-label"));
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
                    Kind::Problem => t!("feedback-what-happened"),
                    Kind::Idea => t!("feedback-your-idea"),
                });
                ui.add(egui::TextEdit::multiline(&mut self.what).id(what_id).lock_focus(true).desired_width(f32::INFINITY).desired_rows(5));
                ui.label(t!("feedback-email"));
                ui.add(egui::TextEdit::singleline(&mut self.email).desired_width(f32::INFINITY).hint_text(t!("feedback-email-hint")));
                ui.weak(t!("feedback-email-note"));
                let mut with_drawing = !self.skip_drawing;
                ui.checkbox(&mut with_drawing, t!("feedback-include-counts"))
                    .on_hover_text(t!("feedback-include-counts-hint"));
                self.skip_drawing = !with_drawing;
            });

            // Everything that goes with it, as it'll be sent.
            let (home, user) = crate::crash::private();
            let system = System::here(crate::updates::VERSION, graphics, woodsketch_doc::units::units().name());
            let drawing = (!self.skip_drawing).then(|| Drawing::of(doc.scene()));
            let error = last_error.filter(|_| self.kind == Kind::Problem).map(|e| scrub(e, home.as_deref(), user.as_deref()));
            let crash = self.crash.as_ref().filter(|_| self.kind == Kind::Problem);
            let text = details(&system, drawing.as_ref(), error.as_deref(), crash);
            ui.add_space(4.0);
            ui.label(t!("feedback-sent-with"));
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.add(egui::Label::new(egui::RichText::new(&text).monospace().small()).wrap());
            });
            ui.weak(t!("feedback-public-note"));
            if let Sending::Failed(why) = &sending {
                ui.colored_label(ui.visuals().warn_fg_color, why);
            }
            ui.add_space(4.0);
            let email = self.email.trim();
            let email_ok = email.is_empty() || valid_email(email);
            if !email_ok {
                ui.colored_label(ui.visuals().warn_fg_color, t!("feedback-bad-email"));
            }
            ui.horizontal(|ui| {
                let ready = !self.what.trim().is_empty() && email_ok && !busy;
                let title = scrub(self.title.trim(), home.as_deref(), user.as_deref());
                let what = scrub(self.what.trim(), home.as_deref(), user.as_deref());
                if ui.add_enabled(ready, egui::Button::new(format!("{} {}", egui_phosphor::regular::PAPER_PLANE_TILT, t!("feedback-send")))).clicked() {
                    let body = request(self.kind, &title, &what, &text, (!email.is_empty()).then_some(email));
                    let (post, state, ctx) = (self.post.clone(), self.sending.clone(), ui.ctx().clone());
                    if let Ok(mut s) = state.lock() {
                        *s = Sending::Waiting;
                    }
                    std::thread::spawn(move || {
                        let result = post(&body).and_then(|(status, text)| reply(status, &text));
                        if let Ok(mut s) = state.lock() {
                            *s = match result {
                                Ok(sent) => Sending::Done(sent),
                                Err(why) => Sending::Failed(why),
                            };
                        }
                        ctx.request_repaint();
                    });
                }
                if busy {
                    ui.spinner();
                    ui.label(t!("feedback-sending"));
                }
                // Or submit it on GitHub (an account is needed there; the email doesn't go).
                if ui
                    .add_enabled(ready, egui::Button::new(t!("feedback-open-github")))
                    .on_hover_text(t!("feedback-open-github-hint"))
                    .clicked()
                {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(issue_url(REPO, self.kind, &title, &what, &text)));
                    close = true;
                }
                if ui.add_enabled(!busy, egui::Button::new(t!("feedback-cancel"))).clicked() {
                    close = true;
                }
            });
        });
        if !open || close {
            let post = self.post.clone();
            *self = Self { post, ..Self::default() };
        }
    }
}
