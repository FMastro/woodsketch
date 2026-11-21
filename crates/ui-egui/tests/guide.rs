//! The User Guide's reference pages, written from what the app actually does.
//!
//! `docs/guide` is an mdBook. Its prose is written by hand; everything under
//! `docs/guide/src/reference/` is written by this test, because a list of tools, shortcuts,
//! joints, profiles, templates, materials and species is exactly the kind of page that goes
//! quietly wrong on a release nobody checked it against. The app already knows all of it.
//!
//! The pages are committed, so the book builds without running the tests and a changed shortcut
//! shows up in a diff like any other change. This test is what keeps them honest: it builds each
//! page again, and where one differs from the file on disk it **writes the new one** and then
//! fails, so the fix is to look at the diff and commit it.
//!
//! What stops a new feature slipping past is the `match` in each `about` function. They are
//! exhaustive, so adding a `ToolId`, a `JointKind`, a `Profile` or a `Template` stops this file
//! compiling until the guide has a line about it. That is the rule from CLAUDE.md -- every
//! feature ships with its help page -- held by the compiler rather than by memory.
//!
//! This lives in `crates/*/tests`, which `check-unreleased.py` treats as shipping to nobody, so
//! guide work needs no version bump. It reads only public API; nothing here reaches into the UI.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use woodsketch_doc::MaterialKind;
use woodsketch_doc::cutplan::lumber_sizes;
use woodsketch_doc::joinery::JointKind;
use woodsketch_doc::library::{LIBRARY, Preset};
use woodsketch_doc::movement::{self, DEFAULT_SWING};
use woodsketch_doc::profile::Profile;
use woodsketch_doc::template::{CaseJoints, DrawerJoints, FrameJoints, Template};
use woodsketch_doc::units::format_length;
use woodsketch_tools::ToolId;

const WARNING: &str = "<!-- Written by crates/ui-egui/tests/guide.rs from the app itself. Don't edit it by hand:\n     change the app and run `cargo test -p woodsketch-ui-egui --test guide`, which rewrites it. -->";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn guide() -> PathBuf {
    root().join("docs/guide")
}

/// Everything this test writes, by its path under `docs/guide/`.
fn pages() -> Vec<(&'static str, String)> {
    vec![
        ("src/reference/tools.md", tools()),
        ("src/reference/joints.md", joints()),
        ("src/reference/profiles.md", profiles()),
        ("src/reference/templates.md", templates()),
        ("src/reference/materials.md", materials()),
        ("src/reference/wood.md", wood()),
        (STYLESHEET, icon_style()),
    ]
}

/// Where the icon stylesheet is written, and so -- since mdBook mirrors the path into the output
/// while `src/icons/` lands at the book's root -- how far its mask urls have to climb to reach
/// the artwork. Worked out rather than written down: the first try put the stylesheet one level
/// deeper than its urls assumed, and a mask that resolves to nothing is an empty square on the
/// page rather than an error anywhere.
const STYLESHEET: &str = "assets/icons.css";

fn to_the_icons() -> String {
    "../".repeat(STYLESHEET.matches('/').count())
}

// ------------------------------------------------------------------ toolbar icons

/// Which Phosphor glyph each tool's button carries, read out of `ui_egui::icons`: `ToolId::Board
/// => ph::LOG` becomes `Board -> log`, which is the name of Phosphor's own SVG for it. Read
/// rather than repeated, so the icon beside a tool in the guide is the icon on its button; the
/// artwork itself is vendored under `docs/guide/src/icons/` from the same MIT set the app draws
/// with (`egui-phosphor`).
fn icon_names() -> Vec<(String, String)> {
    let text = std::fs::read_to_string(root().join("crates/ui-egui/src/icons.rs")).expect("the icons module");
    text.lines()
        .filter_map(|line| {
            let (left, right) = line.split_once("=> ph::")?;
            let tool = left.trim().strip_prefix("ToolId::")?.trim();
            let glyph = right.trim().trim_end_matches(',').trim();
            let shouting = |c: char| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_';
            (!glyph.is_empty() && glyph.chars().all(shouting)).then(|| (tool.to_owned(), glyph.to_lowercase().replace('_', "-")))
        })
        .collect()
}

fn icon_of(tool: ToolId, icons: &[(String, String)]) -> String {
    let variant = format!("{tool:?}");
    icons
        .iter()
        .find(|(named, _)| *named == variant)
        .map(|(_, icon)| icon.clone())
        .unwrap_or_else(|| panic!("ui_egui::icons has no toolbar icon for ToolId::{variant}, so the guide has nothing to show beside it"))
}

/// One of the three colours `icons::paint` draws with, as CSS. Read from the same place for the
/// same reason as the glyphs: the guide's icons should recolour when the toolbar's do.
fn icon_colour(name: &str) -> String {
    let text = std::fs::read_to_string(root().join("crates/ui-egui/src/icons.rs")).expect("the icons module");
    let at = text.find(&format!("const {name}: egui::Color32")).unwrap_or_else(|| panic!("no {name} colour in ui_egui::icons"));
    let open = text[at..].find("from_rgb(").expect("the colour's channels") + at + "from_rgb(".len();
    let close = text[open..].find(')').expect("the end of the colour") + open;
    let channels: Vec<u8> = text[open..close].split(',').filter_map(|part| u8::from_str_radix(part.trim().trim_start_matches("0x"), 16).ok()).collect();
    assert_eq!(channels.len(), 3, "{name} isn't three channels");
    format!("#{:02x}{:02x}{:02x}", channels[0], channels[1], channels[2])
}

