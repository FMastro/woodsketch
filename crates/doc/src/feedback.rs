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
    /// `windows x86_64`.
    pub os: String,
    /// The graphics card and driver, as wgpu reports them.
    pub graphics: String,
    pub units: String,
}

impl System {
    /// This build's OS and processor, with the rest given.
    pub fn here(version: &str, graphics: &str, units: &str) -> Self {
        Self { version: version.to_owned(), os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH), graphics: graphics.to_owned(), units: units.to_owned() }
    }
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
pub fn details(system: &System, drawing: Option<&Drawing>, last_error: Option<&str>) -> String {
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
        let text = details(&system, Some(&drawing), Some("Something failed"));
        assert!(text.contains("WoodSketch 0.1.21") && text.contains("Graphics: NVIDIA") && text.contains("8 joints") && text.contains("Last error shown: Something failed"), "{text}");
        assert!(!text.contains("Cherry") && !text.contains("Leg"), "no names of materials or parts");
        let url = issue_url("https://github.com/FMastro/woodsketch", Kind::Problem, "Crash & burn", "It closed.", &text);
        assert!(url.starts_with("https://github.com/FMastro/woodsketch/issues/new?template=bug.yml&title=Crash%20%26%20burn&what=It%20closed.&details=WoodSketch%200.1.21%0A"), "{url}");
        assert!(issue_url("r", Kind::Idea, "", "", "").contains("template=idea.yml"));
    }
}
