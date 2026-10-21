//! Reporting a problem or an idea: a new GitHub issue, filled in with what helps fix it and
//! nothing private. It carries the version, the system and graphics card, the units, the last
//! error shown (with paths and the user's name taken out), and counts from the open drawing
//! (how many parts, faces, joints...), never names of files, parts or materials, or the drawing
//! itself. The user sees all of it before anything is sent, and sends it from GitHub.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::{MaterialKind, Scene};

/// What's being reported.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    /// Something went wrong: a bug.
    #[default]
    Problem,
    /// A feature idea or other feedback.
    Idea,
}

impl Kind {
    /// The issue form in `.github/ISSUE_TEMPLATE`.
    fn template(self) -> &'static str {
        match self {
            Self::Problem => "bug.yml",
            Self::Idea => "idea.yml",
        }
    }
}

/// The app and the computer it runs on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct System {
    pub version: String,
    /// `Windows 11 Pro 10.0.26100 (x86_64)`.
    pub os: String,
    /// The graphics card and driver, as wgpu reports them.
    pub graphics: String,
    pub units: String,
}

impl System {
    /// This computer's OS, its version and the processor, with the rest given.
    pub fn here(version: &str, graphics: &str, units: &str) -> Self {
        Self { version: version.to_owned(), os: os_name(), graphics: graphics.to_owned(), units: units.to_owned() }
    }
}

/// The OS with its version, and the processor: `Windows 11 Pro 10.0.26100 (x86_64)`, `Mac OS
/// 15.1.0 (aarch64)`. Looked up once.
pub fn os_name() -> String {
    static NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    NAME.get_or_init(|| {
        let info = os_info::get();
        let kind = info.os_type().to_string();
        // Windows names its edition in full (`Windows 11 Pro`); elsewhere it's an extra word.
        let name = match info.edition() {
            Some(edition) if edition.starts_with(&kind) => edition.to_owned(),
            Some(edition) => format!("{kind} {edition}"),
            None => kind,
        };
        format!("{name} {} ({})", info.version(), std::env::consts::ARCH)
    })
    .clone()
}

/// What a crash left: the message (paths taken out), where in WoodSketch's code it happened, the
/// calls that led there (WoodSketch's own, innermost first: see [`own_frames`]), and the version
/// that crashed. Saved when it happens, offered for a report at the next start.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Crash {
    pub message: String,
    pub location: String,
    pub frames: Vec<String>,
    pub version: String,
}

impl Crash {
    /// As saved: the version, the location, a line `in <frame>` for each frame, then the message.
    pub fn to_text(&self) -> String {
        let mut s = format!("{}\n{}\n", self.version, self.location);
        for frame in &self.frames {
            let _ = writeln!(s, "in {frame}");
        }
        s.push_str(&self.message);
        s
    }

    pub fn from_text(text: &str) -> Option<Crash> {
        let mut lines = text.lines();
        let (version, location) = (lines.next()?.trim().to_owned(), lines.next()?.trim().to_owned());
        let rest: Vec<&str> = lines.collect();
        let frames: Vec<String> = rest.iter().map_while(|l| l.strip_prefix("in ")).map(|f| f.trim().to_owned()).collect();
        let message = rest[frames.len()..].join("\n").trim().to_owned();
        Some(Crash { message, location, frames, version })
    }

    /// A title for its report: the message's first line, shortened.
    pub fn title(&self) -> String {
        let first = self.message.lines().next().unwrap_or_default();
        let short: String = first.chars().take(60).collect();
        format!("Crash: {short}{}", if first.chars().count() > 60 { "..." } else { "" })
    }
}