/// The stylesheet that stacks the two weights the way `icons::paint` does: the filled glyph in
/// amber behind, the outline in walnut over it. They are drawn as masks rather than images so
/// the colours are the app's rather than whatever is baked into an SVG, and so one file serves
/// both of mdBook's light themes and all three of its dark ones.
fn icon_style() -> String {
    let (outline, outline_dark, accent) = (icon_colour("OUTLINE"), icon_colour("OUTLINE_DARK"), icon_colour("ACCENT"));
    let mut out = String::new();
    writeln!(out, "/* {} */", WARNING.replace("<!-- ", "").replace("\n     ", "\n   ").replace(" -->", "")).unwrap();
    writeln!(
        out,
        concat!(
            "\n.ws-icon {{ position: relative; display: inline-block; width: 1.4em; height: 1.4em; vertical-align: -0.35em; flex: none; }}\n",
            ".ws-icon::before, .ws-icon::after {{\n",
            "  content: \"\"; position: absolute; inset: 0;\n",
            "  -webkit-mask-repeat: no-repeat; mask-repeat: no-repeat;\n",
            "  -webkit-mask-position: center; mask-position: center;\n",
            "  -webkit-mask-size: contain; mask-size: contain;\n",
            "}}\n",
            "/* The filled glyph, behind; `paint` fades it to 0.6 of the accent in a light theme. */\n",
            ".ws-icon::before {{ background: {accent}; opacity: 0.6; }}\n",
            ".ws-icon::after {{ background: {outline}; }}\n",
            "/* mdBook's dark themes, where `paint` lightens the outline and lifts the accent to 0.75. */\n",
            ".navy .ws-icon::after, .ayu .ws-icon::after, .coffee .ws-icon::after {{ background: {outline_dark}; }}\n",
            ".navy .ws-icon::before, .ayu .ws-icon::before, .coffee .ws-icon::before {{ opacity: 0.75; }}\n"
        ),
        accent = accent,
        outline = outline,
        outline_dark = outline_dark
    )
    .unwrap();
    let up = to_the_icons();
    let icons = icon_names();
    let mut seen: Vec<String> = ToolId::ALL.into_iter().map(|tool| icon_of(tool, &icons)).collect();
    seen.sort();
    seen.dedup();
    out.push('\n');
    for icon in seen {
        let (fill, outline) = (format!("{up}icons/fill/{icon}.svg"), format!("{up}icons/regular/{icon}.svg"));
        writeln!(out, ".ws-icon-{icon}::before {{ -webkit-mask-image: url({fill}); mask-image: url({fill}); }}").unwrap();
        writeln!(out, ".ws-icon-{icon}::after {{ -webkit-mask-image: url({outline}); mask-image: url({outline}); }}").unwrap();
    }
    out
}

// ------------------------------------------------------------------ tools

const GROUPS: [&str; 6] = ["Principal", "Drawing", "Shapes", "Modifying", "Measuring and marking", "Looking around"];

/// Which part of the toolbar a tool belongs to, as SketchUp groups them.
fn group(tool: ToolId) -> &'static str {
    match tool {
        ToolId::Select | ToolId::Eraser | ToolId::PaintBucket => "Principal",
        ToolId::Line | ToolId::Arc | ToolId::CenterArc | ToolId::ThreePointArc | ToolId::Pie => "Drawing",
        ToolId::Rectangle | ToolId::RotatedRectangle | ToolId::Circle | ToolId::Polygon | ToolId::Board => "Shapes",
        ToolId::PushPull | ToolId::Move | ToolId::Rotate | ToolId::Scale | ToolId::Offset => "Modifying",
        ToolId::TapeMeasure | ToolId::Protractor | ToolId::Dimension | ToolId::Text => "Measuring and marking",
        ToolId::Orbit | ToolId::Pan | ToolId::Zoom | ToolId::Walk => "Looking around",
    }
}

