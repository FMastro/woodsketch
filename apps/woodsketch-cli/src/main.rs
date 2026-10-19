//! `woodsketch-cli`: WoodSketch without the window, for scripts and AI assistants. Builds a
//! drawing from a project recipe (`docs/AGENTS.md`), reads drawings, and writes what the app
//! does: the cut list, cut sheet, build sheets, wood movement check, OBJ/STL and PNG pictures.
//! `woodsketch-cli mcp` serves the same as tools over MCP (stdio), for Claude and other agents.

mod mcp;
mod picture;

use std::path::{Path, PathBuf};

use woodsketch_doc::{Document, cutlist, cutplan, export, movement, recipe};

/// How to write recipes and use the tools, for agents (and people).
pub const GUIDE: &str = include_str!("../../../docs/AGENTS.md");

const USAGE: &str = "woodsketch-cli: WoodSketch from the command line.

  build <recipe.json> [-o drawing.woodsketch]   build a drawing from a project recipe
  summary <drawing>                             size, parts, joints, movement warnings
  cutlist <drawing> [--csv]                     the cut list (text or CSV)
  cutsheet <drawing> -o sheet.html              printable cut sheet: parts, what to buy, diagrams
  buildsheets <drawing> -o sheets.html          a page per part, then how it goes together
  movement <drawing>                            the wood movement check
  export <drawing> -o out.obj|out.stl [--mm]    3D model for other programs
  render <drawing> -o view.png [--view iso|top|front|right] [--angle 30,20] [--size 1200x800]
                  [--dark] [--highlight 'Part name']...
  recipe <drawing> [-o recipe.json]             the drawing as a recipe (to change it and rebuild)
  schema [-o recipe.schema.json]                the recipe format as a JSON Schema
  mcp                                           serve these as MCP tools on stdin/stdout
  guide                                         how to write a recipe (for agents)

A <drawing> is a .woodsketch file, a recipe (.json), or a SketchUp 2021+ model (.skp).";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

/// The command's input and options.
struct Args<'a> {
    input: Option<&'a str>,
    out: Option<&'a str>,
    view: &'a str,
    angle: Option<&'a str>,
    size: [u32; 2],
    csv: bool,
    mm: bool,
    dark: bool,
    highlight: Vec<String>,
}

fn options(rest: &[String]) -> Result<Args<'_>, String> {
    let mut args = Args { input: None, out: None, view: "iso", angle: None, size: [1200, 800], csv: false, mm: false, dark: false, highlight: Vec::new() };
    let mut i = 0;
    while i < rest.len() {
        let value = || rest.get(i + 1).map(String::as_str).ok_or_else(|| format!("{} needs a value.", rest[i]));
        match rest[i].as_str() {
            "-o" | "--out" => {
                args.out = Some(value()?);
                i += 1;
            }
            "--view" => {
                args.view = value()?;
                i += 1;
            }
            "--size" => {
                let size = value()?;
                let (w, h) = size.split_once(['x', 'X']).ok_or_else(|| format!("--size is WIDTHxHEIGHT, like 1200x800 (not \"{size}\")."))?;
                args.size = [w.parse().map_err(|_| "--size: a whole number of pixels")?, h.parse().map_err(|_| "--size: a whole number of pixels")?];
                i += 1;
            }
            "--angle" => {
                args.angle = Some(value()?);
                i += 1;
            }
            "--highlight" => {
                args.highlight.push(value()?.to_owned());
                i += 1;
            }
            "--dark" => args.dark = true,
            "--csv" => args.csv = true,
            "--mm" => args.mm = true,
            other if other.starts_with('-') => return Err(format!("Unknown option {other}.\n\n{USAGE}")),
            other if args.input.is_none() => args.input = Some(other),
            other => return Err(format!("One drawing at a time (also got \"{other}\").")),
        }
        i += 1;
    }
    Ok(args)
}

fn run(args: &[String]) -> Result<(), String> {
    let Some((command, rest)) = args.split_first() else {
        println!("{USAGE}");
        return Ok(());
    };
    let a = options(rest)?;
    let input = || a.input.ok_or_else(|| format!("{command} needs a drawing or recipe.\n\n{USAGE}"));
    let out = || a.out.ok_or_else(|| format!("{command} needs -o <file> to write to."));
    match command.as_str() {
        "build" => {
            let path = input()?;
            let doc = load(Path::new(path))?;
            let to = a.out.map(PathBuf::from).unwrap_or_else(|| Path::new(path).with_extension("woodsketch"));
            save(&doc, &to)?;
            print!("{}", recipe::summary(&doc));
        }
        "summary" => print!("{}", recipe::summary(&load(Path::new(input()?))?)),
        "cutlist" => {
            let doc = load(Path::new(input()?))?;
            let list = cutlist::cut_list(doc.scene());
            print!("{}", if a.csv { cutlist::to_csv(&list) } else { cut_list_text(&list, doc.scene()) });
        }
        "cutsheet" => {
            let doc = load(Path::new(input()?))?;
            write(out()?, cut_sheet(&doc))?;
        }
        "buildsheets" => {
            let doc = load(Path::new(input()?))?;
            let list = cutlist::cut_list(doc.scene());
            write(out()?, woodsketch_doc::buildsheet::to_html(doc.scene(), &list, "Build sheets"))?;
        }
        "movement" => print!("{}", movement_text(&load(Path::new(input()?))?)),
        "export" => {
            let doc = load(Path::new(input()?))?;
            export_to(&doc, Path::new(out()?), a.mm)?;
        }
        "render" => {
            let doc = load(Path::new(input()?))?;
            let options = picture::Options {
                view: picture::view(a.view)?,
                angle: a.angle.map(picture::angle).transpose()?,
                size: a.size,
                dark: a.dark,
                highlight: a.highlight.clone(),
            };
            let png = picture::render(&doc, &options)?;
            std::fs::write(out()?, png).map_err(|e| format!("Couldn't write {}: {e}", out().unwrap_or_default()))?;
            println!("Wrote {}.", out()?);
        }
        "recipe" => {
            let doc = load(Path::new(input()?))?;
            let (recipe, notes) = woodsketch_doc::recipe_of::from_document(&doc);
            let json = serde_json::to_string_pretty(&recipe).map_err(|e| e.to_string())?;
            match a.out {
                Some(to) => write(to, json)?,
                None => println!("{json}"),
            }
            for note in notes {
                eprintln!("Note: {note}");
            }
        }
        "schema" => {
            let json = serde_json::to_string_pretty(&recipe::schema()).map_err(|e| e.to_string())?;
            match a.out {
                Some(to) => write(to, json)?,
                None => println!("{json}"),
            }
        }
        "mcp" => mcp::serve(),
        "guide" => print!("{GUIDE}"),
        "help" | "--help" | "-h" => println!("{USAGE}"),
        "version" | "--version" | "-V" => println!("woodsketch-cli {}", env!("CARGO_PKG_VERSION")),
        other => return Err(format!("Unknown command \"{other}\".\n\n{USAGE}")),
    }
    Ok(())
}

