//! Does a translation actually reach the screen?
//!
//! Its own test file, and so its own process, on purpose. The language is one global thing — the
//! window, the thumbnail threads and the sample builder all have to be saying the same language
//! at once — and `workflow.rs` runs its seventy-odd tests in parallel threads, most of which find
//! widgets by their English labels. A test that switched language in that process would fail the
//! others, and the failures would point anywhere but here.
//!
//! One test rather than several, for the same reason one step down: tests in a file share a
//! process, so two of these running at once would each be reading the other's language. It also
//! keeps to one wgpu device at a time, which is what `workflow.rs` needs its GPU lock for.
//!
//! The catalog below is written in this file rather than shipped. No translation exists yet, and
//! inventing one in a language nobody here reads well enough to check would be a worse test than
//! none: it would prove the machinery works while quietly asserting the Portuguese is right.

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use woodsketch_ui_egui::WoodSketchApp;

/// Part of a translation: the File menu and one item inside it, and deliberately nothing else, so
/// what falls back to English can be told apart from what doesn't.
const PARTIAL: &str = "\
menu-file = Arquivo
file-new = Novo
";

fn app() -> Harness<'static, WoodSketchApp> {
    let mut harness = Harness::builder()
        .with_size([1200.0, 800.0])
        .with_step_dt(1.0 / 60.0)
        .with_theme(egui::Theme::Light)
        .wgpu()
        .build_eframe(|cc| WoodSketchApp::new(cc));
    harness.run();
    harness
}

#[test]
fn a_translation_reaches_the_menus_and_english_fills_its_gaps() {
    // One window throughout, switched under it. That is what View > Language does, and it is the
    // stronger claim: nothing is re-read at startup, so no restart is needed. Setting the
    // language *before* building the app would prove less, and in fact proves nothing — the app
    // applies the saved or the computer's language as it starts, so it would overwrite this.
    let mut harness = app();

    // Tests have no settings storage, so nothing has been chosen and nothing is remembered: the
    // app opens in English, which is what every other test in this crate relies on.
    harness.get_by_label("File");
    harness.get_by_label("Edit");

    // Now speak a translation covering two strings out of hundreds.
    assert!(woodsketch_i18n::speak_source("pt-TEST", PARTIAL), "a catalog that parses is spoken");
    harness.run();

    // Translated: the menu's own title, with no restart.
    harness.get_by_label("Arquivo");
    // In the catalog's gaps, so English shows rather than a key or a blank. A reader would see
    // these in English beside the Portuguese, which is the whole point of the fallback.
    harness.get_by_label("Edit");
    harness.get_by_label("View");

    // Inside the menu, which proves a label deeper than the bar is translated too, and not only
    // the one the bar happens to be showing.
    harness.get_by_label("Arquivo").click();
    harness.run();
    // `_contains`, because a menu item's label carries its shortcut: "Novo Ctrl+N".
    harness.get_by_label_contains("Novo");
    harness.get_by_label_contains("Save As...");
    harness.get_by_label("Arquivo").click();
    harness.run();

    // A half-written catalog is refused whole, rather than spoken and leaving blank menus.
    assert!(!woodsketch_i18n::speak_source("bad-TEST", "= not fluent {"), "a broken catalog is refused");
    assert_eq!(woodsketch_i18n::language(), "pt-TEST", "and refusing it leaves what was spoken alone");
    harness.run();
    harness.get_by_label("Arquivo");

    // What happens to someone who chose a language a later version dropped: English, not a
    // window with no menus.
    assert!(!woodsketch_i18n::set_language("tlh-Piqd"), "Klingon is not built in");
    assert!(woodsketch_i18n::set_language(woodsketch_i18n::BASE));
    harness.run();
    harness.get_by_label("File");
    harness.get_by_label("Edit");
}