/// One line per tool: what it is for, and what it does with a typed value.
fn tool_about(tool: ToolId) -> &'static str {
    match tool {
        ToolId::Select => "Pick things. Click, shift-click to add or remove, drag a box. Double-click a group to edit inside it.",
        ToolId::Eraser => "Erase the edge, guide, group or component you click, or everything you drag over, as one undo step.",
        ToolId::PaintBucket => "Paint a material onto a whole object; Ctrl paints one face, Alt samples the one under the cursor.",
        ToolId::Line => "Draw edges point to point. A closed loop makes a face; a line across a face splits it. Type each length.",
        ToolId::Arc => "An arc through two endpoints and a bulge. Type `12s` for the number of segments, `4r` for a radius.",
        ToolId::CenterArc => "An arc from its center: the center, then the start, then the sweep.",
        ToolId::ThreePointArc => "An arc through three points you click.",
        ToolId::Pie => "A filled wedge: center, radius, then the angle.",
        ToolId::Rectangle => "A rectangle on a face or in free space. Type `36x11 1/4` to size it, before or after drawing.",
        ToolId::RotatedRectangle => "A rectangle turned off the axes: two corners set the baseline, the third the width.",
        ToolId::Circle => "A circle, drawn as edges so it splits the face under it. Type `24s` to change the number of sides.",
        ToolId::Polygon => "A regular polygon from its center. Type the number of sides.",
        ToolId::Board => {
            "Draw in real stock: pick a material and one of its standard sizes in the Board panel, then drag or type the size. Lumber drags out its length; sheets and solid wood drag a rectangle at their thickness."
        }
        ToolId::PushPull => {
            "Push or pull a face. Makes a solid from a free face, cuts a rabbet or notch where a wall is flush, and cuts through when pushed to the far face. Double-click repeats the last distance."
        }
        ToolId::Move => {
            "Move or copy. Locks to an axis as you push along one, or with the arrow keys. Ctrl-tap to copy; `3x` makes an array, `/3` divides the move. Type a distance, or `x,y,z`."
        }
        ToolId::Rotate => {
            "Turn things about a point: the protractor sets the axis, snapping at 15 degrees and onto red, green or blue. Type the angle; Ctrl-tap to copy, `3x` for a rotational array."
        }
        ToolId::Scale => "Resize with 26 grips on the bounding box. The opposite grip stays put, or Ctrl holds the center. Type factors, or a finished size.",
        ToolId::Offset => "Offset a face's outline in or out, mitering the corners -- an inward offset makes a frame and a panel. Type the distance.",
        ToolId::TapeMeasure => {
            "Measure, and leave a guide behind: a parallel guide line from an edge, a guide point from a point. Ctrl measures without leaving anything."
        }
        ToolId::Protractor => "Measure an angle, and leave a guide line at it.",
        ToolId::Dimension => "A dimension between two points, placed by moving out and clicking, or by typing how far out.",
        ToolId::Text => "A note on the drawing. It starts with something useful -- a part's name, an edge's length -- and typing replaces it.",
        ToolId::Orbit => "Turn the view around the drawing.",
        ToolId::Pan => "Slide the view.",
        ToolId::Zoom => "Zoom in and out. Shift+Z zooms to fit everything.",
        ToolId::Walk => {
            "Walk through the drawing like a game: the mouse looks, W A S D or the arrows walk, Shift is faster, E and Q go up and down, F flies, the wheel sets the speed. Type a length to change eye height."
        }
    }
}

fn tools() -> String {
    let mut out = String::new();
    writeln!(out, "# Tools and keys\n\n{WARNING}\n").unwrap();
    out.push_str(concat!(
        "Every tool, its key and what it does. The keys are SketchUp's, so they should already be in\n",
        "your fingers; the ones with no key are reached from the toolbar or the Tools menu.\n\n",
        "**You never have to click into a box to type a number.** Start typing and it goes to the\n",
        "Measurements box at the bottom of the window: `Enter` commits it, `Esc` clears it. Type\n",
        "*after* finishing a shape to correct it. A comma is a decimal point (`38,5`), so a list of\n",
        "lengths is separated by a semicolon (`5,5';6'`). `Space` is Select and `Esc` cancels whatever\n",
        "is in hand, which between them get you out of anything.\n",
    ));
    let icons = icon_names();
    for name in GROUPS {
        writeln!(out, "\n## {name}\n").unwrap();
        out.push_str("| | Tool | Key | What it does |\n|---|---|---|---|\n");
        for tool in ToolId::ALL.into_iter().filter(|&tool| group(tool) == name) {
            let key = tool.shortcut_label();
            let key = if key.is_empty() { "--".to_owned() } else { format!("`{key}`") };
            let icon = format!("<span class=\"ws-icon ws-icon-{}\"></span>", icon_of(tool, &icons));
            writeln!(out, "| {icon} | **{}** | {key} | {} |", tool.name(), tool_about(tool)).unwrap();
        }
    }
    out.push_str("\n## Keys that aren't tools\n\n");
    out.push_str(concat!(
        "| Key | What it does |\n|---|---|\n",
        "| `Space` | Select |\n",
        "| `Shift+Z` | Zoom to fit everything |\n",
        "| `G` | Group the selection |\n",
        "| `Ctrl+G` | Make a component of the selection |\n",
        "| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copy, cut, paste -- a paste is carried on the cursor until you click |\n",
        "| `Ctrl+D` | Duplicate |\n",
        "| `Ctrl+Z` / `Ctrl+Y` | Undo, redo |\n",
        "| `Ctrl+N` / `Ctrl+O` / `Ctrl+S` | New, open, save (`Ctrl+Shift+S` saves as) |\n",
        "| `Delete` | Erase the selection |\n",
        "| Arrow keys | Lock the tool in hand to red, green or blue |\n",
        "| `Esc` | Clear what you typed, cancel what's in hand, or deselect |\n",
        "| Double-click | Edit inside a group; with Push/Pull, repeat the last distance |\n",
    ));
    out
}

// ------------------------------------------------------------------ joints

const JOINTS: [JointKind; 14] = [
    JointKind::Dado,
    JointKind::Groove,
    JointKind::Rabbet,
    JointKind::Notch,
    JointKind::Mortise,
    JointKind::MortiseAndTenon,
    JointKind::Dovetails,
    JointKind::BoxJoint,
    JointKind::HalfLap,
    JointKind::Dowels,
    JointKind::Biscuits,
    JointKind::LooseTenons,
    JointKind::Splines,
    JointKind::PocketScrews,
];

