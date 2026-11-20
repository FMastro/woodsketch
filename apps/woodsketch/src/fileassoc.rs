//! Making Windows open a `.woodsketch` drawing when you double-click it.
//!
//! Velopack has nothing for this (no flag on any platform), so the app does it itself from the
//! install hooks: [`register`] after an install or an update, [`unregister`] before an uninstall.
//! The hooks only fire on Windows, and everywhere else these are no-ops; macOS and Linux want
//! quite different machinery (a bundle's `CFBundleDocumentTypes`, a `.desktop` file and a MIME
//! database) and are their own backlog items.
//!
//! Everything goes under `HKEY_CURRENT_USER`, never `HKEY_LOCAL_MACHINE`: it is this user's choice
//! about this user's app, it needs no administrator, and an uninstall can undo it cleanly.
//!
//! **A new association can take a while to show up in File Explorer.** Telling the shell at once
//! means `SHChangeNotify`, which is a Win32 call, and `unsafe_code` is forbidden across this
//! workspace. `.woodsketch` is a brand-new extension with nothing else claiming it, so Explorer
//! picks it up for new windows without being told; it is the *changing* of an existing
//! association that it caches hard. Worth revisiting if it proves annoying in practice.

/// Where the association is written. `Software\Classes` is the real one; tests pass their own
/// so that running them never touches what the machine actually opens.
#[cfg(windows)]
const CLASSES: &str = r"Software\Classes";

/// The extension WoodSketch claims. Not `.scraft`: that name belongs to the Craft apps this was
/// renamed from, and taking it over would be claiming someone else's files.
#[cfg(windows)]
const EXTENSION: &str = ".woodsketch";

/// The program id the extension points at, and which carries the icon and the open command.
#[cfg(windows)]
const PROG_ID: &str = "WoodSketch.Drawing";

/// Make Windows open `.woodsketch` files with this build. Safe to run again: every write replaces
/// what was there, so an update that moved the app simply points at the new place.
pub fn register() {
    #[cfg(windows)]
    if let Err(e) = windows_impl::write(CLASSES, &exe_path()) {
        log::warn!("couldn't register the .woodsketch file association: {e}");
    }
}

/// Take it back out, before the app is uninstalled.
pub fn unregister() {
    #[cfg(windows)]
    if let Err(e) = windows_impl::remove(CLASSES) {
        log::warn!("couldn't remove the .woodsketch file association: {e}");
    }
}

/// The program to open drawings with: this one, wherever it was installed. Velopack keeps the
/// running build in a `current` folder whose name doesn't change between versions, so what is
/// written here survives updates and only the contents of that folder are swapped.
#[cfg(windows)]
fn exe_path() -> String {
    std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default()
}

