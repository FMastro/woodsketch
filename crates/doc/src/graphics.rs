//! Which graphics backend to draw with, and how to step off one that doesn't work.
//!
//! wgpu picks a backend in its own order -- Vulkan, then Metal, then DX12, then OpenGL -- so on
//! Windows the app lands on Vulkan whenever a driver exposes one at all, however old that driver
//! is and however badly it works. Nobody chose that; it was inherited. It is how a report came in
//! from an eleven-year-old laptop chip whose Vulkan driver couldn't reconfigure a surface, which
//! closed the window (#13).
//!
//! So the choice is made here instead. There is a list to try, best first, and a note of what the
//! last run did with the one it picked. Two things count as going wrong: a run that never got as
//! far as a second frame, and a run whose graphics card complained. Either one moves the next run
//! on to the backend after it; nothing else does, so a machine where Vulkan works keeps Vulkan.
//!
//! A new version of WoodSketch starts from the top of the list again. It brings a new wgpu with
//! it, and nobody should be left on a fall-back for ever by a bug that has since been fixed
//! elsewhere.

/// How a backend is named in the note and in `WGPU_BACKEND`: wgpu's own spelling.
pub const VULKAN: &str = "vulkan";
/// Direct3D 12, Windows.
pub const DX12: &str = "dx12";
/// Apple's, and the only one there.
pub const METAL: &str = "metal";
/// OpenGL, the oldest and slowest way to draw, and the one most likely to be there.
pub const GL: &str = "gl";

/// A run that never reached this many frames never really drew: the first frame can be submitted
/// and still take the driver down with it, so the proof of life is the frame after it starting.
const DREW_AFTER: u32 = 2;

/// What a run of the app left behind about the graphics it drew with. Saved in the settings
/// folder beside the crash report (`ui-egui::graphics`), read by the next start.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tried {
    /// The backend it drew with, as wgpu names one ([`VULKAN`] and the rest).
    pub backend: String,
    /// The version of WoodSketch that tried it.
    pub version: String,
    /// How many frames it got through.
    pub frames: u32,
    /// What the graphics card complained about, if it did.
    pub trouble: Option<String>,
}

impl Tried {
    /// The note a run starts with: this backend, this version, nothing drawn yet.
    pub fn starting(backend: &str, version: &str) -> Self {
        Self { backend: backend.to_owned(), version: version.to_owned(), frames: 0, trouble: None }
    }

    /// As saved: the backend, the version, the frames, then anything the card said.
    pub fn to_text(&self) -> String {
        let mut text = format!("{}\n{}\n{}\n", self.backend, self.version, self.frames);
        if let Some(trouble) = &self.trouble {
            text.push_str(trouble);
        }
        text
    }

    pub fn from_text(text: &str) -> Option<Self> {
        let mut lines = text.lines();
        let backend = lines.next()?.trim().to_owned();
        let version = lines.next()?.trim().to_owned();
        let frames = lines.next()?.trim().parse().ok()?;
        let trouble = lines.collect::<Vec<_>>().join("\n").trim().to_owned();
        Some(Self { backend, version, frames, trouble: (!trouble.is_empty()).then_some(trouble) })
    }

    /// A frame went by. True at the one moment the run has drawn enough to count as working,
    /// which is the moment worth writing down.
    pub fn drew(&mut self) -> bool {
        self.frames = self.frames.saturating_add(1);
        self.frames == DREW_AFTER
    }

    /// Whether the run went wrong in the graphics: it never drew, or the card complained.
    pub fn failed(&self) -> bool {
        self.frames < DREW_AFTER || self.trouble.is_some()
    }
}

/// What to draw with this run: the backends to try, the chosen one first, and the one being left
/// behind if this run is stepping off one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Start<'a> {
    /// Best first. The first that starts wins; the rest are there in case it can't.
    pub order: Vec<&'a str>,
    /// The backend the last run went wrong on, to say so plainly rather than changing in silence.
    pub after: Option<String>,
}

/// Which backend to start on, given those worth trying here (best first), what the last run left
/// behind, and the version running now.
pub fn start<'a>(order: &[&'a str], last: Option<&Tried>, version: &str) -> Start<'a> {
    let from_the_top = Start { order: order.to_vec(), after: None };
    // Nothing tried yet, or tried by another version of WoodSketch, which brought another wgpu:
    // let the preferred backend prove itself again.
    let Some(last) = last.filter(|last| last.version == version) else { return from_the_top };
    // A backend that isn't on the list any more (`WGPU_BACKEND` named it, or the list changed)
    // says nothing about which of these to pick.
    let Some(at) = order.iter().position(|backend| *backend == last.backend) else { return from_the_top };
    if !last.failed() {
        // It worked. Keep it, with the rest of the list behind it in case it stops working.
        return Start { order: from(order, at), after: None };
    }
    // Round the end rather than stopping at it: with everything tried and nothing working, the
    // top of the list is as good a guess as the bottom, and the driver may have been updated.
    Start { order: from(order, (at + 1) % order.len()), after: Some(last.backend.clone()) }
}

/// The list from `at` on, wrapping round: the chosen backend first, the others behind it.
fn from<'a>(order: &[&'a str], at: usize) -> Vec<&'a str> {
    order[at..].iter().chain(&order[..at]).copied().collect()
}

