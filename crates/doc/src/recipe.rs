//! Project recipes: a piece of furniture described as JSON (materials, boards placed as boxes,
//! templates, joints between named parts, tags and groups) that builds a real drawing. It's how
//! AI assistants and scripts design with WoodSketch: `woodsketch-cli build recipe.json`, or the
//! MCP server's `build` tool. Mistakes come back as clear messages naming the item, so an agent
//! can fix them ("joints[2] (mortise_and_tenon, Apron into Leg): Apron doesn't touch Leg").
//!
//! Lengths are numbers in the recipe's units (inches unless `"units": "mm"`) or strings like
//! `"3/4\""`, `"19mm"`, `"1 1/2"`. A part is a box: `at` is its lowest corner (smallest x, y, z),
//! its length runs along `length` (an axis: `x`, `y` or `z`), its width along `width`, and its
//! thickness along the third axis. Z is up; x is left to right, y front to back.

use std::collections::HashMap;

use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::joinery::{Dovetails, JoineryError, JointKind, Slots, Spline};
use crate::template::Template;
use crate::units::{Units, format_length, parse_length, set_units, units};
use crate::{DefinitionKind, Document, Entity, InstanceId, MaterialId, MaterialKind};

/// A project to build.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    /// What it is, for the drawing's title.
    #[serde(default)]
    pub name: String,
    /// `in` (the default) or `mm`: what a bare number means.
    #[serde(default)]
    pub units: Option<String>,
    #[serde(default)]
    pub materials: Vec<MaterialSpec>,
    #[serde(default)]
    pub parts: Vec<PartSpec>,
    #[serde(default)]
    pub templates: Vec<TemplateSpec>,
    #[serde(default)]
    pub joints: Vec<JointSpec>,
    #[serde(default)]
    pub groups: Vec<GroupSpec>,
}

/// A material, by an `id` the parts use: one from the library (`"library": "Cherry"`), or your
/// own (`name`, `kind`, `color`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialSpec {
    pub id: String,
    #[serde(default)]
    pub library: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    /// `solid wood`, `lumber`, `sheet`, `hardware` or `other`.
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
}

/// A length: a number in the recipe's units, or text with its own (`"3/4\""`, `"19mm"`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Length {
    Number(f64),
    Text(String),
}

/// A board, placed as a box.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartSpec {
    /// Its name (unique in the recipe: joints and groups use it).
    pub name: String,
    /// A material's `id`.
    #[serde(default)]
    pub material: Option<String>,
    /// `[length, width, thickness]`.
    pub size: [Length; 3],
    /// Its lowest corner (smallest x, y and z).
    #[serde(default = "origin")]
    pub at: [Length; 3],
    /// The axis its length (and grain) runs along: `x`, `y` or `z`. Default `x`.
    #[serde(default = "x_axis")]
    pub length: String,
    /// The axis its width runs along. Default `y` (lying flat, if the length runs along x).
    #[serde(default = "y_axis")]
    pub width: String,
    /// A tag (made if it's new).
    #[serde(default)]
    pub tag: Option<String>,
    /// Fastened so it can move across its grain (tabletop buttons, slotted screws).
    #[serde(default)]
    pub free_to_move: bool,
}

fn origin() -> [Length; 3] {
    [Length::Number(0.0), Length::Number(0.0), Length::Number(0.0)]
}
fn x_axis() -> String {
    "x".into()
}
fn y_axis() -> String {
    "y".into()
}

/// A template: `cabinet`, `drawer`, `face_frame` or `table`, with only the sizes that differ from
/// its defaults (`{"width": 24, "height": 30}`).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateSpec {
    pub kind: String,
    /// Its group's name.
    #[serde(default)]
    pub name: Option<String>,
    /// Its lowest corner.
    #[serde(default = "origin")]
    pub at: [Length; 3],
    /// A material's `id` for its wood.
    #[serde(default)]
    pub wood: Option<String>,
    #[serde(default)]
    pub sizes: serde_json::Map<String, serde_json::Value>,
}

