//! Watching the open drawing's file, so changes made outside WoodSketch show up in the window.
//! The drawing an assistant is building through `woodsketch-cli` (or one in a synced folder, or
//! swapped by a checkout) is saved by that other program; without this, the window keeps showing
//! the drawing as it was, and saving from here would write the stale version back over it.
//!
//! The file is checked every [`INTERVAL`], by its modified time and length rather than by
//! watching the folder: a poll needs no extra dependency, and a drawing is saved by writing a
//! temporary file and renaming it over this one, which folder watchers report inconsistently
//! across platforms anyway.

use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

/// How often the open drawing's file is checked. Short enough to feel immediate while watching
/// an assistant work, long enough to cost nothing.
pub const INTERVAL: Duration = Duration::from_millis(750);

/// What the file looked like when it was last read or written from here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Stamp {
    modified: Option<SystemTime>,
    len: u64,
}

impl Stamp {
    /// The file as it is now, or `None` if it isn't there (deleted, or mid-rename on Windows:
    /// either way there's nothing to reload yet, so the next tick looks again).
    fn of(path: &Path) -> Option<Self> {
        let meta = std::fs::metadata(path).ok()?;
        Some(Self { modified: meta.modified().ok(), len: meta.len() })
    }
}

/// What a check turned up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Found {
    /// The file changed and nothing here is unsaved: reload it.
    Reload,
    /// The file changed and there are unsaved changes here: ask before losing either side.
    Ask,
}

#[derive(Default)]
pub struct Watch {
    /// The open file as it was when read or written; `None` for an unsaved drawing.
    stamp: Option<Stamp>,
    last: Option<Instant>,
    /// A change already reported and waiting on the user, so it isn't reported every tick.
    pending: bool,
}

impl Watch {
    /// The drawing was opened from, or saved to, this file: take it as current.
    pub fn stamped(&mut self, path: Option<&Path>) {
        self.stamp = path.and_then(Stamp::of);
        self.pending = false;
    }

    /// The user answered (reloaded, or kept their own): stop reporting this change. Keeping
    /// your own takes the file as it is now, so it isn't reported again.
    pub fn settled(&mut self, path: Option<&Path>) {
        self.stamped(path);
    }

    /// Look at the file on the next tick instead of waiting out the rest of the interval.
    pub fn check_now(&mut self) {
        self.last = Instant::now().checked_sub(INTERVAL);
        self.pending = false;
    }

    /// Check the file if [`INTERVAL`] has passed. Returns how long until the next check (to
    /// schedule a repaint, since an idle window paints only when asked) and what was found.
    pub fn tick(&mut self, path: Option<&Path>, dirty: bool, now: Instant) -> (Option<Duration>, Option<Found>) {
        let Some(path) = path else {
            self.stamp = None;
            return (None, None);
        };
        if self.pending {
            // Waiting on the user; keep painting so the question stays answerable.
            return (Some(INTERVAL), None);
        }
        // `None` is both "never checked" and, from `check_now` on a machine whose clock is
        // younger than the interval, "check now": either way, look.
        let due = self.last.map_or(now, |last| last + INTERVAL);
        if now < due {
            return (Some(due - now), None);
        }
        self.last = Some(now);
        let Some(current) = Stamp::of(path) else { return (Some(INTERVAL), None) };
        // Nothing recorded yet (the drawing was just opened): take this as the starting point.
        let Some(known) = self.stamp else {
            self.stamp = Some(current);
            return (Some(INTERVAL), None);
        };
        if current == known {
            return (Some(INTERVAL), None);
        }
        if dirty {
            self.pending = true;
            return (Some(INTERVAL), Some(Found::Ask));
        }
        self.stamp = Some(current);
        (Some(INTERVAL), Some(Found::Reload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("woodsketch-watch-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Write the file so its stamp differs: length alone is enough, and doesn't depend on the
    /// filesystem's modified-time resolution.
    fn write(path: &Path, text: &str) {
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn a_change_on_disk_is_found_once_the_interval_passes() {
        let dir = folder("found");
        let path = dir.join("nightstand.woodsketch");
        write(&path, "first");
        let mut watch = Watch::default();
        watch.stamped(Some(&path));
        let start = Instant::now();

        // Before the interval, nothing is even looked at.
        let (next, found) = watch.tick(Some(&path), false, start);
        assert_eq!(found, None);
        assert!(next.is_some(), "it asks to be checked again");

        // Unchanged after the interval: still nothing.
        assert_eq!(watch.tick(Some(&path), false, start + INTERVAL).1, None);

        // Changed, with nothing unsaved here: reload.
        write(&path, "second, longer");
        assert_eq!(watch.tick(Some(&path), false, start + INTERVAL * 2).1, Some(Found::Reload));
        // Taken as current, so it isn't reported twice.
        assert_eq!(watch.tick(Some(&path), false, start + INTERVAL * 3).1, None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unsaved_changes_here_mean_asking_first_and_only_once() {
        let dir = folder("ask");
        let path = dir.join("hutch.woodsketch");
        write(&path, "first");
        let mut watch = Watch::default();
        watch.stamped(Some(&path));
        let start = Instant::now();

        write(&path, "changed outside");
        assert_eq!(watch.tick(Some(&path), true, start + INTERVAL).1, Some(Found::Ask));
        // The question is asked once, not every tick while it's on screen.
        assert_eq!(watch.tick(Some(&path), true, start + INTERVAL * 2).1, None);
        assert_eq!(watch.tick(Some(&path), true, start + INTERVAL * 3).1, None);

        // Once answered, a later change is found again.
        watch.settled(Some(&path));
        write(&path, "changed outside again, longer");
        assert_eq!(watch.tick(Some(&path), false, start + INTERVAL * 5).1, Some(Found::Reload));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_unsaved_drawing_and_a_missing_file_are_left_alone() {
        let mut watch = Watch::default();
        let now = Instant::now();
        // Never saved: nothing to watch, and no repaints asked for.
        assert_eq!(watch.tick(None, true, now + INTERVAL * 2), (None, None));

        let dir = folder("missing");
        let path = dir.join("gone.woodsketch");
        write(&path, "here for now");
        watch.stamped(Some(&path));
        std::fs::remove_file(&path).unwrap();
        // Deleted (or mid-rename): nothing to reload, and no error either.
        assert_eq!(watch.tick(Some(&path), false, now + INTERVAL * 2).1, None);
        // It comes back (the rename landed) and is reloaded.
        write(&path, "back, and different");
        assert_eq!(watch.tick(Some(&path), false, now + INTERVAL * 4).1, Some(Found::Reload));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