/// What the joint is for, and how the two parts have to sit for it to be offered.
fn joint_about(kind: JointKind) -> (&'static str, &'static str) {
    match kind {
        JointKind::Dado => ("Butted: a shelf's end against the side's face", "A square trench across the grain, for a shelf let into a case side."),
        JointKind::Groove => ("Butted", "A trench along the grain. A panel in a groove floats, so it is the answer to wood movement in a frame."),
        JointKind::Rabbet => ("Butted, at the end of the other part", "A step along an edge or an end, for a back or a cabinet bottom."),
        JointKind::Notch => ("Overlapping", "A corner cut out of a part so another passes through it."),
        JointKind::Mortise => ("Overlapping", "A recess sized for whatever sits in it."),
        JointKind::MortiseAndTenon => {
            ("Butted", "The frame joint: a tenon on one part into a mortise in the other. Haunched where it meets a groove, through where you want it seen.")
        }
        JointKind::Dovetails => ("Butted at a corner", "Tails and pins sloped at 1:8, locked against being pulled apart. Drawer fronts, carcase corners."),
        JointKind::BoxJoint => {
            ("Two ends meeting at a corner", "Square fingers, cut on a table saw. Nearly a dovetail's strength for a fraction of the setup.")
        }
        JointKind::HalfLap => ("Crossing, each overlapping the other", "Both parts cut to half their thickness so they finish flush where they cross."),
        JointKind::Dowels => ("Butted", "Round pegs in matching holes. Cheap to get into, and sound."),
        JointKind::Biscuits => ("Butted", "Compressed beech ovals in matching slots: alignment, and enough strength for face frames and casework."),
        JointKind::LooseTenons => (
            "Butted",
            "A separate tenon in a slot in both parts -- a mortise and tenon in a fraction of the time, given the machine. Slots go two thirds of the way into a face and the rest into the end, rounded to whole millimetres, and will take your machine's own depth stops.",
        ),
        JointKind::Splines => ("Butted", "A thin strip in a groove in both parts: mitres, edge joints, and a decorative line."),
        JointKind::PocketScrews => ("Butted", "A screw at 15 degrees through a bored pocket stepping to a pilot hole. No clamps, no drying time; hide them."),
    }
}

/// The picture that shows this joint, by the name of the recipe in `docs/guide/recipes/` that
/// builds it. `Mortise` has no recipe of its own: a mortise is the socket half of a mortise and
/// tenon, and that picture already contains one.
fn joint_picture(kind: JointKind) -> &'static str {
    match kind {
        JointKind::Dado => "dado",
        JointKind::Groove => "groove",
        JointKind::Rabbet => "rabbet",
        JointKind::Notch => "notch",
        JointKind::Mortise | JointKind::MortiseAndTenon => "mortise-and-tenon",
        JointKind::HalfLap => "half-lap",
        JointKind::Dovetails => "dovetails",
        JointKind::BoxJoint => "box-joint",
        JointKind::Dowels => "dowels",
        JointKind::Biscuits => "biscuits",
        JointKind::LooseTenons => "loose-tenons",
        JointKind::Splines => "splines",
        JointKind::PocketScrews => "pocket-screws",
    }
}

/// Two boards and one joint, rendered by the app itself, light and dark so a picture never
/// glares on a dark page. The pair of `img`s is a plain CSS swap -- see `assets/diagrams.css`.
fn joint_gallery() -> String {
    let mut out = String::from(
        "\n## What they look like\n\nEvery picture here is two boards and one joint, built from a recipe in\n`docs/guide/recipes/` by the same joinery the app uses. Change how a dovetail is cut and these\nchange with it.\n\n<div class=\"ws-gallery\">\n",
    );
    let mut shown: Vec<&str> = Vec::new();
    for kind in JOINTS {
        let picture = joint_picture(kind);
        if shown.contains(&picture) {
            continue;
        }
        shown.push(picture);
        let alt = format!("{} between two boards", kind.name());
        writeln!(
            out,
            "<figure class=\"ws-shot\">\n<img class=\"light\" src=\"../images/joints/{picture}-light.png\" alt=\"{alt}\" loading=\"lazy\">\n<img class=\"dark\" src=\"../images/joints/{picture}-dark.png\" alt=\"\" loading=\"lazy\">\n<figcaption>{}</figcaption>\n</figure>",
            kind.name()
        )
        .unwrap();
    }
    out.push_str("</div>\n");
    out
}

fn joints() -> String {
    let mut out = String::new();
    writeln!(out, "# Joints\n\n{WARNING}\n").unwrap();
    out.push_str(concat!(
        "Select exactly two parts and the **Joinery** panel appears in the tray. It offers only the\n",
        "joints that suit how those two parts actually sit, so if the one you want isn't there, move\n",
        "the parts to where that joint needs them.\n\n",
        "Making a joint cuts **both** parts in one undo step and records it, so **Change** re-opens its\n",
        "sizes and **Remove** rebuilds the parts without it. Parts that aren't square to each other get\n",
        "angled joints instead -- an angled mortise and tenon, or a housing.\n\n",
        "Why you would pick one over another is [Choosing a joint](../concepts/choosing-a-joint.md).\n\n",
        "| Joint | How the parts must sit | What it is |\n|---|---|---|\n",
    ));
    for kind in JOINTS {
        let (sit, what) = joint_about(kind);
        writeln!(out, "| **{}** | {sit} | {what} |", kind.name()).unwrap();
    }
    let (lets_in, cut_both): (Vec<_>, Vec<_>) = JOINTS.into_iter().partition(|kind: &JointKind| kind.lets_in());
    let names = |list: &[JointKind]| list.iter().map(|kind| kind.name().to_owned()).collect::<Vec<_>>().join(", ");
    writeln!(
        out,
        concat!(
            "\n## Which parts get cut\n\n",
            "Some joints are a recess in one part that the other is simply made to fit: a shelf let into a\n",
            "dado grows to fill it, so it has no cut of its own to mark out. The rest cut both parts, and\n",
            "both get the cut on their build sheet.\n\n",
            "- **One part cut, the other sized to fit:** {}.\n",
            "- **Both parts cut:** {}.\n"
        ),
        names(&lets_in),
        names(&cut_both)
    )
    .unwrap();
    out.push_str(&joint_gallery());
    out
}