/// A joint between two parts that touch or overlap.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointSpec {
    /// `let_in` (a dado, groove or rabbet: `part` is let into `into`), `cut_overlap`,
    /// `mortise_and_tenon` (`part` is the rail with the tenon), `dowels`, `biscuits`,
    /// `loose_tenons`, `splines`, `pocket_screws`, `dovetails` (`part` gets the tails),
    /// `box_joint` or `half_lap`.
    pub kind: String,
    pub part: String,
    pub into: String,
    /// Sizes; anything left out is suggested from the parts: `depth` (let_in), `length`,
    /// `thickness`, `shoulder` (tenon), `count`, `diameter` (dowels, pocket screws, biscuits),
    /// `tails`, `half_blind` (dovetails), `finger` (box joint), `screw` (pocket screw length).
    #[serde(default)]
    pub options: serde_json::Map<String, serde_json::Value>,
}

/// A group (or component) of named parts and templates, made last.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupSpec {
    pub name: String,
    pub parts: Vec<String>,
    #[serde(default)]
    pub component: bool,
}

/// Read a recipe from JSON, with a message saying where it's wrong.
pub fn parse(json: &str) -> Result<Recipe, String> {
    serde_json::from_str(json).map_err(|e| format!("The recipe isn't valid JSON for WoodSketch: {e}"))
}

/// Build the drawing. Every problem found is listed, each naming its item.
pub fn build(recipe: &Recipe) -> Result<Document, Vec<String>> {
    let before = units();
    let mm = match recipe.units.as_deref().map(str::to_ascii_lowercase).as_deref() {
        None | Some("in" | "inch" | "inches") => false,
        Some("mm" | "millimeters" | "millimetres") => true,
        Some(other) => return Err(vec![format!("units: \"{other}\" isn't one of \"in\" or \"mm\".")]),
    };
    set_units(if mm { Units::Millimeters } else { Units::Inches });
    let result = build_in(recipe, mm);
    set_units(before);
    result
}

