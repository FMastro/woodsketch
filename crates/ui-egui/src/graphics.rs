//! Choosing the graphics backend: the list worth trying on this computer, the note the last run
//! left in the settings folder, and the plumbing that pins eframe to one backend.
//!
//! The rule for which one to start on is [`woodsketch_doc::graphics`], where it can be tested
//! without a graphics card. This is the part that has to touch the machine.

use std::path::{Path, PathBuf};

pub use woodsketch_doc::graphics::said;
use woodsketch_doc::graphics::{self as rule, Tried};

/// Beside the crash report and the autosaves.
const FILE: &str = "graphics.txt";

/// The backends worth trying here, best first.
///
/// Vulkan stays first on Windows and Linux, so nothing changes for a machine where it works --
/// which is the great majority, and all of them until one proves otherwise. macOS has one answer
/// and no fall-back: Metal is Apple's own, and wgpu's Vulkan there needs a MoltenVK we don't ship.
pub fn order() -> &'static [&'static str] {
    if cfg!(target_os = "macos") {
        &[rule::METAL]
    } else if cfg!(target_os = "windows") {
        &[rule::VULKAN, rule::DX12, rule::GL]
    } else {
        &[rule::VULKAN, rule::GL]
    }
}

/// What to try this run, in order. An attempt of `None` is "whatever wgpu picks", which is what
/// `WGPU_BACKEND` asks for.
pub struct Plan {
    /// Never empty: the first that starts wins, the rest are there in case it can't.
    pub attempts: Vec<Option<&'static str>>,
    /// A backend being left behind, to say so rather than change in silence.
    pub after: Option<String>,
}

/// Which backend to draw with, from the note the last run left in `dir`.
pub fn plan(dir: Option<&Path>) -> Plan {
    // Someone who names a backend has said the last word: wgpu reads `WGPU_BACKEND` itself, so
    // the way to honor it is to leave the options alone. Nothing is written down either -- a
    // backend asked for by hand shouldn't decide what the next run does.
    if let Some(asked) = asked_for() {
        log::info!("drawing with {asked}, as WGPU_BACKEND asks");
        return Plan { attempts: vec![None], after: None };
    }
    let last = dir.and_then(left);
    let start = rule::start(order(), last.as_ref(), crate::updates::VERSION);
    if let Some(failed) = &start.after {
        log::warn!("{} went wrong last time; drawing with {} instead", rule::said(failed), rule::said(start.order[0]));
    }
    Plan { attempts: start.order.into_iter().map(Some).collect(), after: start.after }
}

/// Draw with `backend`, where one was chosen. eframe would otherwise take whichever wgpu likes,
/// which is Vulkan on Windows whenever a driver claims to offer it (#13).
pub fn draw_with(options: &mut eframe::NativeOptions, backend: Option<&str>) {
    let Some(backends) = backend.and_then(backends) else { return };
    if let egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup {
        setup.instance_descriptor.backends = backends;
    }
}

fn backends(backend: &str) -> Option<wgpu::Backends> {
    Some(match backend {
        rule::VULKAN => wgpu::Backends::VULKAN,
        rule::DX12 => wgpu::Backends::DX12,
        rule::METAL => wgpu::Backends::METAL,
        rule::GL => wgpu::Backends::GL,
        _ => return None,
    })
}

/// The note the last run left in `dir`.
fn left(dir: &Path) -> Option<Tried> {
    Tried::from_text(&std::fs::read_to_string(dir.join(FILE)).ok()?)
}

/// What `WGPU_BACKEND` says, if it says anything.
fn asked_for() -> Option<String> {
    std::env::var("WGPU_BACKEND").ok().map(|asked| asked.trim().to_owned()).filter(|asked| !asked.is_empty())
}

