//! What WoodSketch says, in whatever language it is being read in.
//!
//! Every user-facing string lives in `i18n/<tag>.ftl` under a key, and the code asks for the key:
//! `t!("menu-file")` is the File menu's title in the language now being spoken. English
//! (`[BASE]`) is the catalog every other language is measured against: it is the one the code is
//! written against, so a key it lacks is a mistake in the code, and a key a translation lacks
//! falls back to English rather than showing a blank menu to someone.
//!
//! Nothing here is checked by the compiler — a key is a string, and a string can be misspelled.
//! Two tests stand in for the type system and both run in CI: `every_key_the_code_asks_for_exists`
//! reads the whole workspace for `t!` calls and looks each key up, and
//! `every_language_says_what_english_says` compares each translation's keys against English.
//! A bad key therefore fails the build, not a user's menu.
//!
//! The language is global rather than per-thread: the window, the thumbnail threads and the
//! sample builder all say the same thing at once, and nobody chooses a language per thread.
//! It starts as English, which is what tests get without arranging anything — they match widgets
//! by their English labels.

use std::sync::{PoisonError, RwLock};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

/// The language the code is written in, and the one every other is filled in from.
pub const BASE: &str = "en-US";

/// Every language WoodSketch has been taught, each named in itself, because a reader looking for
/// their own language is not reading the one it is listed under. Add a `.ftl` beside the others
/// and a line here; nothing else knows how many there are.
const CATALOGS: &[Catalog] = &[
    Catalog { tag: "en-US", endonym: "English", source: include_str!("../i18n/en-US.ftl") },
    Catalog { tag: "pt-PT", endonym: "Português (Portugal)", source: include_str!("../i18n/pt-PT.ftl") },
    // A translation belongs to whoever speaks the language. `docs/TRANSLATING.md` says how to
    // add one, and it is one file and one line here.
];

struct Catalog {
    /// A BCP 47 tag: `pt-BR`, not `Portuguese`.
    tag: &'static str,
    /// The language's name in that language.
    endonym: &'static str,
    source: &'static str,
}

/// A value dropped into a translation: `t!("status-updated", "version" => "0.1.66")`.
///
/// Numbers stay numbers all the way into Fluent rather than being formatted here, because that is
/// what lets a translation pick a plural form — and in a language whose plurals English has no
/// equivalent of, pick one English could not have known to ask for.
#[derive(Debug, Clone)]
pub enum Arg<'a> {
    Text(&'a str),
    Owned(String),
    Count(i64),
    Number(f64),
}

impl<'a> From<&'a str> for Arg<'a> {
    fn from(value: &'a str) -> Self {
        Self::Text(value)
    }
}

impl From<String> for Arg<'_> {
    fn from(value: String) -> Self {
        Self::Owned(value)
    }
}

impl From<i64> for Arg<'_> {
    fn from(value: i64) -> Self {
        Self::Count(value)
    }
}

impl From<usize> for Arg<'_> {
    fn from(value: usize) -> Self {
        Self::Count(i64::try_from(value).unwrap_or(i64::MAX))
    }
}

impl From<u32> for Arg<'_> {
    fn from(value: u32) -> Self {
        Self::Count(i64::from(value))
    }
}

impl From<u64> for Arg<'_> {
    fn from(value: u64) -> Self {
        Self::Count(i64::try_from(value).unwrap_or(i64::MAX))
    }
}

impl From<u8> for Arg<'_> {
    fn from(value: u8) -> Self {
        Self::Count(i64::from(value))
    }
}

impl From<u16> for Arg<'_> {
    fn from(value: u16) -> Self {
        Self::Count(i64::from(value))
    }
}

impl From<i32> for Arg<'_> {
    fn from(value: i32) -> Self {
        Self::Count(i64::from(value))
    }
}

impl From<f64> for Arg<'_> {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl<'a> From<&'a Arg<'a>> for FluentValue<'a> {
    fn from(arg: &'a Arg<'a>) -> Self {
        match arg {
            Arg::Text(text) => FluentValue::from(*text),
            Arg::Owned(text) => FluentValue::from(text.as_str()),
            Arg::Count(n) => FluentValue::from(*n),
            Arg::Number(n) => FluentValue::from(*n),
        }
    }
}

