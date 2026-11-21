// No console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use eframe::egui;
use woodsketch_ui_egui::{WoodSketchApp, graphics};

// Windows only: Velopack's install hooks, and the registry, exist nowhere else.
#[cfg(windows)]
mod fileassoc;
mod releases;
mod update;

fn main() -> eframe::Result {
    // First: the installer and updater run their setup steps through the app, and exit there.
    // Double-clicking a `.woodsketch` drawing opens it, which Velopack has no setting for, so the
    // association is written here: after an install, again after an update in case the app moved,
    // and taken out before an uninstall. These hooks only run on Windows; macOS and Linux need
    // their own machinery and are still on the backlog.
    // The hook methods themselves are `#[cfg(target_os = "windows")]` in Velopack, so this is
    // written twice rather than relying on them being no-ops elsewhere: on macOS and Linux they
    // do not exist at all.
    #[cfg(windows)]
    let mut installer = velopack::VelopackApp::build()
        .on_after_install_fast_callback(|_| fileassoc::register())
        .on_after_update_fast_callback(|_| fileassoc::register())
        .on_before_uninstall_fast_callback(|_| fileassoc::unregister());
    #[cfg(not(windows))]
    let mut installer = velopack::VelopackApp::build();
    installer.run();
    env_logger::init();
    // A crash is saved to the settings folder, and the next start offers to report it.
    let dir = eframe::storage_dir("WoodSketch");
    if let Some(dir) = dir.clone() {
        woodsketch_ui_egui::crash::install(dir);
    }
    // And a note that a run is under way, which is the only thing that can be left behind by a
    // failure no Rust code survives: a graphics driver taking the process down, running out of
    // memory, a stack overflow. It is taken away below, once the window has closed properly.
    if let Some(dir) = &dir {
        woodsketch_ui_egui::crash::running(dir);
    }
    // `woodsketch drawing.woodsketch` opens that drawing.
    let open = std::env::args_os().nth(1).map(PathBuf::from);
    // Which backend to draw with, rather than whichever wgpu happens to pick (#13). The list is
    // never empty, and its first entry is the preferred backend on all but a machine that has
    // shown it doesn't work.
    let result = draw(open.as_deref(), dir.as_deref());
    // The window has closed, so this run ended the way it was meant to and isn't a crash to report
    // at the next start. A panic has already been saved by the hook, and is offered on its own.
    if let Some(dir) = &dir {
        woodsketch_ui_egui::crash::ended(dir);
    }
    result
}

/// Open the window on the first backend that will have it.
fn draw(open: Option<&Path>, dir: Option<&Path>) -> eframe::Result {
    let plan = graphics::plan(dir);
    let mut after = plan.after;
    let (last, rest) = plan.attempts.split_last().expect("there is always a backend to try");
    for backend in rest {
        match run(*backend, after.as_deref(), open, dir) {
            // This one can't start here. Try the next now, in this same run: a backend that
            // isn't there is no reason to leave somebody with an app that won't open.
            Err(eframe::Error::Wgpu(error)) => {
                log::warn!("{} couldn't start ({error})", backend.map_or("the graphics", graphics::said));
                after = backend.map(str::to_owned);
            }
            result => return result,
        }
    }
    run(*last, after.as_deref(), open, dir)
}

/// Open the window and run until it closes, drawing with `backend` where one was chosen.
fn run(backend: Option<&str>, after: Option<&str>, open: Option<&Path>, dir: Option<&Path>) -> eframe::Result {
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WoodSketch")
            // What the window calls itself to the desktop (Wayland's app id, X11's WM_CLASS).
            // It has to match `StartupWMClass` in `packaging/linux/woodsketch.desktop`, or Linux
            // can't tell that this window is that entry's app, and shows it in the taskbar
            // without its name or its icon.
            .with_app_id("woodsketch")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 400.0])
            .with_maximized(true)
            .with_icon(std::sync::Arc::new(eframe::icon_data::from_png_bytes(include_bytes!("../../../assets/icon/woodsketch-256.png")).unwrap_or_default())),
        renderer: eframe::Renderer::Wgpu,
        // Open maximized (as SketchUp does) and centered when restored, so the window never
        // starts partly off a scaled-up screen.
        centered: true,
        // Settings (recent files) are kept between runs, but not the window: it always opens
        // maximized and centered.
        persist_window: false,
        ..Default::default()
    };
    graphics::draw_with(&mut options, backend);
    // Written down before the window opens, not after: a driver that takes the whole process
    // down while wgpu is starting it leaves nothing behind otherwise, and the next run would
    // choose the same backend and go the same way.
    graphics::trying(dir, backend);
    let (open, after) = (open.map(Path::to_owned), after.map(str::to_owned));
    // Owned for the closure, which outlives this call: the settings folder, so applying an update
    // can say the run ended before it ends the process.
    let settings = dir.map(Path::to_owned);
    eframe::run_native(
        "WoodSketch",
        options,
        Box::new(move |cc| {
            let mut app = WoodSketchApp::new(cc);
            app.set_updater(update::updater(cc.egui_ctx.clone(), settings.clone()));
            if let Some(failed) = &after {
                app.fell_back_from(failed);
            }
            if let Some(path) = open
                && let Err(error) = app.open_path(&path)
            {
                app.show_error(error);
            }
            Ok(Box::new(app))
        }),
    )
}
