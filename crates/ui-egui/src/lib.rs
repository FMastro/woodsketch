//! The WoodSketch window: a 3D viewport inside SketchUp-style chrome (`chrome.rs`).

mod chrome;
pub mod crash;
pub mod feedback;
mod cutlist;
pub mod files;
mod icons;
mod keys;
mod materials;
mod autosave;
mod watch;
mod board;
mod joinery;
mod outliner;
mod part;
mod profile;
mod tags;
mod template;
pub mod updates;
mod viewport;

use std::path::{Path, PathBuf};

use eframe::egui;
use woodsketch_doc::Document;
use woodsketch_tools::Tools;
use viewport::Viewport;

/// Where the recent-drawings list is kept in eframe's settings.
const RECENT_KEY: &str = "recent_drawings";
/// Whether to check for updates at startup, in eframe's settings.
const UPDATES_KEY: &str = "check_updates";
/// The version that ran last, to say "Updated to ..." once after an update.
const LAST_VERSION_KEY: &str = "last_version";
/// Inches or millimeters (View > Units).
const UNITS_KEY: &str = "units";
/// Settings key for View > Theme.
const THEME_KEY: &str = "theme";

pub struct WoodSketchApp {
    doc: Document,
    tools: Tools,
    keyboard: keys::Keyboard,
    tags: tags::TagsPanel,
    materials: materials::MaterialsPanel,
    viewport: Viewport,
    files: files::Files,
    cut_list: cutlist::CutListWindow,
    show_tray: bool,
    show_about: bool,
    /// Help > Report a Problem / Send Feedback.
    feedback: feedback::FeedbackWindow,
    /// The graphics card and driver, for a problem report.
    graphics: String,
    /// View > Theme, as last seen (saved with the settings).
    theme: egui::ThemePreference,
    /// Units for File > Export.
    export_units: woodsketch_doc::export::ExportUnits,
    /// Drawings opened or saved lately, newest first (File > Open Recent); kept between runs.
    recent: Vec<PathBuf>,
    /// Writes unsaved changes now and then, for recovery after a crash.
    autosave: autosave::Autosave,
    /// Drawings autosaved by a session that crashed, offered back at start.
    leftovers: Vec<autosave::Leftover>,
    /// Where a crash is saved (the settings folder; none in tests), and the one the last session
    /// left, offered for a report at start.
    crash_dir: Option<PathBuf>,
    crash: Option<woodsketch_doc::feedback::Crash>,
    /// Autosave failed once (shown once, not every interval).
    /// The updater the app binary plugs in (none in tests or builds without one).
    updater: Option<updates::Updater>,
    /// Check for updates at startup (a setting, kept between runs).
    check_updates: bool,
    /// This is the first run after an update: the status bar says so, with a link to what's new.
    /// The version that ran before this one, when it was a different one: What's New then shows
    /// everything between it and now, not just this version.
    updated_from: Option<String>,
    /// What's New is open.
    show_changes: bool,
    /// Whether the startup check has run, and whether the user asked for the last one.
    update_started: bool,
    update_asked: bool,
    autosave_failed: bool,
    /// Watches the open drawing's file, so changes made outside WoodSketch show up.
    watch: watch::Watch,
    /// The file changed outside and there are unsaved changes here: waiting on the answer.
    reload_asking: bool,
    /// Reloaded just now, to say so in the status bar for a moment.
    reloaded: Option<std::time::Instant>,
}