// ------------------------------------------------------------------ edge profiles

fn profile_about(profile: Profile) -> &'static str {
    match profile {
        Profile::Roundover => "A quarter circle, convex. The everyday edge treatment: takes the sharpness off and wears well.",
        Profile::Chamfer => "A flat bevel straight across the corner. Crisp, and the one you can cut with a hand plane.",
        Profile::Cove => "A quarter circle, concave -- scooped into the corner. Under a tabletop edge, or as part of a moulding.",
        Profile::Ogee => "An S curve: concave near one face, convex near the other. The classic traditional moulding.",
    }
}

fn profiles() -> String {
    let mut out = String::new();
    writeln!(out, "# Edge profiles\n\n{WARNING}\n").unwrap();
    out.push_str(concat!(
        "Select one part and the **Edge profile** panel appears in the tray. Pick the edges by where\n",
        "they face -- Top, Front, and so on -- choose a profile and a size, and the part's geometry is\n",
        "rebuilt with it, in one undo step.\n\n",
        "Profiles are recorded with the drawing, so they appear as a step on the part's build sheet and\n",
        "can be taken off again. Running a profile around a face makes stacked rings with mitered\n",
        "corners; running it along an edge extrudes the shape the length of the part.\n\n",
        "| Profile | What it looks like |\n|---|---|\n",
    ));
    for profile in Profile::ALL {
        writeln!(out, "| **{}** | {} |", profile.name(), profile_about(profile)).unwrap();
    }
    out
}

// ------------------------------------------------------------------ templates

fn template_about(template: &Template) -> &'static str {
    match template {
        Template::Cabinet(_) => {
            "A carcase: sides, top, bottom, back and shelves, with the joints you pick. Each opening between shelves can be left open, or filled with a drawer (a box on slides, a front and a pull) or doors (a pair over 24 inches wide, slab or frame-and-panel, on concealed hinges), set overlay or inset."
        }
        Template::Door(_) => {
            "A door on its own, to a size you give, for hanging on something that isn't a WoodSketch cabinet. Slab, or frame-and-panel with stub tenons and a floating panel."
        }
        Template::Drawer(_) => "A drawer box to a size you give, on the longest side-mount slides that fit, with the front and the pull.",
        Template::FaceFrame(_) => "Stiles and rails to the opening you give, with middle rails where you want them.",
        Template::Table(_) => "A top, four legs and aprons, joined to the legs.",
    }
}

fn templates() -> String {
    let mut out = String::new();
    writeln!(out, "# Templates\n\n{WARNING}\n").unwrap();
    out.push_str(concat!(
        "The **Templates** panel in the tray builds a whole piece from a few sizes and a joint choice,\n",
        "rather than board by board. A new one is **carried to where you click** -- it is built at the\n",
        "origin and handed to the Move tool, so `Esc` undoes the whole thing and the status bar tells\n",
        "you to click to put it down.\n\n",
        "The sizes stay on the piece. Select it and the **Sizes** panel re-opens them, so changing a\n",
        "cabinet's height rebuilds it at that height instead of stretching it. Edit one by hand and it\n",
        "will refuse to rebuild until you tell it twice, because it can no longer be sure what it would\n",
        "be throwing away.\n\n",
        "| Template | What it builds |\n|---|---|\n",
    ));
    for template in Template::all() {
        writeln!(out, "| **{}** | {} |", template.name(), template_about(&template)).unwrap();
    }
    let list = |names: Vec<&'static str>| names.iter().map(|name| format!("**{name}**")).collect::<Vec<_>>().join(", ");
    writeln!(
        out,
        concat!(
            "\n## The joints they offer\n\n",
            "- **Carcases** (a cabinet's case): {}.\n",
            "- **Drawers** (the box's corners): {}.\n",
            "- **Frames** (a door, a face frame, a table's aprons): {}.\n\n",
            "What each of those is, and when to choose it, is in [Joints](joints.md) and\n",
            "[Choosing a joint](../concepts/choosing-a-joint.md).\n"
        ),
        list(CaseJoints::ALL.into_iter().map(CaseJoints::name).collect()),
        list(DrawerJoints::ALL.into_iter().map(DrawerJoints::name).collect()),
        list(FrameJoints::ALL.into_iter().map(FrameJoints::name).collect())
    )
    .unwrap();
    out
}

// ------------------------------------------------------------------ materials and stock