/// Ask for a key: `t!("menu-file")`, or with values, `t!("status-updated", "version" => v)`.
///
/// The key must be a literal, so that the test which reads the source for keys can find it. A key
/// worked out at runtime would be invisible to that test, and the first anyone knew of it being
/// wrong would be the key itself appearing in the interface.
#[macro_export]
macro_rules! t {
    ($key:literal) => { $crate::text($key) };
    ($key:literal, $($name:literal => $value:expr),+ $(,)?) => {
        $crate::text_with($key, &[$(($name, $crate::Arg::from($value))),+])
    };
}

struct Speaking {
    /// Owned rather than `'static`, because `speak_source` can be handed a tag from a file.
    tag: String,
    /// The language asked for, then English behind it. One bundle when English *is* the language.
    bundles: Vec<FluentBundle<FluentResource>>,
}

static SPEAKING: RwLock<Option<Speaking>> = RwLock::new(None);

fn bundle_for(catalog: &Catalog) -> FluentBundle<FluentResource> {
    bundle_from(catalog.tag, catalog.source)
}

fn bundle_from(tag: &str, source: &str) -> FluentBundle<FluentResource> {
    let id: LanguageIdentifier = tag.parse().unwrap_or_else(|_| BASE.parse().expect("en-US is a language tag"));
    let mut bundle = FluentBundle::new_concurrent(vec![id]);
    // Fluent wraps every inserted value in invisible direction marks, which are right for prose
    // that mixes a right-to-left language with a left-to-right name and wrong for a menu: they
    // would sit inside the label, so a test looking for "3 parts" would not find it.
    bundle.set_use_isolating(false);
    match FluentResource::try_new(source.to_owned()) {
        Ok(resource) => {
            // An overlapping key would mean the same catalog added twice; there is nothing to do
            // about it here, and the tests below are what catch a catalog that will not parse.
            let _ = bundle.add_resource(resource);
        }
        Err((resource, _errors)) => {
            let _ = bundle.add_resource(resource);
        }
    }
    bundle
}

fn catalog(tag: &str) -> Option<&'static Catalog> {
    CATALOGS.iter().find(|c| c.tag.eq_ignore_ascii_case(tag))
}

fn load(tag: &str) -> Speaking {
    let base = catalog(BASE).expect("English is always built in");
    match catalog(tag) {
        Some(chosen) if chosen.tag != BASE => {
            Speaking { tag: chosen.tag.to_owned(), bundles: vec![bundle_for(chosen), bundle_for(base)] }
        }
        _ => Speaking { tag: base.tag.to_owned(), bundles: vec![bundle_for(base)] },
    }
}

fn with<R>(f: impl FnOnce(&Speaking) -> R) -> R {
    {
        let read = SPEAKING.read().unwrap_or_else(PoisonError::into_inner);
        if let Some(speaking) = read.as_ref() {
            return f(speaking);
        }
    }
    let mut write = SPEAKING.write().unwrap_or_else(PoisonError::into_inner);
    let speaking = write.get_or_insert_with(|| load(BASE));
    f(speaking)
}

/// Speak this language from now on. Returns false, and changes nothing, for a language that isn't
/// built in — a saved setting naming a language a later version dropped shouldn't empty the menus.
pub fn set_language(tag: &str) -> bool {
    let Some(found) = catalog(tag) else { return false };
    let mut write = SPEAKING.write().unwrap_or_else(PoisonError::into_inner);
    *write = Some(load(found.tag));
    true
}

/// The language being spoken, as its tag.
pub fn language() -> String {
    with(|speaking| speaking.tag.clone())
}

/// Speak a catalog that isn't built in: a `.ftl` a translator is still working on, or the one the
/// app's own tests use to prove a translation reaches the screen. English stays behind it, so a
/// key the catalog hasn't got still reads in English rather than as its own name.
///
/// False, changing nothing, if the source won't parse — a half-written translation shouldn't be
/// able to empty somebody's menus.
pub fn speak_source(tag: &str, source: &str) -> bool {
    if FluentResource::try_new(source.to_owned()).is_err() {
        return false;
    }
    let base = catalog(BASE).expect("English is always built in");
    let mut write = SPEAKING.write().unwrap_or_else(PoisonError::into_inner);
    *write = Some(Speaking { tag: tag.to_owned(), bundles: vec![bundle_from(tag, source), bundle_for(base)] });
    true
}