#[cfg(windows)]
mod windows_impl {
    use std::io;

    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};

    use super::{EXTENSION, PROG_ID};

    /// Write the association under `classes` for the program at `exe`.
    pub(super) fn write(classes: &str, exe: &str) -> io::Result<()> {
        if exe.is_empty() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "there is no path to this program"));
        }
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        // The extension points at the program id, which is where everything else hangs. The
        // extension's own key holds nothing but that, so another program taking the file type
        // over later replaces one value rather than having to unpick ours.
        let (ext, _) = hkcu.create_subkey(format!(r"{classes}\{EXTENSION}"))?;
        ext.set_value("", &PROG_ID)?;
        // Listing it here as well is what puts WoodSketch in "Open with" even when something else
        // holds the extension.
        let (with, _) = hkcu.create_subkey(format!(r"{classes}\{EXTENSION}\OpenWithProgids"))?;
        with.set_value(PROG_ID, &"")?;

        let (prog, _) = hkcu.create_subkey(format!(r"{classes}\{PROG_ID}"))?;
        prog.set_value("", &"WoodSketch Drawing")?;
        let (icon, _) = hkcu.create_subkey(format!(r"{classes}\{PROG_ID}\DefaultIcon"))?;
        // The first icon in the program itself, so the file gets the app's own picture.
        icon.set_value("", &format!("{exe},0"))?;
        let (command, _) = hkcu.create_subkey(format!(r"{classes}\{PROG_ID}\shell\open\command"))?;
        // Quoted, both of them: an installed path has spaces in it and so will the drawing.
        command.set_value("", &format!("\"{exe}\" \"%1\""))?;
        Ok(())
    }

    /// Take the association out again, leaving alone anything that isn't ours.
    pub(super) fn remove(classes: &str) -> io::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        // The extension only goes if it still points at us. Another program may have taken the
        // file type over since, and uninstalling WoodSketch is no reason to break that.
        let ours = hkcu
            .open_subkey_with_flags(format!(r"{classes}\{EXTENSION}"), KEY_READ)
            .ok()
            .and_then(|k| k.get_value::<String, _>("").ok())
            .is_some_and(|v| v == PROG_ID);
        if ours {
            let _ = hkcu.delete_subkey_all(format!(r"{classes}\{EXTENSION}"));
        } else if let Ok(with) = hkcu.open_subkey_with_flags(format!(r"{classes}\{EXTENSION}\OpenWithProgids"), KEY_WRITE) {
            // Not ours any more, so only our line in the "Open with" list goes.
            let _ = with.delete_value(PROG_ID);
        }
        // What hangs off the program id is all ours, so it goes whatever happened to the extension.
        let _ = hkcu.delete_subkey_all(format!(r"{classes}\{PROG_ID}"));
        Ok(())
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    /// A scratch place in the registry for one test, cleaned up after. Never `Software\Classes`:
    /// running the tests must not change what this machine actually opens.
    struct Scratch(String);

    impl Scratch {
        fn new(name: &str) -> Self {
            let path = format!(r"Software\WoodSketch\tests\{name}-{}", std::process::id());
            let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&path);
            Self(path)
        }
        fn value(&self, key: &str) -> Option<String> {
            RegKey::predef(HKEY_CURRENT_USER).open_subkey(format!(r"{}\{key}", self.0)).ok()?.get_value("").ok()
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.0);
        }
    }

    #[test]
    fn registering_points_the_extension_at_this_program() {
        let at = Scratch::new("register");
        let exe = r"C:\Program Files\A Folder\woodsketch.exe";
        windows_impl::write(&at.0, exe).expect("written");
        assert_eq!(at.value(EXTENSION).as_deref(), Some(PROG_ID), "the extension names the program id");
        assert_eq!(at.value(PROG_ID).as_deref(), Some("WoodSketch Drawing"), "which has a readable name");
        assert_eq!(at.value(&format!(r"{PROG_ID}\DefaultIcon")).as_deref(), Some(format!("{exe},0").as_str()));
        // Both quoted: the program's path has spaces in it, and so will the drawing's.
        assert_eq!(
            at.value(&format!(r"{PROG_ID}\shell\open\command")).as_deref(),
            Some(format!("\"{exe}\" \"%1\"").as_str()),
            "the command opens the file that was double-clicked"
        );
        // Listed for "Open with" as well.
        let with = RegKey::predef(HKEY_CURRENT_USER).open_subkey(format!(r"{}\{EXTENSION}\OpenWithProgids", at.0)).expect("the list");
        assert!(with.get_value::<String, _>(PROG_ID).is_ok());
    }

    #[test]
    fn registering_again_moves_it_rather_than_leaving_the_old_path() {
        let at = Scratch::new("again");
        windows_impl::write(&at.0, r"C:\Old\woodsketch.exe").expect("written");
        windows_impl::write(&at.0, r"C:\New\woodsketch.exe").expect("written again");
        let command = at.value(&format!(r"{PROG_ID}\shell\open\command")).expect("a command");
        assert!(command.contains(r"C:\New\woodsketch.exe") && !command.contains(r"C:\Old"), "{command}");
    }

    #[test]
    fn a_program_with_no_path_is_refused_rather_than_written_empty() {
        // Writing `"" "%1"` would give a file type that opens nothing, which is worse than none.
        let at = Scratch::new("empty");
        assert!(windows_impl::write(&at.0, "").is_err());
        assert_eq!(at.value(EXTENSION), None, "and nothing was written");
    }

    #[test]
    fn unregistering_takes_ours_out() {
        let at = Scratch::new("remove");
        windows_impl::write(&at.0, r"C:\Program Files\WoodSketch\woodsketch.exe").expect("written");
        windows_impl::remove(&at.0).expect("removed");
        assert_eq!(at.value(EXTENSION), None, "the extension is gone");
        assert_eq!(at.value(PROG_ID), None, "and so is the program id");
    }

    #[test]
    fn unregistering_leaves_a_file_type_someone_else_has_taken_over() {
        // Uninstalling WoodSketch is no reason to break whatever opens the file now.
        let at = Scratch::new("taken");
        windows_impl::write(&at.0, r"C:\Program Files\WoodSketch\woodsketch.exe").expect("written");
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (ext, _) = hkcu.create_subkey(format!(r"{}\{EXTENSION}", at.0)).expect("the extension");
        ext.set_value("", &"SomeoneElse.Drawing").expect("taken over");
        windows_impl::remove(&at.0).expect("removed");
        assert_eq!(at.value(EXTENSION).as_deref(), Some("SomeoneElse.Drawing"), "their claim stands");
        assert_eq!(at.value(PROG_ID), None, "ours is still cleaned up");
        // And our line in the "Open with" list is gone, since the app is going.
        let with = hkcu.open_subkey(format!(r"{}\{EXTENSION}\OpenWithProgids", at.0)).expect("the list");
        assert!(with.get_value::<String, _>(PROG_ID).is_err(), "no longer offered");
    }
}