/// The frames of a backtrace (as `std::backtrace::Backtrace` prints one) in WoodSketch's own code,
/// innermost first, at most `most`: `woodsketch_doc::joinery::rebuilt_without
/// (crates/doc/src/joinery.rs:120:9)`, the place only when the build has line numbers. The
/// libraries' and the standard library's frames, and the crash handler's own, are left out.
pub fn own_frames(backtrace: &str, most: usize) -> Vec<String> {
    let mut frames = Vec::new();
    let mut lines = backtrace.lines().peekable();
    while let Some(line) = lines.next() {
        // `  12: name`, then perhaps `             at file:line:column`.
        let Some((number, name)) = line.trim().split_once(": ") else { continue };
        if number.parse::<u32>().is_err() {
            continue;
        }
        let at = lines.peek().and_then(|l| l.trim().strip_prefix("at ")).map(str::to_owned);
        if at.is_some() {
            lines.next();
        }
        let name = without_hash(name.trim());
        let own = name.trim_start_matches('<').starts_with("woodsketch");
        if !own || name.contains("::crash::") {
            continue;
        }
        let place = at.and_then(|at| {
            let mut parts = at.rsplitn(3, ':');
            let (column, line, file) = (parts.next()?.parse().ok()?, parts.next()?.parse().ok()?, parts.next()?);
            Some(source_location(file, line, column))
        });
        frames.push(match place {
            Some(place) => format!("{name} ({place})"),
            None => name.to_owned(),
        });
        if frames.len() == most {
            break;
        }
    }
    frames
}

/// A symbol without the hash on its end (`::h0123456789abcdef`).
fn without_hash(name: &str) -> &str {
    match name.rsplit_once("::h") {
        Some((front, hash)) if hash.len() == 16 && hash.chars().all(|c| c.is_ascii_hexdigit()) => front,
        _ => name,
    }
}

/// Where in the code a crash happened, without the build machine's folders: from `crates/` or
/// `apps/` for WoodSketch's own code, from the crate's folder for a library's.
pub fn source_location(file: &str, line: u32, column: u32) -> String {
    let file = file.replace('\\', "/");
    let parts: Vec<&str> = file.split('/').collect();
    let own = parts.iter().position(|p| *p == "crates" || *p == "apps");
    // A library's: `.../registry/src/<index>/<crate>-<version>/src/lib.rs`, from `<crate>-<version>`.
    let library = parts.iter().rposition(|p| *p == "src").filter(|&at| at > 0);
    let short = match own.or(library.map(|at| at - 1)) {
        Some(from) => parts[from..].join("/"),
        None => parts.last().copied().unwrap_or_default().to_owned(),
    };
    format!("{short}:{line}:{column}")
}

/// How big and what's in the open drawing, counted, without names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Drawing {
    pub groups_and_components: usize,
    pub placed: usize,
    pub faces: usize,
    pub edges: usize,
    pub joints: usize,
    pub profiles: usize,
    pub templates: usize,
    pub hardware: usize,
    pub tags: usize,
    /// How many materials of each kind (`Solid wood: 2`).
    pub materials: BTreeMap<&'static str, usize>,
}

impl Drawing {
    pub fn of(scene: &Scene) -> Self {
        let mut d = Self::default();
        let mut faces = scene.root.model.faces().count();
        let mut edges = scene.root.model.edges().count();
        let mut placed = scene.root.instances().count();
        for (_, def) in scene.definitions() {
            d.groups_and_components += 1;
            faces += def.container.model.faces().count();
            edges += def.container.model.edges().count();
            placed += def.container.instances().count();
            d.templates += usize::from(def.template.is_some());
            d.hardware += usize::from(def.hardware.is_some());
        }
        (d.faces, d.edges, d.placed) = (faces, edges, placed);
        d.joints = scene.joints.len();
        d.profiles = scene.profiles.len();
        d.tags = scene.tags().count();
        for (_, m) in scene.materials() {
            let kind = match m.kind {
                MaterialKind::SolidWood => "solid wood",
                MaterialKind::DimensionalLumber => "lumber",
                MaterialKind::SheetGood => "sheet goods",
                MaterialKind::Hardware => "hardware",
                MaterialKind::Other => "other",
            };
            *d.materials.entry(kind).or_default() += 1;
        }
        d
    }