/// Every language on offer, as (tag, its name in itself), English first.
pub fn languages() -> impl Iterator<Item = (&'static str, &'static str)> {
    CATALOGS.iter().map(|c| (c.tag, c.endonym))
}

/// The built-in language closest to what the computer is set to, or English.
///
/// An exact tag wins; failing that, any language of the same tongue, so a computer set to
/// `pt-PT` is answered in `pt-BR` rather than in English. That is a better wrong answer: the
/// reader can read it, and the Language menu is right there.
pub fn system_language() -> &'static str {
    let Some(locale) = sys_locale::get_locale() else { return BASE };
    if let Some(exact) = catalog(&locale) {
        return exact.tag;
    }
    let tongue = locale.split(['-', '_']).next().unwrap_or_default().to_ascii_lowercase();
    CATALOGS
        .iter()
        .find(|c| c.tag.split('-').next().unwrap_or_default().eq_ignore_ascii_case(&tongue))
        .map_or(BASE, |c| c.tag)
}

/// What this key says. A key nothing defines comes back as itself, which is wrong on screen but
/// never blank, and is what the tests are there to prevent reaching anyone.
pub fn text(key: &str) -> String {
    text_with(key, &[])
}

/// What this key says, with values put into it.
pub fn text_with(key: &str, args: &[(&str, Arg<'_>)]) -> String {
    let mut fluent = FluentArgs::new();
    for (name, arg) in args {
        fluent.set(*name, FluentValue::from(arg));
    }
    let args = if fluent.iter().next().is_some() { Some(&fluent) } else { None };
    with(|speaking| {
        for bundle in &speaking.bundles {
            if let Some(message) = bundle.get_message(key)
                && let Some(pattern) = message.value()
            {
                let mut errors = Vec::new();
                let text = bundle.format_pattern(pattern, args, &mut errors);
                // Errors here are a translation referring to a value the call didn't pass. The
                // text still comes back, with the name showing, which is the loudest a missing
                // value can be without being a crash.
                return text.into_owned();
            }
        }
        key.to_owned()
    })
}

/// Every key a catalog defines. Only the tests below need it; nothing at runtime asks a
/// catalog what it contains.
#[cfg(test)]
fn keys_in(source: &str) -> std::collections::BTreeSet<String> {
    let resource = match FluentResource::try_new(source.to_owned()) {
        Ok(resource) | Err((resource, _)) => resource,
    };
    resource
        .entries()
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(message) => Some(message.id.name.to_owned()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// Every `.rs` in the workspace, so the keys the code asks for can be read out of it.
    fn sources() -> Vec<PathBuf> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("the crate sits two deep in the workspace").to_owned();
        let mut found = Vec::new();
        let mut stack = vec![root.join("crates"), root.join("apps")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().is_some_and(|n| n == "target") {
                        continue;
                    }
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    found.push(path);
                }
            }
        }
        found
    }

    /// The keys every `t!` in the workspace asks for, with the file each came from.
    ///
    /// Comments and ordinary strings are skipped, so an example in a doc comment is not mistaken
    /// for a call — which it was, the first time this test ran, and it named this crate's own
    /// documentation as the culprit.
    fn keys_asked_for() -> Vec<(String, String)> {
        let mut asked = Vec::new();
        for path in sources() {
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            let where_from = path.display().to_string();
            for key in keys_in_code(&text) {
                asked.push((key, where_from.clone()));
            }
        }
        asked
    }

    /// Every key asked for by a `t!("...")` that is really code: not in a comment, not inside
    /// another string. Rust's own lexer would be the honest way to do this, but a dependency on
    /// one to read our own source is a worse trade than twenty lines of state machine.
    fn keys_in_code(text: &str) -> Vec<String> {
        let bytes = text.as_bytes();
        let mut found = Vec::new();
        let mut i = 0;
        let (mut in_string, mut in_line, mut in_block) = (false, false, 0usize);
        while i < bytes.len() {
            let b = bytes[i];
            if in_line {
                if b == b'\n' {
                    in_line = false;
                }
                i += 1;
            } else if in_block > 0 {
                if bytes[i..].starts_with(b"/*") {
                    in_block += 1;
                    i += 2;
                } else if bytes[i..].starts_with(b"*/") {
                    in_block -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            } else if in_string {
                // An escape takes the next byte with it, so a \" doesn't end the string.
                i += if b == b'\\' { 2 } else { 1 };
                if b == b'"' {
                    in_string = false;
                }
            } else if bytes[i..].starts_with(b"//") {
                in_line = true;
                i += 2;
            } else if bytes[i..].starts_with(b"/*") {
                in_block = 1;
                i += 2;
            } else if bytes[i..].starts_with(b"t!(\"") {
                // `format!("` and `print!("` end in the same four bytes, so what sits before the
                // `t` has to be something no name could end with.
                let joined = i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
                let rest = &text[i + 4..];
                match rest.find('"') {
                    Some(end) if !joined && end > 0 => {
                        found.push(rest[..end].to_owned());
                        i += 4 + end;
                    }
                    _ => {
                        in_string = true;
                        i += 4;
                    }
                }
            } else {
                if b == b'"' {
                    in_string = true;
                }
                i += 1;
            }
        }
        found
    }

    #[test]
    fn english_is_a_catalog_that_parses() {
        let english = catalog(BASE).expect("English is built in");
        assert!(FluentResource::try_new(english.source.to_owned()).is_ok(), "the English catalog has syntax errors");
        assert!(!keys_in(english.source).is_empty(), "the English catalog says nothing");
    }

    #[test]
    fn every_catalog_parses_and_is_named_in_itself() {
        for catalog in CATALOGS {
            assert!(FluentResource::try_new(catalog.source.to_owned()).is_ok(), "{} has syntax errors", catalog.tag);
            assert!(catalog.tag.parse::<LanguageIdentifier>().is_ok(), "{} isn't a language tag", catalog.tag);
            assert!(!catalog.endonym.is_empty(), "{} has no name", catalog.tag);
        }
    }

    /// The test that stands in for the compiler: a misspelled key fails here rather than showing
    /// itself in somebody's menu.
    /// Every string literal handed straight to one of egui's text calls: a word on screen that
    /// never passed through the catalog. Comments are skipped, so a `.label("x")` written in
    /// documentation is not mistaken for one.
    fn screen_literals(text: &str) -> Vec<String> {
        const CALLS: [&str; 9] = [
            ".label(",
            ".button(",
            ".small_button(",
            ".strong(",
            ".weak(",
            ".heading(",
            ".on_hover_text(",
            ".monospace(",
            "Button::new(",
        ];
        let bytes = text.as_bytes();
        let mut found = Vec::new();
        let mut i = 0;
        let (mut in_line, mut in_block) = (false, 0usize);
        while i < bytes.len() {
            if in_line {
                if bytes[i] == b'\n' {
                    in_line = false;
                }
                i += 1;
            } else if in_block > 0 {
                if bytes[i..].starts_with(b"/*") {
                    in_block += 1;
                    i += 2;
                } else if bytes[i..].starts_with(b"*/") {
                    in_block -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            } else if bytes[i..].starts_with(b"//") {
                in_line = true;
                i += 2;
            } else if bytes[i..].starts_with(b"/*") {
                in_block = 1;
                i += 2;
            } else if bytes[i] == b'"' {
                // A `'"'` char literal, not the start of a string.
                if i > 0 && bytes[i - 1] == b'\'' && bytes.get(i + 1) == Some(&b'\'') {
                    i += 2;
                    continue;
                }
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && bytes[j] != b'"' {
                    if bytes[j] == b'\\' {
                        j += 1;
                    }
                    j += 1;
                }
                if CALLS.iter().any(|call| text[..i].ends_with(call)) {
                    found.push(text[start..j.min(text.len())].to_owned());
                }
                i = j + 1;
            } else {
                i += 1;
            }
        }
        found
    }

    /// Nothing a woodworker reads is written into the code.
    ///
    /// `ui-egui` and `tools` are fully translated, and that claim was wrong twice before this
    /// test existed: a Change button and its tooltip were handed to egui as literals while the
    /// catalog held the right keys unused, and the Start page's "New" heading, the What's New
    /// window's "(this one)", a build-sheets tooltip, an OK button and Ctrl+D's "Select
    /// something to duplicate" were never keys at all. Each of those was found by eye, which is
    /// not a method. This looks instead.
    #[test]
    fn nothing_on_screen_is_written_into_the_code() {
        // A key's own name, the same on every keyboard, so deliberately not in the catalog.
        // Anything with no letters in it at all (an arrow, a dash) is fine as it stands.
        const ALLOWED: [&str; 1] = ["Esc"];
        let mut stray = Vec::new();
        let mut looked_at = 0;
        for path in sources() {
            let shown = path.display().to_string().replace('\\', "/");
            if !(shown.contains("/ui-egui/src/") || shown.contains("/tools/src/")) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            looked_at += 1;
            for literal in screen_literals(&text) {
                if literal.chars().any(char::is_alphabetic) && !ALLOWED.contains(&literal.as_str()) {
                    stray.push(format!("{shown}: {literal:?}"));
                }
            }
        }
        assert!(looked_at > 20, "only {looked_at} files scanned, so this test is proving nothing");
        assert!(
            stray.is_empty(),
            "{} word(s) on screen never went through the catalog, so no translation can reach them:\n{}",
            stray.len(),
            stray.join("\n")
        );
    }

    #[test]
    fn every_key_the_code_asks_for_exists() {
        let known = keys_in(catalog(BASE).expect("English is built in").source);
        let asked = keys_asked_for();
        assert!(!asked.is_empty(), "no t! calls found anywhere, so this test is proving nothing");
        let missing: Vec<String> = asked
            .iter()
            .filter(|(key, _)| !known.contains(key))
            .map(|(key, file)| format!("{key} (asked for in {file})"))
            .collect();
        assert!(missing.is_empty(), "keys the code asks for that English doesn't define:\n  {}", missing.join("\n  "));
    }

    /// A translation missing a key is not an error — English shows through. A translation with a
    /// key English has *dropped* is dead weight, and says the catalogs have drifted apart.
    #[test]
    fn no_translation_says_what_english_no_longer_does() {
        let known = keys_in(catalog(BASE).expect("English is built in").source);
        for catalog in CATALOGS.iter().filter(|c| c.tag != BASE) {
            let extra: Vec<String> = keys_in(catalog.source).difference(&known).cloned().collect();
            assert!(extra.is_empty(), "{} defines keys English doesn't: {extra:?}", catalog.tag);
        }
    }

    /// Everything that changes the language, in one test, because the language is global: a test
    /// that set one and left it set would fail the next test for no reason of its own.
    #[test]
    fn the_language_can_be_changed_and_falls_back_to_english() {
        assert_eq!(language(), BASE, "tests match widgets by their English labels, so English has to be the default");

        assert!(!set_language("tlh-Piqd"), "Klingon is not built in");
        assert_eq!(language(), BASE, "a language nobody speaks changes nothing");

        // A catalog written here rather than shipped. It proves the fallback without a
        // translation having to exist yet, and without anyone inventing one in a language they
        // cannot read well enough to check.
        assert!(speak_source("xx-XX", "menu-file = Arquivo\n"), "a catalog that parses is spoken");
        assert_eq!(language(), "xx-XX");
        assert_eq!(text("menu-file"), "Arquivo", "the language being spoken wins");
        assert_eq!(text("menu-edit"), "Edit", "a key the translation lacks shows in English, not blank");

        assert!(!speak_source("yy-YY", "= this is not fluent {"), "a catalog that won't parse is refused");
        assert_eq!(text("menu-file"), "Arquivo", "and refusing it leaves what was being spoken alone");

        assert!(set_language(BASE), "English is built in");
        assert_eq!(language(), BASE);
    }

    #[test]
    fn a_missing_key_comes_back_as_itself() {
        assert_eq!(text("no-such-key-anywhere"), "no-such-key-anywhere");
    }

    #[test]
    fn values_land_in_the_text() {
        let said = t!("about-version", "version" => "0.1.66");
        assert_eq!(said, "WoodSketch 0.1.66");
        assert!(!said.contains('\u{2068}'), "direction marks would sit inside the label: {said:?}");
    }
}
