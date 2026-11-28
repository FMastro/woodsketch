//! Autosave and crash recovery. While a drawing has unsaved changes, a copy is written to
//! WoodSketch's settings folder (not next to the drawing, so synced folders don't fill with
//! temporary files) [`SETTLE`] after the last change, and every [`MOST`] through editing that
//! never lets up. Saving, opening, starting a new drawing or closing normally removes it; one
//! still there when WoodSketch starts means it didn't close properly, and is offered back.
//!
//! It used to be written every two minutes, which is two minutes of somebody's work to lose.
//! Three crash reports in a week made the case for closing that gap. One was a panic, where the
//! hook in `crash.rs` at least gets to write a report; two were quiet stops, the process gone
//! with no Rust code of ours running again -- and for those a recent copy on disk is the only
//! protection there can be, since there is no hook to install and nothing to save from. Writing
//! the biggest drawing here (the Small house, 2.4 MB) takes 8 ms, so writing it this often costs
//! nothing anybody would miss.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use woodsketch_doc::Document;
use woodsketch_doc::file;
use woodsketch_i18n::t;

/// How long after the last change the drawing is written, once the hand has come off it.
pub const SETTLE: Duration = Duration::from_secs(5);
/// How long an unbroken run of changes may go unwritten. The settle clock alone would keep
/// putting the write off for as long as somebody kept dragging something about.
pub const MOST: Duration = Duration::from_secs(30);

/// A drawing autosaved by a session that didn't close properly.
#[derive(Clone, Debug, PartialEq)]
pub struct Leftover {
    /// The autosaved copy.
    pub file: PathBuf,
    /// The drawing's own file, if it had been saved before.
    pub original: Option<PathBuf>,
    pub when: SystemTime,
}

impl Leftover {
    /// What the drawing was called: its file's name, or "Untitled".
    pub fn name(&self) -> String {
        self.original.as_deref().and_then(Path::file_stem).map_or_else(|| t!("files-untitled"), |s| s.to_string_lossy().into_owned())
    }

    /// Delete the autosaved copy (recovered or not wanted).
    pub fn discard(&self) {
        let _ = std::fs::remove_file(&self.file);
        let _ = std::fs::remove_file(self.file.with_extension("path"));
    }
}

pub struct Autosave {
    /// Where autosaves go; `None` turns autosave off (tests).
    dir: Option<PathBuf>,
    /// The model version last written, so an unchanged drawing isn't written again.
    written_version: Option<u64>,
    /// The change waiting to be written: which version it is, and when it arrived.
    waiting: Option<(u64, Instant)>,
    /// When the oldest change not yet written arrived, which is what [`MOST`] is measured from.
    since: Option<Instant>,
}

impl Autosave {
    pub fn new(dir: Option<PathBuf>) -> Self {
        Self { dir, written_version: None, waiting: None, since: None }
    }

    /// This session's autosave file.
    fn file(&self) -> Option<PathBuf> {
        Some(self.dir.as_ref()?.join(format!("autosave-{}.woodsketch", std::process::id())))
    }

    /// Write the drawing if it has unsaved changes and the editing has settled -- or has gone on
    /// for [`MOST`] without settling. Returns how long until it should be looked at again (to
    /// schedule a repaint), if ever.
    pub fn tick(&mut self, doc: &Document, original: Option<&Path>, dirty: bool, now: Instant) -> Result<Option<Duration>, String> {
        let version = doc.model_version();
        if self.dir.is_none() || !dirty || self.written_version == Some(version) {
            // Nothing is waiting, so both clocks start again with whatever changes next.
            self.waiting = None;
            self.since = None;
            return Ok(None);
        }
        // A change nobody has written yet. Its own clock restarts each time the drawing changes
        // again, so the write lands when the hand comes off rather than in the middle of a drag;
        // `since` is the answer to a hand that never comes off.
        if self.waiting.is_none_or(|(v, _)| v != version) {
            self.waiting = Some((version, now));
        }
        let settled = self.waiting.expect("set just above unless it was already this version").1 + SETTLE;
        let capped = *self.since.get_or_insert(now) + MOST;
        let due = settled.min(capped);
        if now < due {
            return Ok(Some(due - now));
        }
        self.write(doc, original)?;
        self.since = None;
        Ok(None)
    }

    /// Write the drawing now.
    pub fn write(&mut self, doc: &Document, original: Option<&Path>) -> Result<(), String> {
        let Some(path) = self.file() else { return Ok(()) };
        let dir = self.dir.as_ref().expect("a file needs a folder");
        std::fs::create_dir_all(dir).map_err(|e| t!("app-autosave-failed", "error" => e.to_string()))?;
        write_safely(&path, file::to_string(doc.scene())).map_err(|e| t!("app-autosave-failed", "error" => e.to_string()))?;
        let note = original.map_or_else(String::new, |p| p.to_string_lossy().into_owned());
        write_safely(&path.with_extension("path"), &note).map_err(|e| t!("app-autosave-failed", "error" => e.to_string()))?;
        self.written_version = Some(doc.model_version());
        Ok(())
    }

    /// The drawing was saved, replaced or closed: its autosave isn't needed.
    pub fn clear(&mut self) {
        if let Some(path) = self.file() {
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(path.with_extension("path"));
        }
        self.written_version = None;
        self.waiting = None;
        self.since = None;
    }

