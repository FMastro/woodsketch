//! Crash reports. A panic hook saves what happened ([`Crash`]: the message with paths and the
//! user's name taken out, where in the code, the calls that led there, the version) to
//! WoodSketch's settings folder; the next start finds it and offers to report it (Help > Report a
//! Problem, filled in).

use std::path::{Path, PathBuf};

use woodsketch_doc::feedback::{Crash, own_frames, scrub, source_location};

const FILE: &str = "crash.txt";
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
        let crash = Crash { message: "boom".into(), location: "crates/doc/src/model.rs:1:1".into(), frames: vec!["woodsketch_doc::model::Model::push_pull".into()], version: "0.1.47".into() };
        std::fs::write(dir.join(FILE), crash.to_text()).unwrap();
        assert_eq!(left(&dir), Some(crash));
        clear(&dir);
        assert_eq!(left(&dir), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
