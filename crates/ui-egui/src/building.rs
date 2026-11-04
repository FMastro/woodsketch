//! Building a sample on a thread, so the app stays alive while it happens.
//!
//! The small samples take about 200 ms, which you can get away with on the click that asks for
//! one. The house takes a couple of seconds: long enough that a frozen window looks broken, and
//! long enough that there is something worth watching. So every sample is built the same way
//! here, on a thread, with a bar that says how far along it is.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, channel};

use eframe::egui;
use woodsketch_doc::Document;
use woodsketch_doc::samples::Sample;

/// How often the builder asks for a repaint. The bar moves smoothly at this rate and the thread
/// spends none of its time waking the UI up.
const TICK: std::time::Duration = std::time::Duration::from_millis(33);

pub struct Building {
    /// The sample being built, for the message.
    pub name: &'static str,
    done: Receiver<Result<Box<Document>, String>>,
    at: Arc<AtomicUsize>,
    of: Arc<AtomicUsize>,
}

impl Building {
    /// Start building `sample`. Returns at once; [`Self::ready`] hands over the drawing.
    pub fn start(sample: &'static Sample, ctx: &egui::Context) -> Self {
        let (tx, done) = channel();
        let (at, of) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(1)));
        let (mine, theirs, ctx) = (Arc::clone(&at), Arc::clone(&of), ctx.clone());
        let started = std::thread::Builder::new()
            .name(format!("build {}", sample.name))
            .spawn(move || {
                let mut last = std::time::Instant::now();
                let built = sample.build_reporting(&mut |i, n, _| {
                    mine.store(i, Ordering::Relaxed);
                    theirs.store(n.max(1), Ordering::Relaxed);
                    if last.elapsed() >= TICK {
                        last = std::time::Instant::now();
                        ctx.request_repaint();
                    }
                });
                // Boxed: a built drawing is megabytes, and this one is going down a channel.
                let _ = tx.send(built.map(Box::new));
                ctx.request_repaint();
            });
        if started.is_err() {
            // No thread to be had: build it here rather than leaving the sample unopenable.
            let (tx, done) = channel();
            let _ = tx.send(sample.build().map(Box::new));
            return Self { name: sample.name, done, at, of };
        }
        Self { name: sample.name, done, at, of }
    }

    /// How far along, 0 to 1. Parts arrive steadily, so this tracks the wait closely enough.
    pub fn how_far(&self) -> f32 {
        let (at, of) = (self.at.load(Ordering::Relaxed), self.of.load(Ordering::Relaxed));
        (at as f32 / of.max(1) as f32).clamp(0.0, 1.0)
    }

    /// The drawing, once it's built. `None` while the thread is still working.
    pub fn ready(&self) -> Option<Result<Box<Document>, String>> {
        self.done.try_recv().ok()
    }

    /// Say what's happening, over the empty drawing the sample is about to replace.
    pub fn show(&self, ctx: &egui::Context) {
        egui::Window::new("Building")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(260.0);
                ui.add_space(4.0);
                ui.strong(format!("Building the {}", self.name.to_lowercase()));
                ui.add_space(6.0);
                ui.add(egui::ProgressBar::new(self.how_far()).desired_height(8.0));
                ui.add_space(6.0);
                ui.weak("It's built from a recipe each time, by the joinery the app has now.");
                ui.add_space(4.0);
            });
    }
}