fn build_in(recipe: &Recipe, mm: bool) -> Result<Document, Vec<String>> {
    let mut doc = Document::default();
    let mut errors = Vec::new();
    let length = |l: &Length, what: &str, errors: &mut Vec<String>| -> f64 {
        match l {
            Length::Number(n) => n / if mm { 25.4 } else { 1.0 },
            Length::Text(t) => parse_length(t).unwrap_or_else(|e| {
                errors.push(format!("{what}: {e}"));
                0.0
            }),
        }
    };

    // Materials.
    let mut materials: HashMap<String, MaterialId> = HashMap::new();
    for (i, m) in recipe.materials.iter().enumerate() {
        let what = format!("materials[{i}] ({})", m.id);
        let id = if let Some(name) = &m.library {
            match crate::library::LIBRARY.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                Some(preset) => doc.add_preset(preset),
                None => {
                    let known: Vec<&str> = crate::library::LIBRARY.iter().map(|p| p.name).collect();
                    errors.push(format!("{what}: \"{name}\" isn't in the library ({}).", known.join(", ")));
                    continue;
                }
            }
        } else {
            let kind = match m.kind.as_deref().map(str::to_ascii_lowercase).as_deref() {
                None | Some("solid wood" | "solid" | "wood") => MaterialKind::SolidWood,
                Some("lumber" | "dimensional lumber") => MaterialKind::DimensionalLumber,
                Some("sheet" | "sheet good" | "sheet goods" | "plywood") => MaterialKind::SheetGood,
                Some("hardware") => MaterialKind::Hardware,
                Some("other") => MaterialKind::Other,
                Some(other) => {
                    errors.push(format!("{what}: kind \"{other}\" isn't one of solid wood, lumber, sheet, hardware or other."));
                    continue;
                }
            };
            let id = doc.add_material(kind);
            let (name, color) = (m.name.clone().unwrap_or_else(|| m.id.clone()), m.color);
            doc.edit_material(id, |mat| {
                mat.name = name.clone();
                mat.movement = crate::movement::species(&name);
                if let Some(c) = color {
                    mat.color = c;
                }
            });
            id
        };
        if materials.insert(m.id.clone(), id).is_some() {
            errors.push(format!("{what}: another material has the id \"{}\".", m.id));
        }
    }
    let material = |id: &Option<String>, what: &str, errors: &mut Vec<String>| -> Option<MaterialId> {
        let id = id.as_ref()?;
        let found = materials.get(id).copied();
        if found.is_none() {
            let known: Vec<&str> = recipe.materials.iter().map(|m| m.id.as_str()).collect();
            errors.push(format!("{what}: no material \"{id}\" (the materials are: {}).", if known.is_empty() { "none yet".to_owned() } else { known.join(", ") }));
        }
        found
    };

    // Parts: boxes from their lowest corner.
    let mut named: HashMap<String, InstanceId> = HashMap::new();
    for (i, p) in recipe.parts.iter().enumerate() {
        let what = format!("parts[{i}] ({})", p.name);
        let [l, w, t] = [0, 1, 2].map(|k| length(&p.size[k], &what, &mut errors));
        let at = DVec3::from_array([0, 1, 2].map(|k| length(&p.at[k], &what, &mut errors)));
        let (Some(la), Some(wa)) = (axis(&p.length), axis(&p.width)) else {
            errors.push(format!("{what}: length and width are axes: \"x\", \"y\" or \"z\" (got \"{}\" and \"{}\").", p.length, p.width));
            continue;
        };
        if la == wa {
            errors.push(format!("{what}: its length and width run along the same axis ({}); they need two different ones.", p.length));
            continue;
        }
        if l <= 0.0 || w <= 0.0 || t <= 0.0 {
            errors.push(format!("{what}: its size needs three lengths above zero."));
            continue;
        }
        let mat = material(&p.material, &what, &mut errors);
        // The board's thickness runs along length x width; if that points down an axis, start
        // from the far side so the box still begins at `at`.
        let (ld, wd) = (DVec3::AXES[la], DVec3::AXES[wa]);
        let td = ld.cross(wd);
        let origin = if td.min_element() < 0.0 { at - td * t } else { at };
        let Some(id) = doc.add_board(&Board { origin, length_dir: ld, width_dir: wd, length: l, width: w, thickness: t }, mat) else {
            errors.push(format!("{what}: couldn't make the board."));
            continue;
        };
        doc.rename_instance(id, &p.name);
        if p.free_to_move {
            doc.edit_part(id, |info| info.free_to_move = true);
        }
        if named.insert(p.name.clone(), id).is_some() {
            errors.push(format!("{what}: another part or template is named \"{}\"; names must be unique.", p.name));
        }
    }

    // Templates: built where they're put.
    for (i, t) in recipe.templates.iter().enumerate() {
        let name = t.name.clone().unwrap_or_else(|| t.kind.clone());
        let what = format!("templates[{i}] ({name})");
        let template = match template(&t.kind, &t.sizes, mm) {
            Ok(template) => template,
            Err(e) => {
                errors.push(format!("{what}: {e}"));
                continue;
            }
        };
        let wood = material(&t.wood, &what, &mut errors);
        let at = DVec3::from_array([0, 1, 2].map(|k| length(&t.at[k], &what, &mut errors)));
        match doc.add_template(&template, wood) {
            Ok(id) => {
                // Move it so its box starts at `at`.
                let corners = doc.scene().root.instance(id).map(|inst| doc.scene().instance_box_corners(inst)).unwrap_or_default();
                let lo = corners.iter().copied().reduce(DVec3::min).unwrap_or(DVec3::ZERO);
                doc.edit_scene(|scene, _| {
                    if let Some(inst) = scene.root.instance_mut(id) {
                        inst.transform.translation += at - lo;
                    }
                });
                doc.rename_instance(id, &name);
                if named.insert(name.clone(), id).is_some() {
                    errors.push(format!("{what}: another part or template is named \"{name}\"; names must be unique."));
                }
            }
            Err(e) => errors.push(format!("{what}: {e}")),
        }
    }

    // Joints, in order.
    for (i, j) in recipe.joints.iter().enumerate() {
        let what = format!("joints[{i}] ({}, {} into {})", j.kind, j.part, j.into);
        let (Some(&part), Some(&into)) = (named.get(&j.part), named.get(&j.into)) else {
            let missing: Vec<&str> = [&j.part, &j.into].into_iter().filter(|n| !named.contains_key(*n)).map(String::as_str).collect();
            errors.push(format!("{what}: no part named {}.", missing.iter().map(|n| format!("\"{n}\"")).collect::<Vec<_>>().join(" or ")));
            continue;
        };
        if let Err(e) = joint(&mut doc, j, part, into, mm) {
            let said = match e {
                JoineryError::NotTouching | JoineryError::NoOverlap => format!("{} doesn't touch {} (move one so they meet, or overlap by the joint's depth).", j.part, j.into),
                other => other.to_string(),
            };
            errors.push(format!("{what}: {said}"));
        }
    }

    // Tags.
    for (i, p) in recipe.parts.iter().enumerate() {
        let (Some(tag), Some(&id)) = (&p.tag, named.get(&p.name)) else { continue };
        let existing = doc.scene().tags().find(|(_, t)| t.name == *tag).map(|(id, _)| id);
        let tag_id = existing.unwrap_or_else(|| doc.edit_scene(|scene, _| scene.add_tag(tag.clone())));
        doc.selection_mut().clear();
        doc.selection_mut().extend([Entity::Instance(id)]);
        if doc.tag_selection(Some(tag_id)) == 0 {
            errors.push(format!("parts[{i}] ({}): couldn't tag it \"{tag}\".", p.name));
        }
    }

    // Groups, last (joints are made between parts side by side).
    for (i, g) in recipe.groups.iter().enumerate() {
        let what = format!("groups[{i}] ({})", g.name);
        let mut members = Vec::new();
        for name in &g.parts {
            match named.get(name) {
                Some(&id) => members.push(Entity::Instance(id)),
                None => errors.push(format!("{what}: no part or template named \"{name}\" (or it's already in another group).")),
            }
        }
        if members.is_empty() {
            continue;
        }
        doc.selection_mut().clear();
        doc.selection_mut().extend(members);
        match doc.group_selection(if g.component { DefinitionKind::Component } else { DefinitionKind::Group }) {
            Some(id) => {
                doc.rename_instance(id, &g.name);
                for name in &g.parts {
                    named.remove(name);
                }
                named.insert(g.name.clone(), id);
            }
            None => errors.push(format!("{what}: couldn't group them.")),
        }
    }
    doc.selection_mut().clear();
    if errors.is_empty() { Ok(doc) } else { Err(errors) }
}

