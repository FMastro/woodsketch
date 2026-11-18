//! What to do when the graphics card says no.
//!
//! `wgpu` reports an error it can't hand back to a caller -- a validation error, a device lost,
//! a surface it couldn't configure -- to the device's uncaptured-error handler, and the default
//! handler panics. So a driver hiccup on one frame closed the window, which is how a report came
//! in reading "WoodSketch closed unexpectedly" with `Failed to wait for GPU to come idle before
//! reconfiguring the Surface` under it, on an eleven-year-old laptop chip, with an empty drawing
//! on screen and nothing of the user's at stake but the session itself (#13).
//!
//! A frame is not worth a session. The error is written down here instead, where the status bar
//! shows it and a problem report carries it, and the app stays up: a drawing is still in memory,
//! can still be saved, and the next frame usually draws.
//!
//! This is a seatbelt, not a repair. It does not make the driver work, and if the surface really
//! can't be configured the window may well go black -- but it is then a black window with a Save
//! that works, rather than no window at all.

use std::sync::{Arc, Mutex};

use woodsketch_i18n::t;

/// The card, the backend drawing on it and its driver, for a problem report.
pub fn describe(adapter: &wgpu::Adapter) -> String {
    let info = adapter.get_info();
    let backend = format!("{:?}", info.backend);
    woodsketch_doc::feedback::graphics_line(&info.name, &backend, kind(info.device_type), &info.driver, &info.driver_info)
}

/// What kind of card it is, in a word. English, like the rest of the report.
fn kind(device: wgpu::DeviceType) -> &'static str {
    match device {
        wgpu::DeviceType::DiscreteGpu => "discrete",
        wgpu::DeviceType::IntegratedGpu => "integrated",
        wgpu::DeviceType::VirtualGpu => "virtual",
        wgpu::DeviceType::Cpu => "cpu",
        wgpu::DeviceType::Other => "",
    }
}

/// Whatever the graphics card last complained about. Shared with the handler `wgpu` calls, which
/// runs on whichever thread hit the error.
#[derive(Clone, Default)]
pub struct Trouble(Arc<Mutex<Option<String>>>);

impl Trouble {
    /// Takes over from `wgpu`'s panicking handler on `device`.
    pub fn watch(device: &wgpu::Device) -> Self {
        let trouble = Self::default();
        let said = trouble.0.clone();
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            // Logged in full, kept short for the screen: the full text runs to several lines of
            // backend detail, and the status bar has one.
            log::error!("the graphics card reported: {error}");
            let whole = error.to_string();
            let first = whole.lines().next().unwrap_or(&whole).trim().to_owned();
            *said.lock().unwrap_or_else(|e| e.into_inner()) = Some(first);
        }));
        trouble
    }

    /// What it said, once, so the status bar shows each complaint rather than the first forever.
    /// A driver that fails every frame would otherwise pin a message there for good.
    pub fn taken(&self) -> Option<String> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).take()
    }

    /// Said for the screen: what went wrong is the driver's words, so the sentence around them
    /// is ours.
    pub fn said(error: &str) -> String {
        t!("gpu-trouble", "error" => error.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nothing to say until something goes wrong, and each complaint is handed over once.
    #[test]
    fn a_complaint_is_read_once() {
        let trouble = Trouble::default();
        assert_eq!(trouble.taken(), None);
        *trouble.0.lock().unwrap() = Some("Validation Error".to_owned());
        assert_eq!(trouble.taken().as_deref(), Some("Validation Error"));
        assert_eq!(trouble.taken(), None, "a card failing every frame mustn't pin one message on screen");
    }

    /// The screen gets one line, whatever the backend wrote.
    #[test]
    fn only_the_first_line_of_a_complaint_is_kept() {
        let whole = "Validation Error\n\nCaused by:\n  In Surface::configure\n    Failed to wait for GPU to come idle";
        assert_eq!(whole.lines().next(), Some("Validation Error"));
        assert!(Trouble::said("Validation Error").contains("Validation Error"));
    }
}
