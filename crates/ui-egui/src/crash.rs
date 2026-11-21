//! Crash reports. A panic hook saves what happened ([`Crash`]: the message with paths and the
//! user's name taken out, where in the code, the calls that led there, the version) to
//! WoodSketch's settings folder; the next start finds it and offers to report it (Help > Report a
//! Problem, filled in).

use std::path::{Path, PathBuf};

use woodsketch_doc::feedback::{Crash, own_frames, scrub, source_location};

const FILE: &str = "crash.txt";
/// Written while the app is running and taken away when it stops properly; see [`running`].
const RUNNING: &str = "running.txt";
/// A [`RUNNING`] note that was still there when the next run started, so the run it belonged to
/// never said goodbye.
const STOPPED: &str = "stopped.txt";
/// At most this many calls are kept.
const FRAMES: usize = 12;

/// Save any panic into `dir`, after the usual message.
pub fn install(dir: PathBuf) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        previous(info);
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "(no message)".to_owned());
        let location = info.location().map_or_else(|| "unknown".to_owned(), |l| source_location(l.file(), l.line(), l.column()));
        // The calls that led there, WoodSketch's own (function names; line numbers too where the
        // build has them).
        let frames = own_frames(&std::backtrace::Backtrace::force_capture().to_string(), FRAMES);
        let (home, user) = private();
        let crash = Crash { message: scrub(&message, home.as_deref(), user.as_deref()), location, frames, version: crate::updates::VERSION.to_owned() };
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(FILE), crash.to_text());
    }));
}

/// The crash the last session left in `dir`, if any. It stays until [`clear`].
pub fn left(dir: &Path) -> Option<Crash> {
    Crash::from_text(&std::fs::read_to_string(dir.join(FILE)).ok()?)
}

/// Forget it (reported, or not wanted).
pub fn clear(dir: &Path) {
    let _ = std::fs::remove_file(dir.join(FILE));
    // A panic explains the run that ended, so the note left over from it has nothing to add.
    let _ = std::fs::remove_file(dir.join(STOPPED));
}

/// Note that this version is running, so the next start can tell whether it ever stopped properly.
/// Called once at the start of a run, before the window opens. A note left over from the last run
/// is set aside as [`STOPPED`] first -- that it is still there is the whole of the evidence, and
/// moving it keeps this run's own note from being read as it.
///
/// The panic hook above covers a Rust panic, and nothing else: a graphics driver that takes the
/// process down with it, running out of memory, and a stack overflow all end the program
/// somewhere no Rust code of ours will run again, and they are exactly the failures that leave a
/// reader with nothing to send. Catching those where they happen means a signal or exception
/// handler, which cannot be written without `unsafe`, forbidden across this workspace. What needs
/// no handler at all is noticing, next time, that the last run never said goodbye.
///
/// It is a guess, and an honest one: a machine switched off at the wall and an app ended from Task
/// Manager look the same as a driver taking it down. So what is offered says only that it stopped
/// without saying why, and the report carries the graphics card and driver, which is usually the
/// answer.
pub fn running(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::rename(dir.join(RUNNING), dir.join(STOPPED));
    // The version is kept in the note rather than assumed at the next start, which may be a
    // different one: a package manager can replace the app between runs.
    let _ = std::fs::write(dir.join(RUNNING), crate::updates::VERSION);
}

/// Note that this run ended the way it was meant to. Called when the window has closed, and
/// before an update restarts the app, which would otherwise look like a stop with no reason.
pub fn ended(dir: &Path) {
    let _ = std::fs::remove_file(dir.join(RUNNING));
}

