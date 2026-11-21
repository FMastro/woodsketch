//! Updates through Velopack, from WoodSketch's GitHub Releases: a background check that
//! downloads a newer version, then "Restart to update" applies it.

use std::sync::{Arc, Mutex};

use eframe::egui;
use velopack::{UpdateCheck, UpdateInfo, UpdateManager};
use woodsketch_ui_egui::updates::{UpdateState, Updater};

use crate::releases::Releases;

pub const REPO: &str = "https://github.com/FMastro/woodsketch";

/// The update found and downloaded, ready to apply.
type Pending = Arc<Mutex<Option<(UpdateManager, UpdateInfo)>>>;

/// `dir` is the settings folder, where the note that a run is under way lives: applying an update
/// ends this process where nothing of ours runs again, which is indistinguishable from a crash
/// unless the note is taken away first.
pub fn updater(ctx: egui::Context, dir: Option<std::path::PathBuf>) -> Updater {
    let state = Arc::new(Mutex::new(UpdateState::Idle));
    let pending: Pending = Arc::new(Mutex::new(None));

    let check = {
        let (state, pending) = (state.clone(), pending.clone());
        move || {
            let busy = matches!(*state.lock().unwrap_or_else(|e| e.into_inner()), UpdateState::Checking | UpdateState::Downloading(_));
            if busy {
                return;
            }
            let (state, pending, ctx) = (state.clone(), pending.clone(), ctx.clone());
            std::thread::spawn(move || {
                let set = |s: UpdateState| {
                    *state.lock().unwrap_or_else(|e| e.into_inner()) = s;
                    ctx.request_repaint();
                };
                set(UpdateState::Checking);
                // Not installed with the installer (run from a build): nothing to update.
                let Ok(manager) = UpdateManager::new(Releases::at(REPO), None, None) else {
                    set(UpdateState::NotInstalled);
                    return;
                };
                match manager.check_for_updates() {
                    Ok(UpdateCheck::UpdateAvailable(info)) => {
                        let version = info.TargetFullRelease.Version.clone();
                        set(UpdateState::Downloading(version.clone()));
                        match manager.download_updates(&info, None) {
                            Ok(()) => {
                                *pending.lock().unwrap_or_else(|e| e.into_inner()) = Some((manager, *info));
                                set(UpdateState::Ready(version));
                            }
                            Err(e) => set(UpdateState::Failed(e.to_string())),
                        }
                    }
                    Ok(_) => set(UpdateState::UpToDate),
                    Err(e) => set(UpdateState::Failed(e.to_string())),
                }
            });
        }
    };

    // The new version starts with the drawing that was open, as `woodsketch <file>` does.
    let restart = move |open: Option<&std::path::Path>| {
        let args: Vec<&std::ffi::OsStr> = open.map(std::path::Path::as_os_str).into_iter().collect();
        // Velopack replaces the app and starts the new one, so this process never returns from the
        // call below. Saying the run ended here is what keeps an update from being reported as a
        // crash by the version it installed.
        if let Some(dir) = &dir {
            woodsketch_ui_egui::crash::ended(dir);
        }
        if let Some((manager, info)) = pending.lock().unwrap_or_else(|e| e.into_inner()).take()
            && let Err(e) = manager.apply_updates_and_restart_with_args(&info, args)
        {
            log::error!("couldn't apply the update: {e}");
        }
    };

    Updater { state, check: Box::new(check), restart: Box::new(restart) }
}