fn kind_about(kind: MaterialKind) -> &'static str {
    match kind {
        MaterialKind::SolidWood => {
            "Boards milled from rough stock. Has grain, moves with the seasons, listed with its thickness, packed two-dimensionally onto boards, and glued up from narrower ones when a part is wider than anything you can buy."
        }
        MaterialKind::SheetGood => "Plywood, MDF and the like: a fixed thickness, bought as sheets, packed as guillotine cuts. Doesn't move.",
        MaterialKind::DimensionalLumber => "Ready-milled stock like a 2x4, bought by section and length, packed along standard lengths.",
        MaterialKind::Hardware => "Hinges, screws, knobs, slides: counted and bought, never cut, and left out of the diagrams and the movement check.",
        MaterialKind::Other => {
            "Glass, steel, concrete. Listed in the parts so nothing is forgotten, named in one line beside the diagrams, and not packed onto stock."
        }
    }
}

/// The stock a material is sold in, said the way the cut list thinks about it.
fn stock_of(preset: &Preset) -> String {
    let standard = preset.standard();
    let lengths = |values: &[f64]| values.iter().map(|&v| format_length(v)).collect::<Vec<_>>().join(", ");
    match preset.kind {
        MaterialKind::SheetGood => {
            let sheets = standard.sheets.iter().map(|&(l, w)| format!("{} x {}", format_length(l), format_length(w))).collect::<Vec<_>>().join(", ");
            format!("{} thick; sheets {sheets}", lengths(&standard.thicknesses))
        }
        MaterialKind::DimensionalLumber => {
            let nominal = lumber_sizes();
            let named = |t: f64, w: f64| {
                nominal
                    .iter()
                    .find(|(_, nt, nw)| (nt - t).abs() < 1e-9 && (nw - w).abs() < 1e-9)
                    .map_or_else(|| format!("{} x {}", format_length(t), format_length(w)), |(name, _, _)| name.clone())
            };
            let sections = standard.sections.iter().map(|&(t, w)| named(t, w)).collect::<Vec<_>>().join(", ");
            format!("{sections}; lengths {}", lengths(&standard.lengths))
        }
        MaterialKind::SolidWood => {
            let widest = standard.widths.iter().copied().fold(0.0, f64::max);
            format!("{} thick; boards up to {} wide, lengths {}", lengths(&standard.thicknesses), format_length(widest), lengths(&standard.lengths))
        }
        MaterialKind::Hardware | MaterialKind::Other => "Bought, not cut".to_owned(),
    }
}

fn materials() -> String {
    let mut out = String::new();
    writeln!(out, "# Materials and stock\n\n{WARNING}\n").unwrap();
    out.push_str(concat!(
        "A new drawing starts with no materials. You add them from the library -- the **Materials**\n",
        "panel, or straight from the Board tool's dropdown, which adds whatever you pick.\n\n",
        "A material is a **kind of stock, not one size**. Plywood comes in several thicknesses; Pine\n",
        "Lumber comes in every standard section; the cut list works out which one each part needs from\n",
        "the part's own size. The sizes below are the US defaults, and they are yours to change: for\n",
        "the whole drawing in **Cut List > Stock & kerf**, or per material in the Materials panel.\n\n",
        "Solid wood and lumber also carry how much the species moves -- see\n",
        "[Wood movement by species](wood.md). A material you make yourself can borrow a species'\n",
        "figures with *Moves like*.\n\n",
        "| Material | Type | Grain | Sold as |\n|---|---|---|---|\n",
    ));
    for preset in LIBRARY {
        let grain = if preset.grained { "yes" } else { "no" };
        writeln!(out, "| **{}** | {} | {grain} | {} |", preset.name, preset.kind.name(), stock_of(preset)).unwrap();
    }
    out.push_str("\n## The types, and what each one changes\n\n| Type | How it is bought, cut and listed |\n|---|---|\n");
    for kind in MaterialKind::ALL {
        writeln!(out, "| **{}** | {} |", kind.name(), kind_about(kind)).unwrap();
    }
    out
}

// ------------------------------------------------------------------ wood movement