impl WoodSketchApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Inches or millimeters, as last chosen.
        if let Some(units) = cc.storage.and_then(|s| eframe::get_value(s, UNITS_KEY)) {
            woodsketch_doc::units::set_units(units);
        }
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("WoodSketch needs eframe's wgpu renderer");
        icons::install(&cc.egui_ctx);
        // Light and dark looks, and the one last chosen (View > Theme; light at first).
        cc.egui_ctx.set_visuals_of(egui::Theme::Light, chrome::visuals(false));
        cc.egui_ctx.set_visuals_of(egui::Theme::Dark, chrome::visuals(true));
        let theme = cc.storage.and_then(|s| eframe::get_value::<String>(s, THEME_KEY));
        let theme = chrome::THEMES.iter().find(|(_, _, key)| Some(*key) == theme.as_deref()).map_or(egui::ThemePreference::Light, |(t, _, _)| *t);
        cc.egui_ctx.set_theme(theme);
        // Autosave only for the real app (tests have no settings storage).
        let dir = cc.storage.is_some().then(|| eframe::storage_dir("WoodSketch")).flatten().map(|d| d.join("autosave"));
        let autosave = autosave::Autosave::new(dir);
        let leftovers = autosave.leftovers();
        let crash_dir = cc.storage.is_some().then(|| eframe::storage_dir("WoodSketch")).flatten();
        let crash = crash_dir.as_deref().and_then(crash::left);
        Self {
            doc: Document::default(),
            tools: Tools::default(),
            keyboard: keys::Keyboard::default(),
            tags: tags::TagsPanel::default(),
            materials: materials::MaterialsPanel::default(),
            viewport: Viewport::new(render_state),
            files: files::Files::default(),
            cut_list: cutlist::CutListWindow::default(),
            show_tray: true,
            show_about: false,
            feedback: feedback::FeedbackWindow::default(),
            theme,
            graphics: {
                let info = render_state.adapter.get_info();
                let driver = [info.driver.as_str(), info.driver_info.as_str()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
                format!("{} ({:?}{})", info.name, info.backend, if driver.is_empty() { String::new() } else { format!(", {driver}") })
            },
            export_units: woodsketch_doc::export::ExportUnits::Inches,
            recent: cc.storage.and_then(|s| eframe::get_value(s, RECENT_KEY)).unwrap_or_default(),
            autosave,
            leftovers,
            crash_dir,
            crash,
            autosave_failed: false,
            watch: watch::Watch::default(),
            reload_asking: false,
            reloaded: None,
            updater: None,
            check_updates: cc.storage.and_then(|s| eframe::get_value(s, UPDATES_KEY)).unwrap_or(true),
            updated_from: cc
                .storage
                .and_then(|s| eframe::get_value::<String>(s, LAST_VERSION_KEY))
                .filter(|last| last != updates::VERSION),
            show_changes: false,
            update_started: false,
            update_asked: false,
        }
    }

    /// Replace the drawing (New or Open), resetting everything that pointed into the old one.
    fn replace_document(&mut self, doc: Document, path: Option<PathBuf>) {
        self.doc = doc;
        self.tools.cancel();
        self.tools.active_material = None;
        self.tags = tags::TagsPanel::default();
        self.materials = materials::MaterialsPanel::default();
        self.viewport.document_replaced(&self.doc);
        self.files.path = path;
        self.files.name_hint = None;
        self.files.mark_saved(&self.doc);
        self.autosave.clear();
        self.watch.stamped(self.files.path.as_deref());
        self.reload_asking = false;
        self.reloaded = None;
    }

    /// Fit the whole drawing in the view (Zoom Extents).
    pub fn zoom_extents(&mut self) {
        self.viewport.request_zoom_extents();
    }

    /// Start a new, empty drawing (without asking about unsaved changes).
    pub fn new_document(&mut self) {
        self.replace_document(Document::default(), None);
    }

    /// Open the drawing at `path` (without asking about unsaved changes).
    pub fn open_path(&mut self, path: &Path) -> Result<(), String> {
        if files::is_sketchup(path) {
            // A new, unsaved drawing named after the model; saving asks where.
            let (doc, skipped) = files::Files::read_sketchup(path)?;
            self.replace_document(doc, None);
            self.files.name_hint = path.file_stem().map(|s| s.to_string_lossy().into_owned());
            self.files.mark_unsaved();
            self.viewport.request_zoom_extents();
            files::remember(&mut self.recent, path);
            if skipped > 0 {
                self.files.show_error(format!("Opened, but {skipped} faces couldn't be read and were left out."));
            }
            return Ok(());
        }
        let doc = files::Files::read(path)?;
        self.replace_document(doc, Some(path.to_owned()));
        files::remember(&mut self.recent, path);
        Ok(())
    }

    /// Add a SketchUp model to the drawing, where it was in its own file, selected; one undo step.
    pub fn import_sketchup(&mut self, path: &Path) -> Result<(), String> {
        let (imported, skipped) = files::Files::read_sketchup(path)?;
        let clip = woodsketch_doc::clipboard::Clip::from_scene(imported.scene().clone()).ok_or("That SketchUp model is empty.")?;
        self.tools.cancel();
        self.doc.paste(&clip, glam::DVec3::ZERO);
        self.viewport.request_zoom_extents();
        if skipped > 0 {
            self.files.show_error(format!("Imported, but {skipped} faces couldn't be read and were left out."));
        }
        Ok(())
    }

    /// Export the drawing to `path` in the File > Export units.
    pub fn export_to(&self, path: &Path, format: files::ExportFormat) -> Result<(), String> {
        files::export_to(&self.doc, path, format, self.export_units)
    }

    /// Save the drawing to `path`, which becomes its file.
    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        self.files.save_to(&self.doc, path)?;
        self.autosave.clear();
        files::remember(&mut self.recent, path);
        // Our own write, so take it as current: otherwise the next check reads it back as a
        // change from outside.
        self.watch.stamped(Some(path));
        self.reload_asking = false;
        Ok(())
    }

    /// Read the open drawing's file again, as one undo step so Ctrl+Z goes back to what was on
    /// screen. The camera, the selection and the tray stay as they are: the point is to watch a
    /// drawing change while something else edits it, which a Zoom Extents would spoil.
    pub fn reload_from_file(&mut self) {
        let Some(path) = self.files.path.clone() else { return };
        match files::Files::read(&path) {
            Ok(fresh) => {
                self.tools.cancel();
                self.doc.reload_from(&fresh);
                self.files.mark_saved(&self.doc);
                self.viewport.drawing_reloaded();
                self.watch.settled(Some(&path));
                self.reload_asking = false;
                self.reloaded = Some(std::time::Instant::now());
            }
            Err(error) => {
                self.watch.settled(Some(&path));
                self.reload_asking = false;
                self.files.show_error(error);
            }
        }
    }

    /// Keep the drawing in the window and leave the file alone; the file as it now is counts as
    /// seen, so the same change isn't raised again (saving from here still overwrites it, which
    /// is what "Keep mine" means).
    fn keep_mine(&mut self) {
        self.watch.settled(self.files.path.as_deref());
        self.reload_asking = false;
    }

    /// Autosave into `dir` (and offer back what's already there), as the app does with its
    /// settings folder; for tests, which have none.
    /// Pretend this version replaced `version`, as storage says after an update; for tests.
    pub fn came_from_version(&mut self, version: &str) {
        self.updated_from = Some(version.to_owned());
    }

    pub fn use_autosave_folder(&mut self, dir: PathBuf) {
        self.autosave = autosave::Autosave::new(Some(dir));
        self.leftovers = self.autosave.leftovers();
    }

    /// Look for a crash the last session left in `dir` (as the app does in its settings folder);
    /// for tests.
    pub fn use_crash_folder(&mut self, dir: PathBuf) {
        self.crash = crash::left(&dir);
        self.crash_dir = Some(dir);
    }

    /// After a crash (and any unsaved work offered back): offer to report it.
    fn offer_crash_report(&mut self, ctx: &egui::Context) {
        if !self.leftovers.is_empty() {
            return;
        }
        let Some(crash) = self.crash.clone() else { return };
        let (mut report, mut skip) = (false, false);
        egui::Modal::new(egui::Id::new("crash")).show(ctx, |ui| {
            ui.set_max_width(400.0);
            ui.heading("WoodSketch ran into a problem");
            ui.label(format!("It closed unexpectedly last time ({}). A report helps get it fixed: it says what went wrong and where in WoodSketch's code, and you'll see all of it before it's sent.", crash.version));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                report = ui.button("Report it...").clicked();
                skip = ui.button("Not now").clicked();
            });
        });
        if report || skip {
            if report {
                self.feedback.start_crash(crash);
            }
            self.crash = None;
            if let Some(dir) = &self.crash_dir {
                crash::clear(dir);
            }
        }
    }

    /// Look at the open drawing's file on the next frame instead of waiting out the check
    /// interval (tests, and anywhere a reload is asked for rather than noticed).
    pub fn check_file_now(&mut self) {
        self.watch.check_now();
    }

    /// Just updated, so the status bar says so until it's dismissed or What's New is read.
    pub(crate) fn updated(&self) -> bool {
        self.updated_from.is_some()
    }

    /// The versions to show in What's New: everything since the one that ran last, or the most
    /// recent few when there's nothing to compare against (a first run, or a long way behind).
    pub(crate) fn changes_to_show(&self) -> Vec<updates::Release> {
        let all = updates::releases(updates::CHANGES);
        let new = updates::since(&all, self.updated_from.as_deref());
        // Nothing new (same version as last run): show the recent ones rather than a blank
        // window, since Help > What's New can be opened any time.
        if new.is_empty() { updates::since(&all, None).to_vec() } else { new.to_vec() }
    }

    /// Open What's New, and stop saying "Updated to ..." in the status bar: it's been read.
    pub(crate) fn read_the_changes(&mut self) {
        self.show_changes = true;
    }

    /// Whether a reload just happened, for the note in the status bar. It clears itself: while
    /// the file is being watched a repaint is already due every [`watch::INTERVAL`].
    pub(crate) fn reloaded_recently(&self) -> bool {
        self.reloaded.is_some_and(|at| at.elapsed() < std::time::Duration::from_secs(5))
    }

    /// Check whether the open drawing's file changed outside WoodSketch, and reload it (or ask).
    fn watch_the_file(&mut self, ctx: &egui::Context) {
        let dirty = self.is_dirty();
        let (next, found) = self.watch.tick(self.files.path.as_deref(), dirty, std::time::Instant::now());
        if let Some(wait) = next {
            ctx.request_repaint_after(wait);
        }
        match found {
            Some(watch::Found::Reload) => self.reload_from_file(),
            Some(watch::Found::Ask) => self.reload_asking = true,
            None => {}
        }
        if !self.reload_asking {
            return;
        }
        let name = self.files.path.as_deref().and_then(Path::file_name).map_or_else(|| "The drawing".to_owned(), |n| n.to_string_lossy().into_owned());
        let (mut reload, mut keep) = (false, false);
        egui::Modal::new(egui::Id::new("reload")).show(ctx, |ui| {
            ui.set_max_width(400.0);
            ui.heading("The file changed outside WoodSketch");
            ui.label(format!("{name} was changed by something else, and this drawing has changes that aren't saved. Only one of the two can be kept."));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                reload = ui.button("Reload from the file").on_hover_text("Take the file as it is now, losing the unsaved changes here (Ctrl+Z brings them back)").clicked();
                keep = ui.button("Keep mine").on_hover_text("Leave this drawing as it is; saving will write over the file").clicked();
            });
        });
        if reload {
            self.reload_from_file();
        } else if keep {
            self.keep_mine();
        }
    }

    /// Write unsaved changes now and then, and offer back drawings a crash left behind.
    fn autosave_and_recover(&mut self, ctx: &egui::Context) {
        let dirty = self.is_dirty();
        match self.autosave.tick(&self.doc, self.files.path.as_deref(), dirty, std::time::Instant::now()) {
            Ok(Some(wait)) => ctx.request_repaint_after(wait),
            Ok(None) => {}
            Err(error) if !self.autosave_failed => {
                self.autosave_failed = true;
                self.files.show_error(error);
            }
            Err(_) => {}
        }
        if self.leftovers.is_empty() {
            return;
        }
        let mut recover = None;
        let mut discard = false;
        egui::Modal::new(egui::Id::new("recover")).show(ctx, |ui| {
            ui.set_max_width(380.0);
            ui.heading("Recover unsaved work?");
            ui.label("WoodSketch didn't close properly last time. It kept a copy of your unsaved changes:");
            for leftover in &self.leftovers {
                let when = match leftover.when.elapsed().map(|d| d.as_secs() / 60) {
                    Ok(0) => ", just now".to_owned(),
                    Ok(1) => ", a minute ago".to_owned(),
                    Ok(m) if m < 120 => format!(", {m} minutes ago"),
                    Ok(m) => format!(", {} hours ago", m / 60),
                    Err(_) => String::new(),
                };
                ui.horizontal(|ui| {
                    ui.label(format!("{}{when}", leftover.name()));
                    if ui.button("Recover").clicked() {
                        recover = Some(leftover.clone());
                    }
                });
            }
            ui.add_space(6.0);
            if ui.button(if self.leftovers.len() > 1 { "Discard all" } else { "Discard" }).clicked() {
                discard = true;
            }
        });
        if let Some(leftover) = recover {
            match files::Files::read(&leftover.file) {
                Ok(doc) => {
                    self.replace_document(doc, leftover.original.clone());
                    // Recovered work isn't saved yet.
                    self.files.mark_unsaved();
                    leftover.discard();
                    self.leftovers.retain(|l| l.file != leftover.file);
                    // Any others wait for the next start.
                    self.leftovers.clear();
                }
                Err(error) => {
                    leftover.discard();
                    self.leftovers.retain(|l| l.file != leftover.file);
                    self.files.show_error(error);
                }
            }
        }
        if discard {
            for leftover in self.leftovers.drain(..) {
                leftover.discard();
            }
        }
    }

    /// The build sheets for the drawing, as a web page (what File > Print build sheets saves).
    pub fn build_sheets_html(&mut self) -> String {
        let title = format!("{} build sheets", self.files.name());
        self.cut_list.build_sheets(&self.doc, &title)
    }

    /// Whether there are changes since the drawing was opened or last saved.
    pub fn is_dirty(&self) -> bool {
        self.files.is_dirty(&self.doc)
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.files.path.as_deref()
    }

    /// Plug in the updater (the app binary does, with Velopack).
    pub fn set_updater(&mut self, updater: updates::Updater) {
        self.updater = Some(updater);
    }

    /// Send problem reports with `post` instead of to woodsketch.net (tests).
    pub fn set_report_post(&mut self, post: feedback::Post) {
        self.feedback.set_post(post);
    }

    /// The quiet check at startup (when it's on), and the result of one the user asked for.
    fn keep_up_to_date(&mut self) {
        let Some(updater) = &self.updater else { return };
        if !self.update_started && self.check_updates {
            self.update_started = true;
            (updater.check)();
        }
        if self.update_asked {
            let state = updater.state();
            if !matches!(state, updates::UpdateState::Checking | updates::UpdateState::Idle) {
                self.update_asked = false;
                if !matches!(state, updates::UpdateState::Ready(_)) {
                    self.files.show_error(updates::describe(&state));
                }
            }
        }
    }

    /// Help > Check for Updates: check now, and say what happened.
    pub(crate) fn check_for_updates(&mut self) {
        match &self.updater {
            Some(updater) => {
                self.update_asked = true;
                self.update_started = true;
                (updater.check)();
            }
            None => self.files.show_error(updates::describe(&updates::UpdateState::NotInstalled)),
        }
    }

    /// "Restart to update": after any unsaved changes are dealt with.
    pub(crate) fn restart_to_update(&mut self, ctx: &egui::Context) {
        self.request(ctx, files::Discard::RestartToUpdate);
    }

    /// The update state for the status bar, if there's an updater.
    pub(crate) fn update_state(&self) -> Option<updates::UpdateState> {
        self.updater.as_ref().map(updates::Updater::state)
    }

    pub fn show_error(&mut self, error: String) {
        self.files.show_error(error);
    }

    /// Save to the drawing's file, asking where if it has none. Returns whether it saved.
    fn save(&mut self, choose: bool) -> bool {
        let path = match (&self.files.path, choose) {
            (Some(path), false) => path.clone(),
            _ => match files::save_dialog(self.files.path.as_deref()) {
                Some(path) => path,
                None => return false,
            },
        };
        match self.save_to(&path) {
            Ok(()) => true,
            Err(error) => {
                self.files.show_error(error);
                false
            }
        }
    }

    /// New, Open or Close, after any unsaved changes are dealt with.
    fn carry_out(&mut self, ctx: &egui::Context, action: files::Discard) {
        match action {
            files::Discard::New => self.new_document(),
            files::Discard::Open => {
                if let Some(path) = files::open_dialog(self.files.path.as_deref())
                    && let Err(error) = self.open_path(&path)
                {
                    self.files.show_error(error);
                }
            }
            files::Discard::OpenRecent(i) => {
                if let Some(path) = self.recent.get(i).cloned()
                    && let Err(error) = self.open_path(&path)
                {
                    // Gone or unreadable: drop it from the list.
                    self.recent.retain(|p| *p != path);
                    self.files.show_error(error);
                }
            }
            files::Discard::Close => self.files.close(ctx),
            files::Discard::RestartToUpdate => {
                self.autosave.clear();
                if let Some(updater) = &self.updater {
                    // The drawing open now (saved, or left as it was on disk) opens again after.
                    (updater.restart)(self.files.path.as_deref());
                }
            }
        }
    }

    /// New, Open or Close, asking "save changes?" first if needed.
    fn request(&mut self, ctx: &egui::Context, action: files::Discard) {
        if self.files.request(&self.doc, action) {
            self.carry_out(ctx, action);
        }
    }

    /// The File menu's shortcuts and dialogs, and the window title.
    fn handle_files(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            self.files.on_close_requested(ctx, &self.doc);
        }
        let command = egui::Modifiers::COMMAND;
        let (save_as, save, open, new) = ctx.input_mut(|i| {
            (
                i.consume_key(command | egui::Modifiers::SHIFT, egui::Key::S),
                i.consume_key(command, egui::Key::S),
                i.consume_key(command, egui::Key::O),
                i.consume_key(command, egui::Key::N),
            )
        });
        if save_as {
            self.save(true);
        } else if save {
            self.save(false);
        }
        if open {
            self.request(ctx, files::Discard::Open);
        }
        if new {
            self.request(ctx, files::Discard::New);
        }
        match self.files.dialogs(ctx) {
            Some(files::FileAction::SaveThen(action)) => {
                if self.save(false) {
                    self.carry_out(ctx, action);
                }
            }
            Some(files::FileAction::Discard(action)) => self.carry_out(ctx, action),
            None => {}
        }
        self.files.update_title(ctx, &self.doc);
    }

    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// The drawing, for building test scenes directly.
    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.doc
    }

    pub fn tools(&self) -> &Tools {
        &self.tools
    }

    /// The viewport's camera. For tests.
    pub fn camera(&self) -> &woodsketch_view::OrbitCamera {
        &self.viewport.camera
    }

    /// The camera to turn, and View > Wood Movement to switch: for tests and the screenshots
    /// they make.
    pub fn camera_mut(&mut self) -> &mut woodsketch_view::OrbitCamera {
        &mut self.viewport.camera
    }

    pub fn show_wood_movement(&mut self, show: bool) {
        self.viewport.show_movement = show;
    }

    /// Where a model point (inches) appears in the window. For tests that click on geometry.
    pub fn to_window(&self, point: glam::DVec3) -> Option<egui::Pos2> {
        self.viewport.to_window(point)
    }
}