/// Write down that a run is about to try `backend`, before it does.
///
/// It has to be written before the window opens, not after: a driver that takes the process down
/// while wgpu is starting it leaves nothing behind to learn from, and the next run would try the
/// same backend and die the same way. [`Record`] writes it again with the backend that actually
/// started.
pub fn trying(dir: Option<&Path>, backend: Option<&str>) {
    if let (Some(dir), Some(backend)) = (dir, backend) {
        write(dir, &Tried::starting(backend, crate::updates::VERSION));
    }
}

fn write(dir: &Path, tried: &Tried) {
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(dir.join(FILE), tried.to_text());
}

/// The note this run leaves for the next one: which backend drew, how far it got, and anything
/// the card complained about.
pub struct Record {
    /// The settings folder, or nothing when there is none (tests) or when `WGPU_BACKEND` named
    /// the backend, since a backend asked for by hand shouldn't decide what the next run does.
    dir: Option<PathBuf>,
    tried: Tried,
}

impl Record {
    /// Starts the note for a run drawing on `backend` -- the one wgpu really gave us, which is
    /// the one worth writing down.
    pub fn new(dir: Option<&Path>, backend: wgpu::Backend) -> Self {
        let record = Self { dir: dir.filter(|_| asked_for().is_none()).map(Path::to_owned), tried: Tried::starting(backend.to_str(), crate::updates::VERSION) };
        record.save();
        record
    }

    /// The backend being drawn with, as wgpu spells it.
    pub fn backend(&self) -> &str {
        &self.tried.backend
    }

    /// A frame went by. Written down once the window has drawn enough to count as working, so a
    /// backend that takes the app down on its first frame isn't recorded as one that works.
    pub fn drew(&mut self) {
        if self.tried.drew() {
            self.save();
        }
    }

    /// The card complained (`gpu::Trouble`). The app carries on, but the next run starts on
    /// another backend: this one has shown what it does.
    pub fn trouble(&mut self, said: &str) {
        if self.tried.trouble.is_none() {
            self.tried.trouble = Some(said.to_owned());
            self.save();
        }
    }

    fn save(&self) {
        if let Some(dir) = &self.dir {
            write(dir, &self.tried);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vulkan is still first where it works, so a machine that was fine stays fine.
    #[test]
    fn the_preferred_backend_is_still_the_one_wgpu_would_have_picked() {
        let order = order();
        assert!(!order.is_empty());
        if cfg!(target_os = "macos") {
            assert_eq!(order, [rule::METAL]);
        } else {
            assert_eq!(order[0], rule::VULKAN);
        }
        // Every name on the list is one we can ask wgpu for.
        for backend in order {
            assert!(backends(backend).is_some(), "{backend}");
        }
    }

    /// The note is written as the run goes, so a run that dies leaves what it got to.
    #[test]
    fn a_run_writes_down_how_far_it_got() {
        let dir = std::env::temp_dir().join(format!("woodsketch-graphics-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(left(&dir), None);
        trying(Some(&dir), Some(rule::VULKAN));
        let started = left(&dir).expect("the attempt is written down before the window opens");
        assert!(started.failed() && started.backend == rule::VULKAN);
        let mut record = Record::new(Some(&dir), wgpu::Backend::Dx12);
        assert_eq!(record.backend(), rule::DX12, "the backend that really started is the one kept");
        for _ in 0..4 {
            record.drew();
        }
        assert!(!left(&dir).unwrap().failed(), "a window that drew is not a window that failed");
        record.trouble("Validation Error");
        let after = left(&dir).unwrap();
        assert!(after.failed() && after.trouble.as_deref() == Some("Validation Error"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Without a settings folder (every test that drives the app) nothing is written anywhere.
    #[test]
    fn a_run_with_nowhere_to_write_writes_nothing() {
        trying(None, Some(rule::VULKAN));
        let mut record = Record::new(None, wgpu::Backend::Vulkan);
        record.drew();
        record.trouble("Validation Error");
        assert_eq!(record.backend(), rule::VULKAN);
    }
}
