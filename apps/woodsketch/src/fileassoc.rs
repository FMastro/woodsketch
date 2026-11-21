//! Making Windows open a `.woodsketch` drawing when you double-click it.
//!
//! Velopack has nothing for this (no flag on any platform), so the app does it itself from the
//! install hooks: [`register`] after an install or an update, [`unregister`] before an uninstall.
//! Velopack's hook methods are themselves Windows-only, so `main.rs` compiles this whole module
//! in only there; macOS and Linux want quite different machinery (a bundle's
//! `CFBundleDocumentTypes`, a `.desktop` file and a MIME database) and are their own backlog
//! items.
//!
//! Everything goes under `HKEY_CURRENT_USER`, never `HKEY_LOCAL_MACHINE`: it is this user's choice
//! about this user's app, it needs no administrator, and an uninstall can undo it cleanly.
//!
//! The shell is told when a file type changes, by [`tell_the_shell`], so Explorer doesn't go on
//! showing the old answer. Not with `SHChangeNotify`, the documented call, which every Rust
//! binding offers as an `unsafe fn` that `unsafe_code = "forbid"` rules out here -- but by asking
//! Windows to do it, which needs no `unsafe` at all. That is insurance rather than a cure: a
//! brand-new extension nothing else claims is picked up without being told anything, which 0.1.85
//! proved by opening a drawing on a double-click straight after installing. What it covers is
//! what that cannot -- writing over a type another program has taken, or an update that lands
//! somewhere new, which is the case Explorer caches hard.

/// Where the association is written. `Software\Classes` is the real one; tests pass their own
/// so that running them never touches what the machine actually opens.
const CLASSES: &str = r"Software\Classes";

/// The extension WoodSketch claims. Not `.scraft`: that name belongs to the Craft apps this was
/// renamed from, and taking it over would be claiming someone else's files.
const EXTENSION: &str = ".woodsketch";

/// The program id the extension points at, and which carries the icon and the open command.
const PROG_ID: &str = "WoodSketch.Drawing";

/// Make Windows open `.woodsketch` files with this build. Safe to run again: every write replaces
/// what was there, so an update that moved the app simply points at the new place.
pub fn register() {
    if let Err(e) = windows_impl::write(CLASSES, &exe_path()) {
        log::warn!("couldn't register the .woodsketch file association: {e}");
        return;
    }
    tell_the_shell(SHELL_REFRESH);
}

/// Take it back out, before the app is uninstalled.
pub fn unregister() {
    if let Err(e) = windows_impl::remove(CLASSES) {
        log::warn!("couldn't remove the .woodsketch file association: {e}");
        return;
    }
    tell_the_shell(SHELL_REFRESH);
}

/// What asks the shell to read file types and icons again. Ships with Windows itself.
const SHELL_REFRESH: &str = "ie4uinit.exe";

/// Ask the shell to look at the file types again, so Explorer doesn't keep answering from what it
/// remembers.
///
/// `SHChangeNotify(SHCNE_ASSOCCHANGED, ...)` is the documented way and is an `unsafe fn` in every
/// binding there is; `forbid` cannot be lifted by an `allow` in one module, and nothing wraps that
/// call safely (winsafe, the one library worth looking in, has no such function). Starting a
/// program needs no `unsafe`, and `-show` is what makes `ie4uinit.exe` re-read associations and
/// icon caches.
///
/// Best effort on purpose. Nothing is waited for -- this runs from the installer, which shouldn't
/// be held up by a cache refresh -- and a failure is logged and no more, since the association is
/// written either way and Explorer comes round on its own. `program` is a parameter so a test can
/// watch that failure happen without touching the real shell.
fn tell_the_shell(program: &str) {
    use std::os::windows::process::CommandExt;
    // Started from an installer, where a flash of console window is all anyone would see of it.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    if let Err(e) = std::process::Command::new(program).arg("-show").creation_flags(CREATE_NO_WINDOW).spawn() {
        log::warn!("couldn't ask the shell to re-read the file types ({program}): {e}");
    }
}

/// The program to open drawings with: this one, wherever it was installed. Velopack keeps the
/// running build in a `current` folder whose name doesn't change between versions, so what is
/// written here survives updates and only the contents of that folder are swapped.
fn exe_path() -> String {
    std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default()
}

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

#[cfg(test)]
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

    /// The nudge is best effort: on a Windows where that program is missing or renamed, the
    /// association is still written and Explorer catches up by itself, so this must not panic or
    /// hand an error back to an installer.
    #[test]
    fn a_missing_shell_tool_doesnt_take_the_registration_down_with_it() {
        tell_the_shell("woodsketch-there-is-no-such-program.exe");
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