impl eframe::App for WoodSketchApp {
    fn on_exit(&mut self) {
        // Closed properly (saved, or the changes declined): nothing to recover.
        self.autosave.clear();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, RECENT_KEY, &self.recent);
        eframe::set_value(storage, UPDATES_KEY, &self.check_updates);
        eframe::set_value(storage, LAST_VERSION_KEY, &updates::VERSION.to_owned());
        eframe::set_value(storage, UNITS_KEY, &woodsketch_doc::units::units());
        let theme = self.theme;
        if let Some((_, _, key)) = chrome::THEMES.iter().find(|(t, _, _)| *t == theme) {
            eframe::set_value(storage, THEME_KEY, &key.to_string());
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.theme = ctx.options(|o| o.theme_preference);
        self.handle_files(&ctx);
        self.watch_the_file(&ctx);
        self.autosave_and_recover(&ctx);
        self.offer_crash_report(&ctx);
        self.keep_up_to_date();
        if !self.files.is_asking() && self.leftovers.is_empty() && self.crash.is_none() && !self.reload_asking {
            self.keyboard.handle(&ctx, &mut self.doc, &mut self.tools, &mut self.viewport);
        }

        // Panels claim space in order: bars across the top and bottom, then the sides.
        self.menu_bar(ui);
        self.toolbar(ui);
        self.status_bar(ui);
        self.tool_set(ui);
        self.tray(ui);

        // "Edit material" in the Cut List opens it in the tray.
        if let Some(material) = self.cut_list.show(&ctx, &mut self.doc) {
            self.materials.editing = Some(material);
            self.show_tray = true;
        }
        self.about_window(&ctx);
        self.changes_window(&ctx);
        self.feedback.show(&ctx, &self.doc, &self.graphics, self.files.last_error.as_deref());

        egui::CentralPanel::no_frame()
            .show(ui, |ui| self.viewport.show(ui, frame, &mut self.doc, &mut self.tools));
    }
}