fn wood() -> String {
    // Two feet wide: the same board the concept page works through, and wide enough that the two
    // columns don't both round to the same sixteenth of an inch.
    const EXAMPLE: f64 = 24.0;
    let species = movement::species_list();
    // How much of its flat-sawn movement a quartersawn board of the same species keeps. Said as
    // the range these species actually cover, rather than as a rule of thumb: "less than half"
    // was written here first and birch (76%) and walnut (69%) are nothing of the kind.
    let mut ratios: Vec<f64> = species.iter().map(|(_, moves)| moves.radial / moves.tangential).collect();
    ratios.sort_by(f64::total_cmp);
    let percent = |fraction: f64| fraction * 100.0;
    let (least, most, middle) = (percent(ratios[0]), percent(ratios[ratios.len() - 1]), percent(ratios[ratios.len() / 2]));

    let mut out = String::new();
    writeln!(out, "# Wood movement by species\n\n{WARNING}\n").unwrap();
    writeln!(
        out,
        concat!(
            "How much each species swells and shrinks **across** the grain, from the USDA Wood Handbook.\n",
            "Along the grain the movement is small enough to design as though it were zero.\n\n",
            "The two figures are the fraction of its width a board gains or loses per 1% change in moisture\n",
            "content: the **flat-sawn** figure where the growth rings run roughly parallel to the face, the\n",
            "**quartersawn** figure where they run across it. Quartersawn always moves less -- across the\n",
            "species below it runs from {:.0}% of the flat-sawn figure to {:.0}%, about {:.0}% for the middle\n",
            "species -- and that stability is what you are paying for.\n\n",
            "> movement = width x coefficient x moisture swing\n\n",
            "The last two columns work that out for a {} board over a {}% swing in moisture content, which\n",
            "is the default for a drawing and about what a heated house does across a year. Change it in\n",
            "**Cut List > Stock & kerf**.\n\n",
            "What to do about all this is [Grain, and why wood moves](../concepts/grain-and-movement.md).\n\n",
            "| Species | Flat-sawn | Quartersawn | A {} board moves, flat-sawn | ...and quartersawn |\n|---|---|---|---|---|"
        ),
        least,
        most,
        middle,
        format_length(EXAMPLE),
        DEFAULT_SWING,
        format_length(EXAMPLE)
    )
    .unwrap();
    for (name, moves) in species {
        let across = |coefficient: f64| format_length(EXAMPLE * coefficient * DEFAULT_SWING);
        writeln!(out, "| **{name}** | {:.5} | {:.5} | {} | {} |", moves.tangential, moves.radial, across(moves.tangential), across(moves.radial)).unwrap();
    }
    out.push_str(concat!(
        "\n## Where these come from, and where they go\n\n",
        "A material made from the library carries its species' figures. One you make yourself can\n",
        "borrow them in the Materials panel with *Moves like*; without them, it isn't checked.\n\n",
        "Each part also carries which way its grain runs and how it was sawn (**Entity Info**), and the\n",
        "sawing is what picks between the two columns above. A part whose sawing you haven't said is\n",
        "assumed flat-sawn, which is the worse case.\n\n",
        "**Cut List > Wood movement** then lists every solid-wood part's movement and every place two\n",
        "touching parts disagree about it. **View > Wood Movement** draws the same thing as arrows in\n",
        "the viewport. A panel floating in a groove, and a part you have marked *free to move*, are\n",
        "understood to be allowed for and aren't reported.\n",
    ));
    out
}

// ------------------------------------------------------------------ the tests

#[test]
fn the_reference_pages_say_what_the_app_does() {
    let mut rewritten = Vec::new();
    for (name, built) in pages() {
        let path = guide().join(name);
        std::fs::create_dir_all(path.parent().expect("a folder")).expect("the folder to write into");
        let on_disk = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
        if on_disk != built {
            std::fs::write(&path, &built).expect("writing the page");
            rewritten.push(name);
        }
    }
    assert!(
        rewritten.is_empty(),
        "the guide's reference pages had fallen behind the app: {}.\nThey have been written again -- look at the diff and commit it.",
        rewritten.join(", ")
    );
}

/// `ToolId::ALL` is what the toolbar, the shortcuts and this guide are all built from, and
/// nothing makes the compiler check that it holds every tool. A variant added without it would
/// be a tool with no page, no toolbar button and no key.
#[test]
fn every_tool_is_in_the_list_the_guide_is_built_from() {
    let listed = variants("crates/tools/src/lib.rs", "ToolId");
    assert_eq!(listed.len(), ToolId::ALL.len(), "ToolId has {} variants but ToolId::ALL holds {}: {listed:?}", listed.len(), ToolId::ALL.len());
}

/// The guide shows each tool's own toolbar icon, read from `ui_egui::icons` and drawn from
/// Phosphor's SVGs vendored under `src/icons/`. A tool whose glyph changes to one nobody has
/// vendored would otherwise leave an empty square on the page, which nothing else would notice.
#[test]
fn every_tool_has_its_toolbar_icon_vendored() {
    let icons = icon_names();
    let dir = guide().join("src/icons");
    for tool in ToolId::ALL {
        let icon = icon_of(tool, &icons);
        for (weight, file) in [("regular", dir.join(format!("regular/{icon}.svg"))), ("fill", dir.join(format!("fill/{icon}.svg")))] {
            assert!(
                file.exists(),
                "the {} tool draws Phosphor's `{icon}`, but its {weight} SVG isn't in docs/guide/src/icons/.\nFetch it from https://github.com/phosphor-icons/core (assets/{weight}/{icon}{}.svg).",
                tool.name(),
                if weight == "fill" { "-fill" } else { "" }
            );
        }
    }
}

/// Nothing vendored that nothing uses: 26 tools' worth of artwork is small, but an icon left
/// behind by a tool that changed its glyph is dead weight nobody would think to look for.
#[test]
fn no_icon_is_vendored_that_no_tool_uses() {
    let icons = icon_names();
    let wanted: Vec<String> = ToolId::ALL.into_iter().map(|tool| icon_of(tool, &icons)).collect();
    for weight in ["regular", "fill"] {
        let dir = guide().join("src/icons").join(weight);
        for entry in std::fs::read_dir(&dir).expect("a folder of icons").flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|extension| extension == "svg") {
                let stem = path.file_stem().expect("a name").to_string_lossy().into_owned();
                assert!(wanted.contains(&stem), "docs/guide/src/icons/{weight}/{stem}.svg belongs to no tool any more; delete it");
            }
        }
    }
}

#[test]
fn every_tool_is_named_in_the_guide() {
    let text: String = pages().into_iter().map(|(_, page)| page).collect();
    for tool in ToolId::ALL {
        assert!(text.contains(&tool.name()), "the guide never names the {} tool", tool.name());
    }
}

/// Every joint the app can record is in the list the guide is built from. The `match` in
/// `joint_about` already forces a line to be written for a new one; this catches the other half,
/// a variant that has a line but never reaches the page.
#[test]
fn every_joint_is_in_the_list_the_guide_is_built_from() {
    let listed = variants("crates/doc/src/joinery.rs", "JointKind");
    assert_eq!(listed.len(), JOINTS.len(), "JointKind has {} variants but the guide lists {}: {listed:?}", listed.len(), JOINTS.len());
}

