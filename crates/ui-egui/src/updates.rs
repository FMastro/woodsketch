//! Keeping WoodSketch up to date. The app binary plugs in the actual updater (Velopack, checking
//! GitHub Releases); this side holds what it found and shows it: a quiet check at startup
//! (unless turned off), "Restart to update" once a new version is downloaded, and Help > Check
//! for Updates to ask on demand.

use std::sync::{Arc, Mutex};

/// What the updater is doing, or found.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    /// Run from a build, not installed: there's nothing to update in place.
    NotInstalled,
    Checking,
    UpToDate,
    /// Downloading this version.
    Downloading(String),
    /// This version is downloaded and installs on restart.
    Ready(String),
    Failed(String),
}

/// The updater the app binary provides.
/// Install the downloaded update and restart, reopening the drawing at the path given.
pub type Restart = Box<dyn Fn(Option<&std::path::Path>) + Send + Sync>;

pub struct Updater {
    pub state: Arc<Mutex<UpdateState>>,
    /// Check for an update in the background, downloading it if there is one.
    pub check: Box<dyn Fn() + Send + Sync>,
    /// Install the downloaded update and restart (the process exits), reopening the drawing at
    /// this path, if any.
    pub restart: Restart,
}

impl Updater {
    pub fn state(&self) -> UpdateState {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

/// The running version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What to say about an update for a check the user asked for (startup checks only speak up
/// when an update is ready).
pub fn describe(state: &UpdateState) -> String {
    match state {
        UpdateState::Idle | UpdateState::Checking => "Checking for updates...".into(),
        UpdateState::NotInstalled => "This copy wasn't installed with the WoodSketch installer, so it can't update itself. Download the latest from woodsketch.net.".into(),
        UpdateState::UpToDate => format!("WoodSketch {VERSION} is up to date."),
        UpdateState::Downloading(v) => format!("WoodSketch {v} is downloading in the background (shown at the bottom right). When it's ready, a green \"Restart to update\" button appears there."),
        UpdateState::Ready(v) => format!("WoodSketch {v} is ready: restart to update."),
        UpdateState::Failed(e) => format!("Couldn't check for updates: {e}"),
    }
}

/// Every version's release notes, on GitHub.
pub const RELEASES_URL: &str = "https://github.com/FMastro/woodsketch/releases";

/// Where to ask how to do something, suggest what to build next, or show a piece.
pub const DISCUSSIONS_URL: &str = "https://github.com/FMastro/woodsketch/discussions";

/// Everything reported or suggested so far, open and fixed: worth a look before reporting.
pub const ISSUES_URL: &str = "https://github.com/FMastro/woodsketch/issues";

/// The release notes for `version`.
pub fn notes_url(version: &str) -> String {
    format!("{RELEASES_URL}/tag/v{version}")
}

/// Every version's notes, built into the app so What's New works offline. Small (tens of KB)
/// and the file the release workflow already reads, so there is one source of release notes.
pub const CHANGES: &str = include_str!("../../../CHANGELOG.md");

/// How many versions What's New shows when it can't tell which one ran last.
const RECENT: usize = 10;

/// One version's notes, as `CHANGELOG.md` has them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    /// Its notes, without the heading; the lines as written (`- ` bullets, wrapped).
    pub notes: String,
}

/// Every version in `changes`, newest first. A `## <version>` heading starts each one, which is
/// the shape `CHANGELOG.md` promises and the release workflow relies on.
pub fn releases(changes: &str) -> Vec<Release> {
    let mut found: Vec<Release> = Vec::new();
    for line in changes.lines() {
        if let Some(version) = line.strip_prefix("## ") {
            found.push(Release { version: version.trim().to_owned(), notes: String::new() });
        } else if let Some(current) = found.last_mut() {
            current.notes.push_str(line);
            current.notes.push('\n');
        }
    }
    for release in &mut found {
        release.notes = release.notes.trim().to_owned();
    }
    found
}

/// The versions worth showing: everything newer than `ran_last`, so someone who skipped four
/// updates sees all four. `None` (a first run, or a version too old to be in the notes) shows
/// the [`RECENT`] newest instead of nothing.
pub fn since<'a>(releases: &'a [Release], ran_last: Option<&str>) -> &'a [Release] {
    match ran_last.and_then(|last| releases.iter().position(|r| r.version == last)) {
        // The versions above the one that ran: what this update brought.
        Some(0) => &[],
        Some(at) => &releases[..at],
        None => &releases[..releases.len().min(RECENT)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# What's new

Blurb.

## 0.1.3

- Third.

## 0.1.2

- Second,
  wrapped.

## 0.1.1

- First.
";

    #[test]
    fn versions_are_read_newest_first_with_their_notes() {
        let found = releases(SAMPLE);
        assert_eq!(found.iter().map(|r| r.version.as_str()).collect::<Vec<_>>(), ["0.1.3", "0.1.2", "0.1.1"]);
        assert_eq!(found[0].notes, "- Third.");
        assert_eq!(found[1].notes, "- Second,
  wrapped.", "wrapped lines stay together");
        // The blurb above the first version isn't a release.
        assert!(!found.iter().any(|r| r.notes.contains("Blurb")));
    }

    #[test]
    fn what_to_show_depends_on_the_version_that_ran_last() {
        let found = releases(SAMPLE);
        // Skipped two updates: both are shown.
        let since_oldest = since(&found, Some("0.1.1"));
        assert_eq!(since_oldest.iter().map(|r| r.version.as_str()).collect::<Vec<_>>(), ["0.1.3", "0.1.2"]);
        // Already on the newest: nothing is new.
        assert!(since(&found, Some("0.1.3")).is_empty());
        // No idea what ran last, or it's older than the notes go: show the recent ones.
        assert_eq!(since(&found, None).len(), 3);
        assert_eq!(since(&found, Some("0.0.1")).len(), 3);
    }

    #[test]
    fn the_real_changelog_parses_and_starts_at_this_version() {
        let found = releases(CHANGES);
        assert!(found.len() > 20, "every version is there: {}", found.len());
        assert_eq!(found[0].version, VERSION, "the newest section is the running version");
        assert!(found.iter().all(|r| !r.notes.is_empty()), "no version has empty notes");
        assert!(found.iter().all(|r| r.version.starts_with(|c: char| c.is_ascii_digit())), "headings are versions");
    }
}
