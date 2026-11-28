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
    /// Takes over from `wgpu`'s panicking handler on `device`. `tell` is handed each complaint
    /// as it arrives, for whoever has to know before the app gets a chance to ask -- in practice
    /// `graphics::Note`, writing down that this backend went wrong.
    pub fn watch(device: &wgpu::Device, tell: impl Fn(&str) + Send + Sync + 'static) -> Self {
        let trouble = Self::default();
        let heard = trouble.clone();
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| heard.heard(&error.to_string(), &tell)));
        trouble
    }

    /// What to do with a complaint, wherever it came from.
    ///
    /// `tell` goes first, and that order is the point of it. The app usually stays up and reads
    /// [`Self::taken`] on the next frame -- but where it doesn't, there is no next frame: a
    /// staging allocation the driver refuses arrives here and is panicked on by `egui-wgpu` a
    /// statement later (#22), and all that survives is what reached the disk before this
    /// returned.
    fn heard(&self, whole: &str, tell: &dyn Fn(&str)) {
        // Logged in full, kept short for the screen: the full text runs to several lines of
        // backend detail, and the status bar has one.
        log::error!("the graphics card reported: {whole}");
        let first = whole.lines().next().unwrap_or(whole).trim().to_owned();
        tell(&first);
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(first);
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

    /// A complaint is handed on as it arrives, before it is stored for the next frame to read.
    /// Where the app doesn't survive the frame (#22) that hand-off is the only thing that
    /// happens at all, so the order here is the behaviour, not an implementation detail.
    #[test]
    fn a_complaint_is_handed_on_the_moment_it_arrives() {
        let trouble = Trouble::default();
        let told: Arc<Mutex<Vec<String>>> = Arc::default();
        let keep = Arc::clone(&told);
        let whole = "Validation Error\n\nCaused by:\n  In Queue::write_buffer_with\n    Out of memory";
        trouble.heard(whole, &move |said: &str| keep.lock().unwrap_or_else(|e| e.into_inner()).push(said.to_owned()));
        assert_eq!(told.lock().unwrap().as_slice(), ["Validation Error".to_owned()], "the card's first line, handed straight on");
        assert_eq!(trouble.taken().as_deref(), Some("Validation Error"), "and kept for the status bar as well");
    }

    /// The screen gets one line, whatever the backend wrote.
    #[test]
    fn only_the_first_line_of_a_complaint_is_kept() {
        let whole = "Validation Error\n\nCaused by:\n  In Surface::configure\n    Failed to wait for GPU to come idle";
        assert_eq!(whole.lines().next(), Some("Validation Error"));
        assert!(Trouble::said("Validation Error").contains("Validation Error"));
    }
}