/// The hand-drawn diagrams are pulled into a page by mdBook's `{{#include}}`, which inlines
/// them so their `currentColor` strokes follow the theme the reader chose. An include whose
/// target has moved leaves the literal `{{#include ...}}` text sitting in the page -- mdBook
/// says nothing about it -- so both directions are checked here.
#[test]
fn every_diagram_is_drawn_into_a_page_and_every_include_finds_one() {
    let src = guide().join("src");
    let mut included: Vec<PathBuf> = Vec::new();
    for page in markdown_files(&src) {
        let text = std::fs::read_to_string(&page).expect("a page");
        let here = page.parent().expect("a folder");
        for rest in text.split("{{#include ").skip(1) {
            let target = rest.split("}}").next().expect("the end of an include").trim();
            let path = here.join(target);
            assert!(path.exists(), "{} includes {target}, which isn't there", page.display());
            included.push(path.canonicalize().expect("a real path"));
        }
    }
    let drawings = src.join("diagrams");
    for entry in std::fs::read_dir(&drawings).expect("the diagrams folder").flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "svg") {
            let real = path.canonicalize().expect("a real path");
            assert!(included.contains(&real), "{} is drawn but no page includes it", path.display());
        }
    }
    assert!(!included.is_empty(), "no page includes a diagram any more");
}

/// Every joint the app can make has a picture, in both themes. The pictures are rendered by
/// `scripts/make-guide-pictures.py`, which needs a GPU and so cannot run in CI; this is what
/// notices that a new joint arrived without one, or that a rendering was never committed.
#[test]
fn every_joint_has_a_picture_in_both_themes() {
    let dir = guide().join("src/images/joints");
    for kind in JOINTS {
        let picture = joint_picture(kind);
        assert!(
            guide().join(format!("recipes/{picture}.json")).exists(),
            "{} is pictured by `{picture}`, but docs/guide/recipes/{picture}.json isn't there",
            kind.name()
        );
        for theme in ["light", "dark"] {
            let file = dir.join(format!("{picture}-{theme}.png"));
            assert!(
                file.exists(),
                "no {theme} picture of the {} joint.
Run `python -I scripts/make-guide-pictures.py`.",
                kind.name()
            );
        }
    }
}

#[test]
fn every_page_is_in_the_contents() {
    let src = guide().join("src");
    let summary = std::fs::read_to_string(src.join("SUMMARY.md")).expect("SUMMARY.md");
    for page in markdown_files(&src) {
        let relative = page.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
        if relative == "SUMMARY.md" {
            continue;
        }
        assert!(summary.contains(&relative), "{relative} is in the guide but not in SUMMARY.md, so nothing links to it");
    }
}

#[test]
fn every_link_in_the_guide_goes_somewhere() {
    let src = guide().join("src");
    for page in markdown_files(&src) {
        let text = std::fs::read_to_string(&page).expect("a page");
        let here = page.parent().expect("a folder");
        for link in links(&text) {
            let target = link.split('#').next().unwrap_or(&link);
            if target.is_empty() || target.starts_with("http") || target.starts_with("mailto:") {
                continue;
            }
            assert!(here.join(target).exists(), "{} links to {target}, which isn't there", page.display());
        }
    }
}

/// The worked example in the movement page is arithmetic, and arithmetic in prose goes stale
/// silently. This is the same sum the app does.
#[test]
fn the_movement_example_is_what_the_numbers_say() {
    let page = std::fs::read_to_string(guide().join("src/concepts/grain-and-movement.md")).expect("the movement page");
    let oak = movement::species("Red Oak").expect("red oak's figures");
    for (coefficient, side) in [(oak.tangential, "flat-sawn"), (oak.radial, "quartersawn")] {
        let moves = format!("{:.3}", 24.0 * coefficient * DEFAULT_SWING);
        assert!(page.contains(&moves), "the page's {side} example no longer matches the app, which works it out as {moves} inches");
        assert!(page.contains(&format!("{coefficient:.5}")), "red oak's {side} coefficient on the page isn't the app's any more");
    }
}

// ------------------------------------------- reading the guide, and the source it is checked against

/// The variant names of an enum, read out of the source the way the i18n guard reads `t!` keys:
/// there is no way to ask Rust for them, and a list kept by hand is the thing being checked.
fn variants(file: &str, name: &str) -> Vec<String> {
    let text = std::fs::read_to_string(root().join(file)).expect("the source file");
    let at = text.find(&format!("pub enum {name} {{")).expect("the enum");
    let body = &text[at..];
    let end = body.find("\n}").expect("the end of the enum");
    body[..end]
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with('#'))
        .filter_map(|line| line.split(['(', '{', ',', ' ']).next())
        .filter(|word| word.starts_with(char::is_uppercase))
        .map(str::to_owned)
        .collect()
}

fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut folders = vec![dir.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(&folder).expect("a folder of the guide").flatten() {
            let path = entry.path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().is_some_and(|extension| extension == "md") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// The targets of every `[text](target)` link in a page.
fn links(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        rest = &rest[at + 2..];
        if let Some(end) = rest.find(')') {
            found.push(rest[..end].to_owned());
            rest = &rest[end..];
        }
    }
    found
}