    fn describe(&self) -> String {
        let materials = if self.materials.is_empty() {
            "none".to_owned()
        } else {
            self.materials.iter().map(|(kind, n)| format!("{n} {kind}")).collect::<Vec<_>>().join(", ")
        };
        format!(
            "{} groups/components ({} placed), {} faces, {} edges, {} joints, {} profiles, {} templates, {} hardware, {} tags; materials: {materials}",
            self.groups_and_components, self.placed, self.faces, self.edges, self.joints, self.profiles, self.templates, self.hardware, self.tags
        )
    }
}

/// `text` with anything private taken out: the home folder and the user's name, and any file
/// path (a word with a folder separator or a drive letter) becomes `[path]`.
pub fn scrub(text: &str, home: Option<&str>, user: Option<&str>) -> String {
    let mut text = text.to_owned();
    for (private, stand_in) in [(home, "~"), (user, "[user]")] {
        if let Some(private) = private.filter(|p| p.len() > 1) {
            text = text.replace(private, stand_in);
        }
    }
    // Paths, quoted or not: whole words that look like one.
    let mut out = String::new();
    for (i, word) in text.split(' ').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let bare = word.trim_matches(|c: char| matches!(c, '"' | '\'' | '(' | ')' | ',' | '.' | ':' | ';'));
        let drive = bare.len() > 2 && bare.as_bytes()[1] == b':' && bare.as_bytes()[0].is_ascii_alphabetic();
        // A fraction (3/4) has one slash; a path starts with one or has two or more.
        let slashes = bare.matches('/').count();
        let path = drive || bare.contains('\\') || (!bare.starts_with("http") && (bare.starts_with('/') || slashes >= 2));
        if path && !bare.is_empty() {
            out.push_str(&word.replace(bare, "[path]"));
        } else {
            out.push_str(word);
        }
    }
    out
}

/// The details that go with a report, as shown to the user and sent.
pub fn details(system: &System, drawing: Option<&Drawing>, last_error: Option<&str>, crash: Option<&Crash>) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "WoodSketch {}", system.version);
    let _ = writeln!(s, "System: {}", system.os);
    let _ = writeln!(s, "Graphics: {}", system.graphics);
    let _ = writeln!(s, "Units: {}", system.units);
    if let Some(drawing) = drawing {
        let _ = writeln!(s, "Drawing: {}", drawing.describe());
    }
    if let Some(error) = last_error {
        let _ = writeln!(s, "Last error shown: {error}");
    }
    if let Some(crash) = crash {
        let _ = writeln!(s, "Crashed (in {}): {}", crash.version, crash.message);
        let _ = writeln!(s, "  at {}", crash.location);
        for frame in &crash.frames {
            let _ = writeln!(s, "  from {frame}");
        }
    }
    s
}

/// The address of a new issue on `repo` (`https://github.com/owner/name`), its form filled in.
pub fn issue_url(repo: &str, kind: Kind, title: &str, what: &str, details: &str) -> String {
    format!("{repo}/issues/new?template={}&title={}&what={}&details={}", kind.template(), encode(title), encode(what), encode(details))
}

/// Where the app sends a report for someone without a GitHub account: woodsketch.net makes the
/// issue (with a token only it has) and emails a left address to the developer, privately.
pub const RELAY: &str = "https://woodsketch.net/api/report";

/// A plausible email address: something, one @, something with a dot, no spaces.
pub fn valid_email(text: &str) -> bool {
    let text = text.trim();
    let Some((name, domain)) = text.split_once('@') else { return false };
    !name.is_empty() && !domain.contains('@') && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.') && !text.contains(char::is_whitespace) && text.len() <= 200
}

/// The report as the relay takes it (JSON). The email, if any, never goes in the public issue.
pub fn request(kind: Kind, title: &str, what: &str, details: &str, email: Option<&str>) -> String {
    let kind = match kind {
        Kind::Problem => "problem",
        Kind::Idea => "idea",
    };
    serde_json::json!({ "kind": kind, "title": title, "what": what, "details": details, "email": email.unwrap_or("") }).to_string()
}