/// `x`, `y`, `z` (or `-x`...: the sign doesn't matter for a box).
fn axis(name: &str) -> Option<usize> {
    match name.trim().trim_start_matches(['-', '+']).to_ascii_lowercase().as_str() {
        "x" | "red" => Some(0),
        "y" | "green" => Some(1),
        "z" | "blue" | "up" => Some(2),
        _ => None,
    }
}

/// A template from its kind and the sizes that differ from its defaults (lengths in the
/// recipe's units).
fn template(kind: &str, sizes: &serde_json::Map<String, serde_json::Value>, mm: bool) -> Result<Template, String> {
    let defaults = match kind.to_ascii_lowercase().replace([' ', '-'], "_").as_str() {
        "cabinet" => Template::Cabinet(Default::default()),
        "drawer" => Template::Drawer(Default::default()),
        "face_frame" | "faceframe" => Template::FaceFrame(Default::default()),
        "table" => Template::Table(Default::default()),
        other => return Err(format!("\"{other}\" isn't a template (cabinet, drawer, face_frame or table).")),
    };
    // Its fields as JSON, with the given ones laid over.
    let mut json = serde_json::to_value(defaults).map_err(|e| e.to_string())?;
    let (variant, fields) = json.as_object_mut().and_then(|o| o.iter_mut().next()).ok_or("no template fields")?;
    let variant = variant.clone();
    let fields = fields.as_object_mut().ok_or("no template fields")?;
    for (key, value) in sizes {
        if !fields.contains_key(key) {
            let known: Vec<&String> = fields.keys().collect();
            return Err(format!("{variant} has no size \"{key}\" (it has: {}).", known.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")));
        }
        // Lengths given in millimeters become inches; a short list (a cabinet's first few
        // openings) keeps the defaults for the rest.
        let value = match (value, &fields[key]) {
            (serde_json::Value::Number(n), default) if mm && default.is_f64() => serde_json::json!(n.as_f64().unwrap_or(0.0) / 25.4),
            (serde_json::Value::Array(given), serde_json::Value::Array(default)) if given.len() < default.len() => {
                serde_json::Value::Array(given.iter().chain(&default[given.len()..]).cloned().collect())
            }
            (other, _) => other.clone(),
        };
        fields.insert(key.clone(), value);
    }
    serde_json::from_value(json).map_err(|e| format!("{variant}: {e}"))
}

/// Make one joint, with sizes given or suggested.
fn joint(doc: &mut Document, j: &JointSpec, part: InstanceId, into: InstanceId, mm: bool) -> Result<(), JoineryError> {
    let fit = doc.joint_fit(part, into)?;
    let number = |key: &str| j.options.get(key).and_then(serde_json::Value::as_f64);
    let len = |key: &str, default: f64| number(key).map_or(default, |v| v / if mm { 25.4 } else { 1.0 });
    let count = |default: u32| number("count").map_or(default, |v| v.max(1.0) as u32);
    let (thick, wide) = fit.contact;
    match j.kind.to_ascii_lowercase().replace([' ', '-'], "_").as_str() {
        "let_in" | "dado" | "groove" | "rabbet" => doc.let_in(part, into, len("depth", fit.let_in_depth)).map(|_| ()),
        "cut_overlap" => doc.cut_overlap(part, into).map(|_| ()),
        "mortise_and_tenon" | "tenon" => {
            let mut tenon = fit.tenon;
            tenon.length = len("length", tenon.length);
            tenon.thickness = len("thickness", tenon.thickness);
            tenon.shoulder = len("shoulder", tenon.shoulder);
            doc.mortise_and_tenon(part, into, tenon)
        }
        "dowels" => {
            let mut dowels = fit.dowels;
            dowels.count = count(dowels.count);
            dowels.diameter = len("diameter", dowels.diameter);
            dowels.depth = len("depth", dowels.depth);
            doc.dowels(part, into, dowels)
        }
        "biscuits" => {
            let mut slots = Slots::biscuits(Slots::suggested_biscuit(thick, wide), wide);
            slots.count = count(slots.count);
            doc.slots(part, into, &slots)
        }
        "loose_tenons" => {
            let mut slots = Slots::loose_tenons(Slots::suggested_loose_tenon(thick), wide);
            slots.count = count(slots.count);
            doc.slots(part, into, &slots)
        }
        "splines" | "spline" => doc.slots(part, into, &Spline::for_stock(thick)),
        "pocket_screws" | "pocket_holes" => doc.pocket_screws(part, into, count(2), len("screw", 1.25)),
        "dovetails" | "dovetail" => {
            let tails = number("tails").map_or(Dovetails::suggested(wide), |v| v.max(1.0) as u32);
            let half_blind = j.options.get("half_blind").and_then(serde_json::Value::as_bool).unwrap_or(false);
            doc.dovetails(part, into, Dovetails { tails, half_blind })
        }
        "box_joint" | "finger_joint" => doc.box_joint(part, into, len("finger", fit.finger)),
        "half_lap" => doc.half_lap(part, into),
        other => Err(JoineryError::Geometry(format!(
            "\"{other}\" isn't a joint WoodSketch makes (let_in, cut_overlap, mortise_and_tenon, dowels, biscuits, loose_tenons, splines, pocket_screws, dovetails, box_joint, half_lap)."
        ))),
    }
}

/// What a drawing has, in a few lines: its size, parts by material, joints, what to buy and
/// any wood movement warnings. For agents to check their work.
pub fn summary(doc: &Document) -> String {
    let scene = doc.scene();
    let mut out = String::new();
    if let Some((lo, hi)) = scene.bounds() {
        let size = hi - lo;
        out.push_str(&format!("Overall: {} wide (x) x {} deep (y) x {} tall (z).\n", format_length(size.x), format_length(size.y), format_length(size.z)));
    }
    let list = crate::cutlist::cut_list(scene);
    for m in &list.materials {
        out.push_str(&format!("{}: ", m.name));
        let parts: Vec<String> = m
            .parts
            .iter()
            .map(|p| format!("{} {} x{} ({} x {} x {})", p.label, p.names.join("/"), p.count, format_length(p.length), format_length(p.width), format_length(p.thickness)))
            .collect();
        out.push_str(&parts.join("; "));
        out.push('\n');
    }
    if !scene.joints.is_empty() {
        let mut kinds: Vec<(JointKind, usize)> = Vec::new();
        for joint in &scene.joints {
            match kinds.iter_mut().find(|(k, _)| *k == joint.kind) {
                Some((_, n)) => *n += 1,
                None => kinds.push((joint.kind, 1)),
            }
        }
        out.push_str(&format!("Joints: {}.\n", kinds.iter().map(|(k, n)| format!("{n} {}", k.name().to_lowercase())).collect::<Vec<_>>().join(", ")));
    }
    let report = crate::movement::check(scene, scene.moisture_swing());
    if report.conflicts.is_empty() {
        out.push_str("Wood movement: no conflicts.\n");
    } else {
        for c in &report.conflicts {
            out.push_str(&format!("Wood movement warning: {}\n", c.describe()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const NIGHTSTAND: &str = r#"{
        "name": "Nightstand",
        "materials": [{ "id": "cherry", "library": "Cherry" }, { "id": "ply", "library": "Plywood" }],
        "parts": [
            { "name": "Left side", "material": "cherry", "size": [24, 14, "3/4\""], "at": [0, 0, 0], "length": "z", "width": "y" },
            { "name": "Right side", "material": "cherry", "size": [24, 14, 0.75], "at": [17.25, 0, 0], "length": "z", "width": "y" },
            { "name": "Bottom", "material": "cherry", "size": [16.5, 14, 0.75], "at": [0.75, 0, 2], "length": "x", "width": "y" },
            { "name": "Top", "material": "cherry", "size": [20, 16, 0.75], "at": [-1, -1, 24], "free_to_move": true, "tag": "Tops" }
        ],
        "joints": [
            { "kind": "let_in", "part": "Bottom", "into": "Left side", "options": { "depth": 0.25 } },
            { "kind": "dado", "part": "Bottom", "into": "Right side" }
        ],
        "groups": [{ "name": "Nightstand", "parts": ["Left side", "Right side", "Bottom", "Top"] }]
    }"#;

    #[test]
    fn a_recipe_builds_a_drawing_with_its_joints_and_group() {
        let recipe = parse(NIGHTSTAND).unwrap();
        let doc = build(&recipe).unwrap_or_else(|e| panic!("{e:?}"));
        let scene = doc.scene();
        assert_eq!(scene.root.instances().count(), 1, "one group");
        assert_eq!(scene.joints.len(), 2);
        let (lo, hi) = scene.bounds().unwrap();
        assert_eq!((lo.z, hi.z), (0.0, 24.75), "the sides start on the floor; the top sits on them");
        assert!((hi.x - lo.x - 20.0).abs() < 1e-9);
        let text = summary(&doc);
        assert!(text.contains("Cherry:") && text.contains("Joints: 2 dado") && text.contains("Wood movement: no conflicts"), "{text}");
        assert!(scene.tags().any(|(_, t)| t.name == "Tops"));
    }

    #[test]
    fn mistakes_come_back_naming_the_item() {
        let recipe = parse(
            r#"{
            "materials": [{ "id": "oak", "library": "Oak Tree" }],
            "parts": [
                { "name": "Shelf", "material": "walnut", "size": [20, 10, 0.75], "at": [0, 0, 30] },
                { "name": "Side", "size": [30, 10, 0.75], "length": "z", "width": "y" },
                { "name": "Side", "size": [30, 10, 0.75], "length": "q", "width": "y" }
            ],
            "joints": [
                { "kind": "let_in", "part": "Shelf", "into": "Side" },
                { "kind": "glue", "part": "Shelf", "into": "Side" },
                { "kind": "dowels", "part": "Shelf", "into": "Leg" }
            ]
        }"#,
        )
        .unwrap();
        let errors = build(&recipe).err().expect("errors");
        let all = errors.join("\n");
        assert!(all.contains("materials[0] (oak): \"Oak Tree\" isn't in the library"), "{all}");
        assert!(all.contains("parts[0] (Shelf): no material \"walnut\""), "{all}");
        assert!(all.contains("parts[2] (Side): length and width are axes"), "{all}");
        assert!(all.contains("joints[0] (let_in, Shelf into Side): Shelf doesn't touch Side"), "{all}");
        assert!(all.contains("\"glue\" isn't a joint"), "{all}");
        assert!(all.contains("joints[2] (dowels, Shelf into Leg): no part named \"Leg\""), "{all}");
        assert!(parse(r#"{ "partz": [] }"#).unwrap_err().contains("unknown field `partz`"));
    }

    #[test]
    fn templates_and_millimeters() {
        let recipe = parse(
            r#"{
            "units": "mm",
            "materials": [{ "id": "oak", "library": "White Oak" }],
            "templates": [{ "kind": "table", "name": "Hall table", "at": [1000, 0, 0], "wood": "oak", "sizes": { "width": 900, "depth": 400 } }]
        }"#,
        )
        .unwrap();
        let doc = build(&recipe).unwrap_or_else(|e| panic!("{e:?}"));
        let (lo, hi) = doc.scene().bounds().unwrap();
        assert!((lo.x - 1000.0 / 25.4).abs() < 1e-6, "{lo}");
        assert!((hi.x - lo.x - 900.0 / 25.4).abs() < 1e-6, "900 mm wide: {}", hi.x - lo.x);
        assert!(doc.scene().joints.len() >= 8, "the table's mortises and tenons");
        let bad = parse(r#"{ "templates": [{ "kind": "table", "sizes": { "colour": 3 } }] }"#).unwrap();
        assert!(build(&bad).err().expect("an error")[0].contains("Table has no size \"colour\""));
    }
}

#[cfg(test)]
mod guide {
    /// The example in the agents' guide builds, with no wood movement warnings.
    #[test]
    fn the_guides_example_builds() {
        let guide = include_str!("../../../docs/AGENTS.md");
        let start = guide.find("```json").expect("an example") + "```json".len();
        let example = &guide[start..start + guide[start..].find("```").unwrap()];
        let doc = super::build(&super::parse(example).unwrap()).unwrap_or_else(|e| panic!("{e:?}"));
        let summary = super::summary(&doc);
        assert!(summary.contains("Joints: 2 dado, 2 notch") && summary.contains("Wood movement: no conflicts"), "{summary}");
    }
}