/// A drawing from a `.woodsketch` file, a recipe (`.json`) or a SketchUp model (`.skp`).
pub fn load(path: &Path) -> Result<Document, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if ext == "skp" {
        let bytes = std::fs::read(path).map_err(|e| format!("Couldn't open {}: {e}", path.display()))?;
        let imported = woodsketch_doc::skp::read(&bytes).map_err(|e| e.to_string())?;
        return Ok(Document::from_scene(imported.scene));
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("Couldn't open {}: {e}", path.display()))?;
    if ext == "json" {
        return build(&text);
    }
    let scene = woodsketch_doc::file::from_str(&text).map_err(|e| e.to_string())?;
    Ok(Document::from_scene(scene))
}

/// Build a recipe, its problems listed one per line.
pub fn build(json: &str) -> Result<Document, String> {
    let recipe = recipe::parse(json)?;
    recipe::build(&recipe).map_err(|errors| format!("The recipe has {} problem(s):\n- {}", errors.len(), errors.join("\n- ")))
}

pub fn save(doc: &Document, path: &Path) -> Result<(), String> {
    write(&path.to_string_lossy(), woodsketch_doc::file::to_string(doc.scene()))
}

fn write(path: &str, text: String) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("Couldn't write {path}: {e}"))?;
    println!("Wrote {path}.");
    Ok(())
}

/// The printable cut sheet (parts, what to buy, cutting diagrams).
pub fn cut_sheet(doc: &Document) -> String {
    let list = cutlist::cut_list(doc.scene());
    let plans = cutplan::plan_all(doc.scene(), &list);
    woodsketch_doc::cutsheet::to_html(doc.scene(), &list, &plans, "Cut sheet")
}

/// OBJ (with its MTL beside it) or STL, by the file's extension.
pub fn export_to(doc: &Document, path: &Path, mm: bool) -> Result<(), String> {
    let units = if mm { export::ExportUnits::Millimeters } else { export::ExportUnits::Inches };
    match path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("obj") => {
            let mtl = path.with_extension("mtl");
            let mtl_name = mtl.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let (obj, materials) = export::to_obj(doc.scene(), units, &mtl_name);
            write(&path.to_string_lossy(), obj)?;
            write(&mtl.to_string_lossy(), materials)
        }
        Some("stl") => write(&path.to_string_lossy(), export::to_stl(doc.scene(), units)),
        _ => Err("Export to a .obj or .stl file.".to_owned()),
    }
}

/// The cut list as plain text: each material's parts, then the hardware.
pub fn cut_list_text(list: &cutlist::CutList, scene: &woodsketch_doc::Scene) -> String {
    use woodsketch_doc::units::format_length;
    let mut out = String::new();
    for m in &list.materials {
        out.push_str(&format!("{}\n", m.name));
        for p in &m.parts {
            out.push_str(&format!(
                "  {}  {} x{}  {} x {} x {}\n",
                p.label,
                p.names.join(", "),
                p.count,
                format_length(p.length),
                format_length(p.width),
                format_length(p.thickness)
            ));
        }
    }
    let hardware = list.hardware(scene);
    if !hardware.is_empty() {
        out.push_str("Hardware\n");
        for (name, count) in &hardware {
            out.push_str(&format!("  {count} x {name}\n"));
        }
    }
    if out.is_empty() {
        out.push_str("No parts yet.\n");
    }
    out
}

/// The wood movement check as plain text.
pub fn movement_text(doc: &Document) -> String {
    use woodsketch_doc::units::format_length;
    let scene = doc.scene();
    let report = movement::check(scene, scene.moisture_swing());
    let mut out = String::new();
    for part in &report.parts {
        out.push_str(&format!("{} ({}): {} wide, moves about {}\n", part.name, part.material, format_length(part.width), format_length(part.movement)));
    }
    for c in &report.conflicts {
        out.push_str(&format!("WARNING: {}\n", c.describe()));
    }
    for a in &report.allowed {
        out.push_str(&format!("OK: {}\n", a.describe()));
    }
    if !report.unknown.is_empty() {
        out.push_str(&format!("No movement data for: {}\n", report.unknown.join(", ")));
    }
    if out.is_empty() {
        out.push_str("No solid-wood parts.\n");
    }
    out
}
