//! What the app can say, against the fonts it can draw it with.
//!
//! A translation can be letter-perfect and still reach the screen as rows of empty boxes: the
//! glyph has to be in a font that is listed under the family the text is drawn in, and nothing
//! else here would notice that it wasn't. The i18n guards compare keys, not characters.
//! `workflow.rs` reads labels through the accessibility tree, where a label is the string and not
//! the drawing of it, so a window full of tofu passes every test in this repository. The first to
//! notice would be a reader, which is too late.
//!
//! It also catches a font arriving broken, which is the likeliest way for one to stop working
//! without anybody touching the code: `.ttf` is `binary` in `.gitattributes` because a
//! 20,677-byte render of a dovetail once went into git a byte short, and a font mangled the same
//! way would still be nine megabytes of plausible-looking file.
//!
//! The fonts come from `fonts::definitions`, which is what the app hands egui, so this is the
//! real set -- egui's own Latin faces included -- rather than a list kept in step by hand.

use eframe::egui;
use read_fonts::{FontRef, TableProvider};
use woodsketch_ui_egui::fonts;

/// Every language's own catalog, found rather than listed, so a new one is held to this the day
/// it lands without anybody adding it here.
fn catalogs() -> Vec<(String, String)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../i18n/i18n");
    let mut found: Vec<(String, String)> = std::fs::read_dir(&dir)
        .expect("the catalogs")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|kind| kind == "ftl"))
        .map(|path| {
            let tag = path.file_stem().expect("a tag").to_string_lossy().into_owned();
            let catalog = std::fs::read_to_string(&path).expect("a catalog");
            // Comment lines are for whoever translates the file and never reach a window. The
            // English one describes the close button as "the X on a panel's header bar" with a
            // character the app doesn't draw with, which would fail this for no reader's sake.
            let said: String = catalog.lines().filter(|line| !line.trim_start().starts_with('#')).collect::<Vec<_>>().join(
                "
",
            );
            (tag, said)
        })
        .collect();
    found.sort();
    assert!(found.len() > 1, "only one catalog was found, so the rest of this would prove nothing");
    found
}

/// The fonts behind one family, in the order egui reaches them.
fn behind(family: &egui::FontFamily) -> Vec<(String, Vec<u8>)> {
    let definitions = fonts::definitions();
    let named = definitions.families.get(family).unwrap_or_else(|| panic!("nothing is listed under {family:?}"));
    named
        .iter()
        .map(|name| {
            let data = definitions.font_data.get(name).unwrap_or_else(|| panic!("{name} is listed under {family:?} but was never registered"));
            (name.clone(), data.font.to_vec())
        })
        .collect()
}

/// The families any of the app's own words are drawn in. Text goes out proportional; the
/// Measurements box and the cut list's columns are monospace.
fn families() -> Vec<egui::FontFamily> {
    vec![egui::FontFamily::Proportional, egui::FontFamily::Monospace]
}

fn can_draw(bytes: &[u8], letter: char) -> bool {
    FontRef::new(bytes).ok().and_then(|font| font.cmap().ok()).and_then(|cmap| cmap.map_codepoint(letter)).is_some()
}

/// A font that doesn't parse draws nothing at all.
#[test]
fn every_font_the_app_hands_egui_is_a_font() {
    let mut checked = 0;
    for family in families() {
        for (name, bytes) in behind(&family) {
            let font = FontRef::new(&bytes);
            assert!(font.is_ok(), "{name} isn't a font ({} bytes): {:?}", bytes.len(), font.err());
            let font = font.expect("checked");
            assert!(font.cmap().is_ok(), "{name} has no character map, so nothing in it can be looked up");
            assert!(font.glyf().is_ok() || font.cff().is_ok() || font.cff2().is_ok(), "{name} carries no outlines");
            checked += 1;
        }
    }
    assert!(checked >= 2, "only {checked} font(s) were checked, which is fewer than the app ships");
}

/// Every character every catalog can put on screen is in one of the fonts behind the family it
/// would be drawn in.
#[test]
fn nothing_a_catalog_says_comes_out_as_an_empty_box() {
    let catalogs = catalogs();
    let mut missing: Vec<String> = Vec::new();
    for family in families() {
        let fonts = behind(&family);
        for (tag, catalog) in &catalogs {
            let mut seen = std::collections::BTreeSet::new();
            for letter in catalog.chars().filter(|letter| !letter.is_ascii() && !letter.is_whitespace()) {
                if seen.insert(letter) && !fonts.iter().any(|(_, bytes)| can_draw(bytes, letter)) {
                    missing.push(format!("{tag}: {letter:?} (U+{:04X}) in {family:?}", letter as u32));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "no font behind the family these are drawn in can draw them, so they reach the screen as empty boxes:\n  {}",
        missing.join("\n  ")
    );
}

/// The Japanese catalog is the reason there is a 9 MB font in here: if its characters were
/// already covered, the font would be weight for nothing. This says which way round it is, so
/// nobody removes the font on the grounds that the tests still pass without it.
#[test]
fn the_japanese_catalog_is_what_needs_the_japanese_font() {
    let (_, japanese) = catalogs().into_iter().find(|(tag, _)| tag == "ja-JP").expect("the Japanese catalog");
    let fonts = behind(&egui::FontFamily::Proportional);
    let (last, bytes) = fonts.last().expect("a font").clone();
    let wanted: Vec<char> = japanese.chars().filter(|letter| *letter as u32 >= 0x3000).take(200).collect();
    assert!(wanted.len() > 50, "the Japanese catalog has almost no Japanese in it");
    assert!(wanted.iter().all(|letter| can_draw(&bytes, *letter)), "{last} is last in the family but can't draw the Japanese it is there for");
    let others: Vec<&(String, Vec<u8>)> = fonts.iter().filter(|(name, _)| *name != last).collect();
    let covered = wanted.iter().filter(|letter| others.iter().any(|(_, other)| can_draw(other, **letter))).count();
    assert!(covered * 4 < wanted.len(), "{} of {} Japanese characters are already covered without {last}", covered, wanted.len());
}