/// A report the relay took: its issue.
#[derive(Clone, Debug, PartialEq)]
pub struct Sent {
    pub number: u64,
    pub url: String,
    /// The left email reached the developer.
    pub emailed: bool,
}

/// What the relay answered (its HTTP status and body): the issue, or why not, said for the user.
pub fn reply(status: u16, body: &str) -> Result<Sent, String> {
    let json: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    if status == 201
        && let (Some(number), Some(url)) = (json["number"].as_u64(), json["url"].as_str())
    {
        return Ok(Sent { number, url: url.to_owned(), emailed: json["emailed"].as_bool().unwrap_or(false) });
    }
    Err(json["error"].as_str().map_or_else(|| format!("The report didn't go through (error {status}). Try again, or use Open on GitHub."), str::to_owned))
}

/// Percent-encoding for a URL's query: everything but letters, digits and `-_.~`.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_and_names_are_taken_out() {
        let error = r#"Couldn't open "C:\Users\fred\Dropbox\Woodworking\drawer.woodsketch": access denied"#;
        assert_eq!(scrub(error, Some(r"C:\Users\fred"), Some("fred")), r#"Couldn't open "[path]": access denied"#);
        assert_eq!(scrub("Saved to /home/fred/bench.woodsketch.", None, Some("fred")), "Saved to [path].");
        assert_eq!(scrub("The cabinet is too shallow for a drawer (10\" slides at least).", None, None), "The cabinet is too shallow for a drawer (10\" slides at least).");
        assert_eq!(scrub("fred's table", None, Some("fred")), "[user]'s table");
        assert_eq!(scrub("A 3/4\" dado, ~ 1/16\" off; see https://woodsketch.net/help", None, None), "A 3/4\" dado, ~ 1/16\" off; see https://woodsketch.net/help", "fractions and links stay");
    }

    #[test]
    fn a_crash_says_where_without_the_build_machines_folders() {
        assert_eq!(source_location(r"D:\a\woodsketch\woodsketch\crates\doc\src\joinery.rs", 120, 9), "crates/doc/src/joinery.rs:120:9");
        assert_eq!(source_location("apps/woodsketch/src/main.rs", 3, 1), "apps/woodsketch/src/main.rs:3:1");
        assert_eq!(
            source_location(r"C:\Users\runneradmin\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\wgpu-30.0.0\src\api\device.rs", 50, 5),
            "wgpu-30.0.0/src/api/device.rs:50:5"
        );
        assert_eq!(source_location("lib.rs", 1, 1), "lib.rs:1:1");
        let crash = Crash {
            message: "index out of bounds: the len is 3 but the index is 7\nsecond line".into(),
            location: "crates/doc/src/model.rs:12:5".into(),
            frames: vec!["woodsketch_doc::model::Model::push_pull (crates/doc/src/push_pull.rs:62:13)".into(), "woodsketch_tools::Tools::key".into()],
            version: "0.1.47".into(),
        };
        assert_eq!(Crash::from_text(&crash.to_text()), Some(crash.clone()));
        assert_eq!(crash.title(), "Crash: index out of bounds: the len is 3 but the index is 7");
        let text = details(&System::here("0.1.47", "GPU", "Inches"), None, None, Some(&crash));
        assert!(text.contains("Crashed (in 0.1.47): index out of bounds") && text.contains("  at crates/doc/src/model.rs:12:5"), "{text}");
        assert!(text.contains("  from woodsketch_doc::model::Model::push_pull (crates/doc/src/push_pull.rs:62:13)\n  from woodsketch_tools::Tools::key"), "{text}");
        // Saved by the version before frames were kept: still read.
        let old = Crash::from_text("0.1.47\ncrates/doc/src/model.rs:12:5\nboom").unwrap();
        assert_eq!((old.message.as_str(), old.frames.len()), ("boom", 0));
        // The OS with a version, and the processor.
        let os = os_name();
        assert!(os.contains(std::env::consts::ARCH), "{os}");
        println!("{os}");
    }

    #[test]
    fn a_backtrace_keeps_woodsketchs_own_calls() {
        let windows = r"   0: std::backtrace_rs::backtrace::win64::trace
             at /rustc/abc/library/std/src/../../backtrace/src/backtrace/win64.rs:85
   1: woodsketch_ui_egui::crash::install::closure$0
   2: std::panicking::rust_panic_with_hook
   3: woodsketch_doc::joinery::rebuilt_without::h0123456789abcdef
             at D:\a\woodsketch\woodsketch\crates\doc\src\joinery.rs:980:17
   4: <woodsketch_tools::Tools as core::default::Default>::default
   5: eframe::native::run::run_and_return
   6: woodsketch::main
             at .\apps\woodsketch\src\main.rs:40:5";
        assert_eq!(
            own_frames(windows, 12),
            [
                "woodsketch_doc::joinery::rebuilt_without (crates/doc/src/joinery.rs:980:17)",
                "<woodsketch_tools::Tools as core::default::Default>::default",
                "woodsketch::main (apps/woodsketch/src/main.rs:40:5)",
            ]
        );
        assert_eq!(own_frames(windows, 1).len(), 1, "at most as many as asked");
        // A real one, from here (an optimized build without line tables has this inlined away).
        let here = own_frames(&std::backtrace::Backtrace::force_capture().to_string(), 12);
        assert!(!cfg!(debug_assertions) || here.iter().any(|f| f.contains("a_backtrace_keeps_woodsketchs_own_calls")), "{here:?}");
    }

    #[test]
    fn a_report_for_the_relay_and_its_answer() {
        let json: serde_json::Value = serde_json::from_str(&request(Kind::Idea, "Bed", "A bed template", "WoodSketch 0.1.23", Some("maker@example.com"))).unwrap();
        assert_eq!((json["kind"].as_str(), json["what"].as_str(), json["email"].as_str()), (Some("idea"), Some("A bed template"), Some("maker@example.com")));
        assert_eq!(
            reply(201, r#"{"number":42,"url":"https://github.com/FMastro/woodsketch/issues/42","emailed":true}"#),
            Ok(Sent { number: 42, url: "https://github.com/FMastro/woodsketch/issues/42".into(), emailed: true })
        );
        assert_eq!(reply(429, r#"{"error":"That's a lot of reports"}"#), Err("That's a lot of reports".into()));
        assert!(reply(500, "<html>").unwrap_err().contains("error 500"));
        assert!(valid_email("maker@example.com") && !valid_email("maker@example") && !valid_email("no at.com") && !valid_email("a b@c.com") && !valid_email("@c.com"));
    }

    #[test]
    fn a_report_counts_the_drawing_and_fills_in_the_issue_form() {
        let mut doc = crate::Document::default();
        let cherry = doc.add_preset(crate::library::LIBRARY.iter().find(|p| p.name == "Cherry").unwrap());
        doc.add_template(&crate::template::Template::Table(crate::template::Table::default()), Some(cherry)).unwrap();
        let drawing = Drawing::of(doc.scene());
        assert_eq!((drawing.joints, drawing.templates), (8, 1));
        assert_eq!(drawing.materials.get("solid wood"), Some(&1));
        let system = System::here("0.1.21", "NVIDIA RTX 4060 (Vulkan, driver 560)", "Inches");
        let text = details(&system, Some(&drawing), Some("Something failed"), None);
        assert!(text.contains("WoodSketch 0.1.21") && text.contains("Graphics: NVIDIA") && text.contains("8 joints") && text.contains("Last error shown: Something failed"), "{text}");
        assert!(!text.contains("Cherry") && !text.contains("Leg"), "no names of materials or parts");
        let url = issue_url("https://github.com/FMastro/woodsketch", Kind::Problem, "Crash & burn", "It closed.", &text);
        assert!(url.starts_with("https://github.com/FMastro/woodsketch/issues/new?template=bug.yml&title=Crash%20%26%20burn&what=It%20closed.&details=WoodSketch%200.1.21%0A"), "{url}");
        assert!(issue_url("r", Kind::Idea, "", "", "").contains("template=idea.yml"));
    }
}
