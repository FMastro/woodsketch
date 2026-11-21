//! What `packaging/linux/` says about WoodSketch, held to what WoodSketch actually does.
//!
//! These files are read by other programs on a machine nobody here can see: `update-mime-database`
//! when a `.deb` or `.rpm` is installed, and the desktop when it matches a window to its entry.
//! Both fail quietly. A malformed MIME file is refused by the database and the install carries on
//! past it, so a drawing simply never gets its icon and double-clicking never opens it; a
//! `StartupWMClass` that doesn't match what the window calls itself leaves the app nameless in the
//! taskbar. There is nothing to notice either, short of installing a package on each of several
//! distributions, so the checks live here instead.
//!
//! The one that matters most is the magic: it is written from the shape of a saved drawing, and
//! a saved drawing is written by `doc::file::to_string`. Nothing else connects the two.

use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    // CARGO_MANIFEST_DIR is apps/woodsketch.
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read(relative: &str) -> String {
    let path = repo().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The one attribute `name` of the first `<tag`, as a crude but sufficient reader: these two files
/// are written by hand here and never generated, so they are always the shape this expects.
fn attribute(xml: &str, tag: &str, name: &str) -> String {
    let at = xml.find(&format!("<{tag}")).unwrap_or_else(|| panic!("no <{tag} in the file"));
    let rest = &xml[at..];
    let from = rest.find(&format!("{name}=\"")).unwrap_or_else(|| panic!("no {name}= on <{tag}")) + name.len() + 2;
    let to = rest[from..].find('"').expect("an attribute value ends");
    rest[from..from + to].replace("&quot;", "\"")
}

/// What a `.desktop` entry says for `key`.
fn entry(desktop: &str, key: &str) -> String {
    desktop.lines().find_map(|l| l.strip_prefix(&format!("{key}="))).unwrap_or_else(|| panic!("no {key}= in the desktop entry")).trim().to_owned()
}

/// A double hyphen inside an XML comment is not allowed, and `update-mime-database` refuses the
/// whole file over it -- which is how the type came to not register at all the first time this
/// was packaged. The prose in these files is as explanatory as the rest, so it is worth checking.
#[test]
fn the_mime_file_is_what_an_xml_parser_will_take() {
    let xml = read("packaging/linux/woodsketch.xml");
    let mut rest = xml.as_str();
    while let Some(open) = rest.find("<!--") {
        let body = &rest[open + 4..];
        let close = body.find("-->").expect("every comment is closed");
        assert!(!body[..close].contains("--"), "a double hyphen inside an XML comment:\n{}", &body[..close]);
        rest = &body[close + 3..];
    }
    assert!(xml.contains(r#"xmlns="http://www.freedesktop.org/standards/shared-mime-info""#), "the namespace the database expects");
}

/// The glob claims the extension the app actually saves, and only that one: `.scraft` is still
/// opened but deliberately not claimed, as on Windows.
#[test]
fn the_mime_file_claims_the_extension_the_app_saves() {
    let xml = read("packaging/linux/woodsketch.xml");
    // Every glob in the file, not just the first: a second one is how the old extension would
    // creep back in. The prose above them is allowed to mention it; a glob is the claim.
    let globs: Vec<String> = xml.match_indices("<glob").map(|(at, _)| attribute(&xml[at..], "glob", "pattern")).collect();
    assert_eq!(globs, vec![format!("*.{}", woodsketch_doc::file::EXTENSION)], "one glob, for the extension the app saves");
    assert!(!globs.iter().any(|g| g.contains(woodsketch_doc::file::OLD_EXTENSION)), "the old extension belongs to the Craft apps, so it isn't claimed");
}

/// The magic is the first bytes of a real saved drawing, so a renamed one is still recognized.
/// This is what would break silently if the drawing were ever written compactly instead of
/// pretty-printed, or if the format stopped being the first key.
#[test]
fn the_mime_magic_is_how_a_saved_drawing_really_starts() {
    let xml = read("packaging/linux/woodsketch.xml");
    let magic = attribute(&xml, "match", "value");
    let offsets = attribute(&xml, "match", "offset");
    let (from, to) = offsets.split_once(':').expect("an offset range, as `0:16`");
    let (from, to): (usize, usize) = (from.parse().expect("a number"), to.parse().expect("a number"));

    let saved = woodsketch_doc::file::to_string(woodsketch_doc::Document::default().scene());
    let at = saved.find(&magic).unwrap_or_else(|| panic!("a saved drawing doesn't contain {magic:?}; it starts:\n{}", &saved[..80.min(saved.len())]));
    assert!(at >= from && at <= to, "{magic:?} is at {at}, outside the {from}:{to} the file says to look in");
}

/// The desktop entry and the window have to agree on what the app is called, and the entry has to
/// claim the type the MIME file defines, or double-clicking a drawing opens nothing.
#[test]
fn the_desktop_entry_agrees_with_the_app_and_the_mime_type() {
    let desktop = read("packaging/linux/woodsketch.desktop");
    let xml = read("packaging/linux/woodsketch.xml");
    let main = read("apps/woodsketch/src/main.rs");

    // The app id `main.rs` asks for, read from the source the way the i18n guard tests do: a
    // binary's constants can't be imported, and this is the only thing that holds the two names
    // together.
    let id = main.split("with_app_id(\"").nth(1).expect("main.rs asks for an app id").split('"').next().expect("a string");
    assert_eq!(entry(&desktop, "StartupWMClass"), id, "the taskbar matches the window to the entry by this name");

    let mime = attribute(&xml, "mime-type", "type");
    assert_eq!(entry(&desktop, "MimeType"), format!("{mime};"), "the entry claims the type the MIME file defines");
    // `%f` is the file that was double-clicked; without it a drawing opens the app and no drawing.
    assert_eq!(entry(&desktop, "Exec"), "woodsketch %f");
    assert_eq!(entry(&desktop, "Icon"), "woodsketch", "the icon name the packages install under hicolor");
}