/// A run that stopped leaving no reason, as a crash to offer.
///
/// `None` when the last run ended properly, and `None` when it panicked: a panic says far more
/// about itself, and [`left`] offers that instead. The note is taken away as it is read, so a
/// stop is offered once however it is answered.
pub fn stopped(dir: &Path) -> Option<Crash> {
    let version = std::fs::read_to_string(dir.join(STOPPED)).ok()?;
    let _ = std::fs::remove_file(dir.join(STOPPED));
    if dir.join(FILE).exists() {
        return None;
    }
    // No message to scrub, no location, no calls: that there are none is the whole report, and
    // what makes the window say so rather than promise a place in the code.
    Some(Crash {
        message: "WoodSketch stopped without saying why.".to_owned(),
        location: String::new(),
        frames: Vec::new(),
        version: version.trim().to_owned(),
    })
}

/// The home folder and the user's name, to take out of anything reported.
pub fn private() -> (Option<String>, Option<String>) {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok();
    let user = std::env::var("USERNAME").or_else(|_| std::env::var("USER")).ok();
    (home, user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_crash_is_found_once_until_cleared() {
        let dir = std::env::temp_dir().join(format!("woodsketch-crash-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(left(&dir), None);
        std::fs::create_dir_all(&dir).unwrap();
        let crash = Crash {
            message: "boom".into(),
            location: "crates/doc/src/model.rs:1:1".into(),
            frames: vec!["woodsketch_doc::model::Model::push_pull".into()],
            version: "0.1.47".into(),
        };
        std::fs::write(dir.join(FILE), crash.to_text()).unwrap();
        assert_eq!(left(&dir), Some(crash));
        clear(&dir);
        assert_eq!(left(&dir), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A scratch settings folder for one test, cleaned up after.
    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("woodsketch-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("a scratch folder");
            Self(dir)
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_run_that_ended_properly_is_not_a_crash() {
        let at = Folder::new("ended");
        running(&at.0);
        ended(&at.0);
        // The next run.
        running(&at.0);
        assert_eq!(stopped(&at.0), None, "nothing to report: it closed the way it should");
    }

    #[test]
    fn a_run_that_stopped_without_a_word_is_offered_as_one() {
        let at = Folder::new("stopped");
        running(&at.0);
        // No `ended`: the process went away where no code of ours ran again. Then the next run,
        // whose own note must not be mistaken for the one left behind.
        running(&at.0);
        let crash = stopped(&at.0).expect("a stop to report");
        assert_eq!(crash.version, crate::updates::VERSION, "the version that was running, read from the note");
        assert!(crash.location.is_empty() && crash.frames.is_empty(), "there is nothing to say about where, and it mustn't pretend there is");
        assert_eq!(crash.title(), "Crash: WoodSketch stopped without saying why.");
        assert_eq!(stopped(&at.0), None, "offered once, not at every start from now on");
    }

    /// The version comes out of the note, not out of this build: a package manager can replace the
    /// app between the run that died and the one that reports it.
    #[test]
    fn the_version_reported_is_the_one_that_was_running() {
        let at = Folder::new("version");
        std::fs::write(at.0.join(RUNNING), "0.1.50\n").unwrap();
        running(&at.0);
        assert_eq!(stopped(&at.0).expect("a stop").version, "0.1.50");
    }

    #[test]
    fn a_panic_is_reported_as_a_panic_rather_than_as_a_silent_stop() {
        let at = Folder::new("panicked");
        // The run that panicked, then the one that finds what it left.
        running(&at.0);
        running(&at.0);
        let panicked = Crash {
            message: "index out of bounds".into(),
            location: "crates/doc/src/model.rs:12:5".into(),
            frames: vec!["woodsketch_doc::model::Model::face".into()],
            version: crate::updates::VERSION.to_owned(),
        };
        std::fs::write(at.0.join(FILE), panicked.to_text()).unwrap();
        assert_eq!(stopped(&at.0), None, "the panic says far more, and `left` offers that");
        assert_eq!(left(&at.0), Some(panicked));
        // Reporting it clears both, so the panic isn't followed by a silent stop next time.
        clear(&at.0);
        assert_eq!(left(&at.0), None);
        assert_eq!(stopped(&at.0), None);
    }
}