/// What a backend is called on screen. The spellings above are wgpu's, and what `WGPU_BACKEND`
/// takes; these are for people.
pub fn said(backend: &str) -> &str {
    match backend {
        VULKAN => "Vulkan",
        DX12 => "DirectX 12",
        METAL => "Metal",
        GL => "OpenGL",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOWS: [&str; 3] = [VULKAN, DX12, GL];

    /// With nothing to go on, the preferred backend is tried first and the others stand behind it.
    #[test]
    fn the_first_run_tries_the_best_backend_first() {
        assert_eq!(start(&WINDOWS, None, "0.1.75"), Start { order: vec![VULKAN, DX12, GL], after: None });
    }

    /// A backend that worked is kept -- which is every machine where nothing is wrong.
    #[test]
    fn a_backend_that_drew_is_kept() {
        let drew = Tried { frames: 240, ..Tried::starting(VULKAN, "0.1.75") };
        assert_eq!(start(&WINDOWS, Some(&drew), "0.1.75"), Start { order: vec![VULKAN, DX12, GL], after: None });
        // And so is one that isn't the preferred one: the fall-back doesn't un-fall-back itself.
        let drew = Tried { frames: 240, ..Tried::starting(DX12, "0.1.75") };
        assert_eq!(start(&WINDOWS, Some(&drew), "0.1.75"), Start { order: vec![DX12, GL, VULKAN], after: None });
    }

    /// The two ways a run goes wrong, and the one way it doesn't.
    #[test]
    fn a_backend_that_went_wrong_is_stepped_off() {
        // It started but never drew: whatever it was, it wasn't a working window.
        let never_drew = Tried::starting(VULKAN, "0.1.75");
        assert_eq!(start(&WINDOWS, Some(&never_drew), "0.1.75"), Start { order: vec![DX12, GL, VULKAN], after: Some(VULKAN.to_owned()) });
        // It drew, and then the card complained -- #13 itself, where the window went away.
        let complained = Tried { frames: 1_200, trouble: Some("Validation Error".to_owned()), ..Tried::starting(VULKAN, "0.1.75") };
        assert_eq!(start(&WINDOWS, Some(&complained), "0.1.75"), Start { order: vec![DX12, GL, VULKAN], after: Some(VULKAN.to_owned()) });
        // One frame is not a working window either: a driver can take the app down presenting it.
        let mut counting = Tried::starting(VULKAN, "0.1.75");
        assert!(!counting.drew() && counting.failed(), "one frame proves nothing");
        assert!(counting.drew(), "the frame after it is the moment it counts as working");
        assert!(!counting.failed());
        assert!(!counting.drew(), "and it is written down once, not every frame after");
    }

    /// Nothing works: round the list rather than stuck on the last of it.
    #[test]
    fn the_list_comes_round_again() {
        let last_one_failed = Tried::starting(GL, "0.1.75");
        assert_eq!(start(&WINDOWS, Some(&last_one_failed), "0.1.75"), Start { order: vec![VULKAN, DX12, GL], after: Some(GL.to_owned()) });
    }

    /// A new version brings a new wgpu, so the preferred backend gets another go. Nobody is left
    /// on OpenGL for ever by a driver bug someone else has since fixed.
    #[test]
    fn a_new_version_starts_from_the_top_again() {
        let failed = Tried { trouble: Some("Device lost".to_owned()), ..Tried::starting(VULKAN, "0.1.74") };
        assert_eq!(start(&WINDOWS, Some(&failed), "0.1.75"), Start { order: vec![VULKAN, DX12, GL], after: None });
    }

    /// A backend nobody offers here says nothing about the ones that are offered.
    #[test]
    fn a_backend_that_isnt_on_the_list_is_no_guide() {
        let elsewhere = Tried::starting(METAL, "0.1.75");
        assert_eq!(start(&WINDOWS, Some(&elsewhere), "0.1.75"), Start { order: vec![VULKAN, DX12, GL], after: None });
        // One backend to try, and it failed: there is nowhere else to go, so it is tried again.
        let only = [METAL];
        assert_eq!(start(&only, Some(&elsewhere), "0.1.75"), Start { order: vec![METAL], after: Some(METAL.to_owned()) });
    }

    /// The note survives being written down, complaint and all.
    #[test]
    fn a_note_reads_back_as_it_was_written() {
        let plain = Tried { frames: 90, ..Tried::starting(DX12, "0.1.75") };
        assert_eq!(Tried::from_text(&plain.to_text()), Some(plain));
        let complained = Tried { frames: 3, trouble: Some("Validation Error\nIn Surface::configure".to_owned()), ..Tried::starting(VULKAN, "0.1.75") };
        assert_eq!(Tried::from_text(&complained.to_text()), Some(complained));
        // Half a file, or none, is no note at all rather than a wrong one.
        assert_eq!(Tried::from_text("vulkan\n0.1.75"), None);
        assert_eq!(Tried::from_text(""), None);
        assert_eq!(Tried::from_text("vulkan\n0.1.75\nlots\n"), None, "frames have to be a number");
    }

    /// Backends are spoken of by the names their makers use.
    #[test]
    fn a_backend_is_said_the_way_people_write_it() {
        assert_eq!([VULKAN, DX12, METAL, GL].map(said), ["Vulkan", "DirectX 12", "Metal", "OpenGL"]);
        assert_eq!(said("noop"), "noop", "anything else speaks for itself");
    }
}