    /// Autosaves left by sessions that didn't close properly, newest first.
    pub fn leftovers(&self) -> Vec<Leftover> {
        let Some(dir) = &self.dir else { return Vec::new() };
        let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
        let mine = self.file();
        let mut found: Vec<Leftover> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "woodsketch") && Some(p) != mine.as_ref())
            .map(|file| {
                let original = std::fs::read_to_string(file.with_extension("path")).ok().filter(|s| !s.trim().is_empty()).map(|s| PathBuf::from(s.trim()));
                let when = std::fs::metadata(&file).and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
                Leftover { file, original, when }
            })
            .collect();
        found.sort_by_key(|l| std::cmp::Reverse(l.when));
        found
    }
}

/// Write `text` to `path` whole or not at all; the drawing format's own writer, so the app and
/// `woodsketch-cli` write files the same way.
pub use woodsketch_doc::file::write_safely;

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("woodsketch-autosave-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn drawing() -> Document {
        let mut doc = Document::default();
        doc.edit(|m| m.add_rectangle(glam::DVec3::ZERO, glam::DVec3::X, glam::DVec3::Y, glam::DVec3::Z));
        doc
    }

    #[test]
    fn unsaved_changes_are_written_once_the_editing_settles_and_cleared_on_save() {
        let dir = folder("tick");
        let mut autosave = Autosave::new(Some(dir.clone()));
        let doc = drawing();
        let start = Instant::now();
        let original = Path::new("C:/Shop/Nightstand.woodsketch");
        assert!(autosave.tick(&doc, Some(original), true, start).unwrap().is_some(), "not yet");
        assert!(autosave.file().is_some_and(|f| !f.exists()));
        autosave.tick(&doc, Some(original), true, start + SETTLE).unwrap();
        let file = autosave.file().unwrap();
        assert!(file.exists(), "written once the editing settled");
        assert_eq!(autosave.tick(&doc, Some(original), true, start + SETTLE * 3).unwrap(), None, "unchanged: not again");

        // Another session (a new start after a crash) finds it, with the drawing's own file.
        let next = Autosave { dir: Some(dir.clone()), written_version: None, waiting: None, since: None };
        let mut leftovers = next.leftovers();
        assert!(leftovers.is_empty(), "this session's own file isn't a leftover");
        std::fs::rename(&file, dir.join("autosave-1.woodsketch")).unwrap();
        std::fs::rename(file.with_extension("path"), dir.join("autosave-1.path")).unwrap();
        leftovers = next.leftovers();
        assert_eq!(leftovers.len(), 1);
        assert_eq!(leftovers[0].original.as_deref(), Some(original));
        assert_eq!(leftovers[0].name(), "Nightstand");
        let recovered = crate::files::Files::read(&leftovers[0].file).unwrap();
        assert_eq!(recovered.model().faces().count(), 1);
        leftovers[0].discard();
        assert!(next.leftovers().is_empty());

        autosave.write(&doc, None).unwrap();
        autosave.clear();
        assert!(!autosave.file().unwrap().exists(), "saving clears it");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Each change puts the write off a little, so a drag isn't written at every step -- and
    /// editing that never stops is written anyway, since how much work a crash can take with it
    /// is the whole point.
    #[test]
    fn a_change_puts_the_write_off_but_only_so_far() {
        let dir = folder("settle");
        let mut autosave = Autosave::new(Some(dir.clone()));
        let mut doc = drawing();
        let mut square = 1.0;
        let mut change = |doc: &mut Document| {
            square += 2.0;
            doc.edit(|m| m.add_rectangle(glam::DVec3::new(square, 0.0, 0.0), glam::DVec3::X, glam::DVec3::Y, glam::DVec3::Z));
        };

        // A change a second, as a drag would arrive. Nothing is written while it is still
        // going, because each change puts the write off again.
        let start = Instant::now();
        let mut now = start;
        let file = autosave.file().expect("a file");
        for _ in 0..3 {
            change(&mut doc);
            assert!(autosave.tick(&doc, None, true, now).unwrap().is_some(), "still waiting {:?} in", now - start);
            assert!(!file.exists(), "written {:?} in, while the editing was still going", now - start);
            now += Duration::from_secs(1);
        }
        // Keep it up and it is written anyway, inside a minute however long the editing lasts.
        // A number rather than `MOST`, so this says what the guarantee is instead of agreeing
        // with whatever the constant happens to be.
        let deadline = start + Duration::from_secs(60);
        while !file.exists() {
            assert!(now <= deadline, "editing that didn't let up went {:?} unwritten", now - start);
            change(&mut doc);
            autosave.tick(&doc, None, true, now).unwrap();
            now += Duration::from_secs(1);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn off_without_a_folder() {
        let mut autosave = Autosave::new(None);
        assert_eq!(autosave.tick(&drawing(), None, true, Instant::now() + MOST * 2).unwrap(), None);
        assert!(autosave.leftovers().is_empty());
    }

    #[test]
    fn safe_writes_replace_the_file_whole() {
        let dir = folder("safe");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.woodsketch");
        write_safely(&path, "first").unwrap();
        write_safely(&path, "second").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1, "no temporary file left");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
