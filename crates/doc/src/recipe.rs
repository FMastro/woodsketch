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

use glam::{DAffine3, DVec3};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::angled::AngledJoint;
use crate::joinery::{Dovetails, JoineryError, JointKind, Slots, Spline};
use crate::part::{Cut, PartAxis, Side};
use crate::profile::{Edges, Profile};
use crate::template::Template;
use crate::units::{Units, format_length, parse_length, set_units, units};
use crate::{DefinitionKind, Document, Entity, InstanceId, MaterialId, MaterialKind};

/// A project to build.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
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
    /// Edge profiles, made after the parts and before the joints.
    #[serde(default)]
    pub profiles: Vec<ProfileSpec>,
    /// Tags to make, hidden or in folders (a tag only named on a part is made shown, at the top).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<TagSpec>,
}

/// A tag, or a folder of tags (`"folder": true`), `inside` a folder, `hidden` or shown.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TagSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hidden: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub folder: bool,
    /// The folder it's in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inside: Option<String>,
}

/// A material, by an `id` the parts use: one from the library (`"library": "Cherry"`), or your
/// own (`name`, `kind`, `color`).
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum Length {
    Number(f64),
    Text(String),
}

/// A board, placed as a box.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PartSpec {
    /// Its name (unique in the recipe: joints and groups use it).
    pub name: String,
    /// The name it shows in the drawing and cut list, when that isn't `name` (two parts both
    /// called "Stile": `name`s `Stile` and `Stile 2`, both `called` "Stile").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub called: Option<String>,
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
    /// A piece of hardware, not wood (a drawer slide, a pull): named here, left out of the cut
    /// list and counted in the shopping list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware: Option<String>,
    /// Fastened so it can move across its grain (tabletop buttons, slotted screws).
    #[serde(default)]
    pub free_to_move: bool,
    /// Which way its grain runs: `length` (the default) or `width`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grain: Option<String>,
    /// How it's sawn: `flat_sawn` or `quartersawn` (unknown if left out).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cut: Option<String>,
    /// Turned about an axis through `at` (its lowest corner before turning): a brace, a splayed leg.
    /// A list turns it about each in order (a compound angle).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn: Option<Turns>,
    /// Its reference face, edge and end, by where they face once placed: `top`, `bottom`,
    /// `front`, `back`, `left` or `right`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<Reference>,
    /// More of the same part (it becomes a component; a joint or profile on one is on all).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub copies: Vec<CopySpec>,
    /// Its shape, when it isn't a plain box (angled ends, a taper, a curve, cuts made by hand):
    /// its faces, in its own frame (x along its length, y its width, z its thickness, from its
    /// box's corner). Written when a drawing is read back, so changing the rest keeps it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<Shape>,
    /// Its outline seen on its face, cut through its thickness: points `[x, y]` along its length
    /// and width from its box's corner, or `{"arc_to": [x, y], "through": [x, y]}` for a curve
    /// from the point before (angled ends, tapers, curved aprons, a bracket).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outline: Vec<OutlinePoint>,
    /// Holes cut through it, each an outline like `outline` (a hand hold, a cut-out).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cutouts: Vec<Vec<OutlinePoint>>,
}

/// A point of an outline (see [`PartSpec::outline`]).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum OutlinePoint {
    Point([f64; 2]),
    Arc(ArcTo),
}

/// A curve from the point before to `arc_to`, through `through`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArcTo {
    pub arc_to: [f64; 2],
    pub through: [f64; 2],
}

/// One turn, or several made in order.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum Turns {
    One(Turn),
    Several(Vec<Turn>),
}

impl Turns {
    pub fn list(&self) -> &[Turn] {
        match self {
            Turns::One(t) => std::slice::from_ref(t),
            Turns::Several(v) => v,
        }
    }
}

/// A part's faces (see [`PartSpec::shape`]): each an outline, with any holes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    pub faces: Vec<ShapeFace>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShapeFace {
    pub outer: Vec<[f64; 3]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holes: Vec<Vec<[f64; 3]>>,
}

/// A turn about an axis: `{"axis": "z", "degrees": 90}` (counter-clockwise looking down the axis).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Turn {
    pub axis: String,
    pub degrees: f64,
}

/// A part's reference face, edge and end (see [`PartSpec::reference`]).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

/// A copy of a part: its own name (for joints), its lowest corner, turned about z first.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CopySpec {
    pub name: String,
    pub at: [Length; 3],
    /// Degrees about z (up), counter-clockwise from above: a leg turned to each corner.
    #[serde(default)]
    pub turn: f64,
    /// Its own tag, when it isn't the part's (`""` for none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// An edge profile on a part: `around` a face (its edges all round: `top`, `front`...), or `along`
/// the part's `length`, `width` or `thickness` (the four edges running that way).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProfileSpec {
    pub part: String,
    /// `roundover`, `chamfer`, `cove` or `ogee`.
    pub profile: String,
    pub size: Length,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub around: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub along: Option<String>,
    /// Only some of those edges, by where they face: around a face, the sides they're on
    /// (`["front", "left"]`); along a way, the corners, each two sides (`["top front"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<String>,
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// A joint between two parts that touch or overlap.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GroupSpec {
    pub name: String,
    pub parts: Vec<String>,
    #[serde(default)]
    pub component: bool,
    /// The name it shows, when that isn't `name` (as for parts).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub called: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// A drawer or door: it moves in its case, so the wood movement check doesn't hold its parts
    /// against the case's.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub moves: bool,
}

/// Piece-by-piece edits, for agents changing a drawing without rewriting its whole recipe. Each
/// says what's wrong (an unknown name, a name taken) without changing anything.
impl Recipe {
    /// Every name a joint, profile or group can use: parts, their copies, templates, groups.
    pub fn names(&self) -> Vec<&str> {
        let parts = self.parts.iter().flat_map(|p| std::iter::once(p.name.as_str()).chain(p.copies.iter().map(|c| c.name.as_str())));
        let templates = self.templates.iter().map(|t| t.name.as_deref().unwrap_or(t.kind.as_str()));
        parts.chain(templates).chain(self.groups.iter().map(|g| g.name.as_str())).collect()
    }

    /// Add a part (or replace the one with its name, with `replace`).
    pub fn add_part(&mut self, part: PartSpec, replace: bool) -> Result<(), String> {
        let taken = self.names().contains(&part.name.as_str());
        match self.parts.iter().position(|p| p.name == part.name) {
            Some(i) if replace => self.parts[i] = part,
            Some(_) => return Err(format!("There's already a part named \"{}\" (change it, or remove it first).", part.name)),
            None if taken => return Err(format!("\"{}\" is already the name of a copy, template or group.", part.name)),
            None => self.parts.push(part),
        }
        Ok(())
    }

    /// Change some of a part's fields (`{"at": [0, 0, 12]}`, `{"size": [30, 10, 0.75]}`), keeping
    /// the rest. Renaming it renames it in its joints, profiles and groups too.
    pub fn change_part(&mut self, name: &str, changes: &serde_json::Value) -> Result<(), String> {
        let index = self.parts.iter().position(|p| p.name == name).ok_or_else(|| self.no_part(name))?;
        let serde_json::Value::Object(changes) = changes else { return Err("Give the changes as an object of the part's fields.".into()) };
        let mut json = serde_json::to_value(&self.parts[index]).map_err(|e| e.to_string())?;
        for (key, value) in changes {
            json[key] = value.clone();
        }
        let part: PartSpec = serde_json::from_value(json).map_err(|e| format!("Those changes don't fit a part: {e}"))?;
        let renamed = part.name != name;
        if renamed && self.names().contains(&part.name.as_str()) {
            return Err(format!("\"{}\" is already used.", part.name));
        }
        let new = part.name.clone();
        self.parts[index] = part;
        if renamed {
            self.rename(name, &new);
        }
        Ok(())
    }

    /// Take out a part (or a copy, template or group) and everything that names it: its joints,
    /// profiles and place in groups. A group taken out leaves its members.
    pub fn remove(&mut self, name: &str) -> Result<(), String> {
        let before = (self.parts.len(), self.templates.len(), self.groups.len(), self.parts.iter().map(|p| p.copies.len()).sum::<usize>());
        // A part's copies go with it.
        let mut gone: Vec<String> = vec![name.to_owned()];
        if let Some(p) = self.parts.iter().find(|p| p.name == name) {
            gone.extend(p.copies.iter().map(|c| c.name.clone()));
        }
        self.parts.retain(|p| p.name != name);
        for p in &mut self.parts {
            p.copies.retain(|c| c.name != name);
        }
        self.templates.retain(|t| t.name.as_deref().unwrap_or(t.kind.as_str()) != name);
        self.groups.retain(|g| g.name != name);
        let after = (self.parts.len(), self.templates.len(), self.groups.len(), self.parts.iter().map(|p| p.copies.len()).sum::<usize>());
        if before == after {
            return Err(self.no_part(name));
        }
        self.joints.retain(|j| !gone.contains(&j.part) && !gone.contains(&j.into));
        self.profiles.retain(|p| !gone.contains(&p.part));
        for g in &mut self.groups {
            g.parts.retain(|p| !gone.contains(p));
        }
        self.groups.retain(|g| !g.parts.is_empty());
        Ok(())
    }

    /// Take out the joints between two parts (of one kind, or all of them).
    pub fn remove_joint(&mut self, part: &str, into: &str, kind: Option<&str>) -> Result<usize, String> {
        let before = self.joints.len();
        self.joints.retain(|j| !(j.part == part && j.into == into && kind.is_none_or(|k| j.kind.eq_ignore_ascii_case(k))));
        match before - self.joints.len() {
            0 => Err(format!("No joint of {part} into {into}{}.", kind.map(|k| format!(" ({k})")).unwrap_or_default())),
            n => Ok(n),
        }
    }

    fn rename(&mut self, from: &str, to: &str) {
        for j in &mut self.joints {
            for name in [&mut j.part, &mut j.into] {
                if name == from {
                    *name = to.to_owned();
                }
            }
        }
        for p in &mut self.profiles {
            if p.part == from {
                p.part = to.to_owned();
            }
        }
        for g in &mut self.groups {
            for name in &mut g.parts {
                if name == from {
                    *name = to.to_owned();
                }
            }
        }
    }

    fn no_part(&self, name: &str) -> String {
        format!("No part named \"{name}\" (there are: {}).", self.names().join(", "))
    }
}

/// The recipe format as a JSON Schema, for editors and agents to check a recipe against before
/// building it.
pub fn schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(Recipe)).unwrap_or_default()
}

/// Read a recipe from JSON, with a message saying where it's wrong.
pub fn parse(json: &str) -> Result<Recipe, String> {
    serde_json::from_str(json).map_err(|e| format!("The recipe isn't valid JSON for WoodSketch: {e}"))
}

/// Build the drawing. Every problem found is listed, each naming its item.
pub fn build(recipe: &Recipe) -> Result<Document, Vec<String>> {
    build_reporting(recipe, &mut |_, _, _| {})
}

/// [`build`], saying how far along it is: `progress(done, total, item)` as each part, profile,
/// template, joint and group is started (joints take the longest).
pub fn build_reporting(recipe: &Recipe, progress: &mut dyn FnMut(usize, usize, &str)) -> Result<Document, Vec<String>> {
    let before = units();
    let mm = match recipe.units.as_deref().map(str::to_ascii_lowercase).as_deref() {
        None | Some("in" | "inch" | "inches") => false,
        Some("mm" | "millimeters" | "millimetres") => true,
        Some(other) => return Err(vec![format!("units: \"{other}\" isn't one of \"in\" or \"mm\".")]),
    };
    set_units(if mm { Units::Millimeters } else { Units::Inches });
    let result = build_in(recipe, mm, progress);
    set_units(before);
    result
}

fn build_in(recipe: &Recipe, mm: bool, progress: &mut dyn FnMut(usize, usize, &str)) -> Result<Document, Vec<String>> {
    let mut doc = Document::default();
    let total = recipe.parts.len() + recipe.profiles.len() + recipe.templates.len() + recipe.joints.len() + recipe.groups.len();
    let mut step = 0;
    let mut report = |what: &str| {
        progress(step, total, what);
        step += 1;
    };
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
        report(&what);
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
        // A shape of its own instead of the box, in the board's frame: its faces, or an outline.
        let model = match (&p.shape, p.outline.is_empty()) {
            (Some(_), false) => {
                errors.push(format!("{what}: give a shape or an outline, not both."));
                None
            }
            (Some(shape), true) => Some(shape_model(shape)),
            (None, false) => match outline_model(&p.outline, &p.cutouts, if mm { 25.4 } else { 1.0 }, [l, w, t]) {
                Ok(model) => Some(model),
                Err(e) => {
                    errors.push(format!("{what}: {e}"));
                    None
                }
            },
            (None, true) => {
                if !p.cutouts.is_empty() {
                    errors.push(format!("{what}: cutouts need an outline too."));
                }
                None
            }
        };
        if let Some(model) = model
            && let Some(def) = doc.scene().root.instance(id).map(|i| i.definition)
        {
            doc.edit_scene(|scene, _| {
                if let Some(d) = scene.definition_mut(def) {
                    d.container.model = model;
                }
            });
        }
        doc.rename_instance(id, p.called.as_deref().unwrap_or(&p.name));
        if p.free_to_move {
            doc.edit_part(id, |info| info.free_to_move = true);
        }
        if let Some(piece) = &p.hardware
            && let Some(def) = doc.scene().root.instance(id).map(|i| i.definition)
        {
            doc.edit_scene(|scene, _| {
                if let Some(d) = scene.definition_mut(def) {
                    d.hardware = Some(piece.clone());
                }
            });
        }
        if let Some(turns) = &p.turn {
            let mut spin = DAffine3::IDENTITY;
            for turn in turns.list() {
                match axis(&turn.axis) {
                    Some(a) => spin = DAffine3::from_axis_angle(DVec3::AXES[a], turn.degrees.to_radians()) * spin,
                    None => errors.push(format!("{what}: turn axis \"{}\" isn't x, y or z.", turn.axis)),
                }
            }
            let spin = DAffine3::from_translation(at) * spin * DAffine3::from_translation(-at);
            place(&mut doc, id, |t| spin * t);
        }
        match p.grain.as_deref().map(str::to_ascii_lowercase).as_deref() {
            None => {}
            Some("length") => doc.edit_part(id, |info| info.grain = Some(PartAxis::X)),
            Some("width") => doc.edit_part(id, |info| info.grain = Some(PartAxis::Y)),
            Some(other) => errors.push(format!("{what}: grain \"{other}\" isn't \"length\" or \"width\".")),
        }
        match p.cut.as_deref().map(|c| c.to_ascii_lowercase().replace([' ', '-'], "_")).as_deref() {
            None => {}
            Some("flat_sawn" | "flatsawn" | "plain_sawn") => doc.edit_part(id, |info| info.cut = Cut::FlatSawn),
            Some("quartersawn" | "quarter_sawn" | "rift" | "rift_sawn") => doc.edit_part(id, |info| info.cut = Cut::QuarterSawn),
            Some(other) => errors.push(format!("{what}: cut \"{other}\" isn't \"flat_sawn\" or \"quartersawn\".")),
        }
        if let Some(reference) = &p.reference {
            for (side, kind) in [(&reference.face, "face"), (&reference.edge, "edge"), (&reference.end, "end")] {
                let Some(side) = side else { continue };
                let Some(normal) = direction(side) else {
                    errors.push(format!("{what}: reference {kind} \"{side}\" isn't top, bottom, front, back, left or right."));
                    continue;
                };
                // Its own axes: the face that way once placed.
                let local = doc.scene().root.instance(id).map(|i| i.transform.matrix3.inverse() * normal).unwrap_or(normal);
                match doc.set_reference(id, local) {
                    Ok(said) if said.to_lowercase().contains(kind) => {}
                    Ok(said) => errors.push(format!("{what}: its {side} side isn't a {kind} ({said}).")),
                    Err(e) => errors.push(format!("{what}: {e}")),
                }
            }
        }
        if named.insert(p.name.clone(), id).is_some() {
            errors.push(format!("{what}: another part or template is named \"{}\"; names must be unique.", p.name));
        }
        // Copies: the part becomes a component, each copy turned about z, then put at its corner.
        if !p.copies.is_empty() {
            doc.selection_mut().clear();
            doc.selection_mut().extend([Entity::Instance(id)]);
            let id = doc.group_selection(DefinitionKind::Component).unwrap_or(id);
            named.insert(p.name.clone(), id);
            for (k, c) in p.copies.iter().enumerate() {
                let what = format!("{what} copies[{k}] ({})", c.name);
                let to = DVec3::from_array([0, 1, 2].map(|i| length(&c.at[i], &what, &mut errors)));
                let spin = DAffine3::from_axis_angle(DVec3::Z, c.turn.to_radians());
                let Ok(made) = doc.transform_entities(&[Entity::Instance(id)], &[spin], true) else {
                    errors.push(format!("{what}: couldn't place the copy."));
                    continue;
                };
                let Some(Entity::Instance(copy)) = made.first().copied() else { continue };
                let lo = lowest(&doc, copy);
                place(&mut doc, copy, |t| DAffine3::from_translation(to - lo) * t);
                doc.rename_instance(copy, p.called.as_deref().unwrap_or(&p.name));
                if named.insert(c.name.clone(), copy).is_some() {
                    errors.push(format!("{what}: another part or template is named \"{}\"; names must be unique.", c.name));
                }
            }
        }
    }

    // Profiles, on the parts as they are (before joints change their shape).
    for (i, pr) in recipe.profiles.iter().enumerate() {
        let what = format!("profiles[{i}] ({}, {})", pr.profile, pr.part);
        report(&what);
        let Some(&id) = named.get(&pr.part) else {
            errors.push(format!("{what}: no part named \"{}\".", pr.part));
            continue;
        };
        let profile = match pr.profile.to_ascii_lowercase().as_str() {
            "roundover" | "round_over" => Profile::Roundover,
            "chamfer" => Profile::Chamfer,
            "cove" => Profile::Cove,
            "ogee" => Profile::Ogee,
            other => {
                errors.push(format!("{what}: \"{other}\" isn't a profile (roundover, chamfer, cove or ogee)."));
                continue;
            }
        };
        let size = length(&pr.size, &what, &mut errors);
        let transform = doc.scene().root.instance(id).map_or(DAffine3::IDENTITY, |i| i.transform);
        let edges = match (&pr.around, &pr.along) {
            (Some(face), None) => match direction(face) {
                Some(normal) => {
                    let side = Side::facing(transform.matrix3.inverse() * normal);
                    Edges::AroundFace { axis: side.axis.index(), positive: side.positive, sides: [true; 4] }
                }
                None => {
                    errors.push(format!("{what}: around \"{face}\" isn't top, bottom, front, back, left or right."));
                    continue;
                }
            },
            (None, Some(way)) => match way.to_ascii_lowercase().as_str() {
                // A board's own axes: length, width, thickness.
                "length" => Edges::Along { axis: 0, corners: [true; 4] },
                "width" => Edges::Along { axis: 1, corners: [true; 4] },
                "thickness" => Edges::Along { axis: 2, corners: [true; 4] },
                other => {
                    errors.push(format!("{what}: along \"{other}\" isn't length, width or thickness."));
                    continue;
                }
            },
            _ => {
                errors.push(format!("{what}: give either \"around\" (a face: top, front...) or \"along\" (length, width or thickness)."));
                continue;
            }
        };
        // Only some of them: each edge by the sides it's on.
        let edges = if pr.edges.is_empty() {
            edges
        } else {
            match some_edges(edges, &pr.edges, transform.matrix3) {
                Ok(edges) => edges,
                Err(e) => {
                    errors.push(format!("{what}: {e}"));
                    continue;
                }
            }
        };
        if let Err(e) = doc.profile_edges(id, profile, size, edges) {
            errors.push(format!("{what}: {e}"));
        }
    }

    // Templates: built where they're put.
    for (i, t) in recipe.templates.iter().enumerate() {
        let name = t.name.clone().unwrap_or_else(|| t.kind.clone());
        let what = format!("templates[{i}] ({name})");
        report(&what);
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
        report(&what);
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

    // Tags: the folders and tags listed, then on the parts and templates.
    for (i, t) in recipe.tags.iter().enumerate() {
        if doc.scene().tags().any(|(_, have)| have.name == t.name) {
            errors.push(format!("tags[{i}] ({}): another tag has that name.", t.name));
            continue;
        }
        doc.edit_scene(|scene, _| if t.folder { scene.add_tag_folder(t.name.clone(), None) } else { scene.add_tag(t.name.clone()) });
    }
    for (i, t) in recipe.tags.iter().enumerate() {
        let Some(folder) = &t.inside else { continue };
        let find = |name: &str| doc.scene().tags().find(|(_, have)| have.name == name).map(|(id, _)| id);
        let (Some(tag), Some(into)) = (find(&t.name), find(folder)) else {
            errors.push(format!("tags[{i}] ({}): no folder named \"{folder}\" in the tags.", t.name));
            continue;
        };
        if !doc.edit_scene(|scene, _| scene.move_tag(tag, Some(into))) {
            errors.push(format!("tags[{i}] ({}): \"{folder}\" isn't a folder (give it \"folder\": true), or it's inside this one.", t.name));
        }
    }
    // (Copies take the part's tag unless they have their own.)
    let mut tagged: Vec<(String, &String, Option<&String>)> = Vec::new();
    for (i, p) in recipe.parts.iter().enumerate() {
        tagged.push((format!("parts[{i}] ({})", p.name), &p.name, p.tag.as_ref()));
        for (k, c) in p.copies.iter().enumerate() {
            tagged.push((format!("parts[{i}] ({}) copies[{k}] ({})", p.name, c.name), &c.name, c.tag.as_ref().or(p.tag.as_ref()).filter(|t| !t.is_empty())));
        }
    }
    tagged.extend(recipe.templates.iter().enumerate().filter_map(|(i, t)| Some((format!("templates[{i}]"), t.name.as_ref()?, t.tag.as_ref()))));
    for (what, name, tag) in tagged {
        let (Some(tag), Some(&id)) = (tag, named.get(name)) else { continue };
        if !tag_instance(&mut doc, id, tag) {
            errors.push(format!("{what}: couldn't tag it \"{tag}\"."));
        }
    }

    // Groups, last (joints are made between parts side by side).
    for (i, g) in recipe.groups.iter().enumerate() {
        let what = format!("groups[{i}] ({})", g.name);
        report(&what);
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
                doc.rename_instance(id, g.called.as_deref().unwrap_or(&g.name));
                if g.moves
                    && let Some(def) = doc.scene().root.instance(id).map(|i| i.definition)
                {
                    doc.edit_scene(|scene, _| {
                        if let Some(d) = scene.definition_mut(def) {
                            d.moves = true;
                        }
                    });
                }
                if let Some(tag) = &g.tag
                    && !tag_instance(&mut doc, id, tag)
                {
                    errors.push(format!("{what}: couldn't tag it \"{tag}\"."));
                }
                for name in &g.parts {
                    named.remove(name);
                }
                named.insert(g.name.clone(), id);
            }
            None => errors.push(format!("{what}: couldn't group them.")),
        }
    }
    doc.selection_mut().clear();
    // Hidden last, so what's on them could be tagged and grouped.
    for t in recipe.tags.iter().filter(|t| t.hidden) {
        doc.edit_scene(|scene, _| {
            let id = scene.tags().find(|(_, have)| have.name == t.name).map(|(id, _)| id);
            if let Some(tag) = id.and_then(|id| scene.tag_mut(id)) {
                tag.visible = false;
            }
        });
    }
    if errors.is_empty() { Ok(doc) } else { Err(errors) }
}

/// `edges` (all of a face's, or all four one way) cut down to those named by where they face
/// once placed (`placed`: the part's own axes in the world): a face's sides (`front`), or the
/// corners running one way (`top front`).
fn some_edges(edges: Edges, names: &[String], placed: glam::DMat3) -> Result<Edges, String> {
    // A side of the part by where it faces: its own axis and which end.
    let side = |name: &str| -> Result<Side, String> {
        let normal = direction(name).ok_or_else(|| format!("\"{name}\" isn't top, bottom, front, back, left or right."))?;
        Ok(Side::facing(placed.inverse() * normal))
    };
    match edges {
        Edges::AroundFace { axis, positive, .. } => {
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            let mut sides = [false; 4];
            for name in names {
                let s = side(name.trim())?;
                let k = s.axis.index();
                if k != u && k != v {
                    return Err(format!("\"{name}\" isn't a side of that face (it's the face, or the one opposite)."));
                }
                sides[if k == u { 0 } else { 2 } + usize::from(s.positive)] = true;
            }
            Ok(Edges::AroundFace { axis, positive, sides })
        }
        Edges::Along { axis, .. } => {
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            let mut corners = [false; 4];
            for name in names {
                let words: Vec<&str> = name.split(|c: char| c.is_whitespace() || c == '-' || c == ',' || c == '/').filter(|w| !w.is_empty()).collect();
                let [a, b] = words.as_slice() else {
                    return Err(format!("\"{name}\": name a corner by the two sides it's between (\"top front\")."));
                };
                let (a, b) = (side(a)?, side(b)?);
                let (on_u, on_v) = match (a.axis.index(), b.axis.index()) {
                    (x, y) if x == u && y == v => (a.positive, b.positive),
                    (x, y) if x == v && y == u => (b.positive, a.positive),
                    _ => return Err(format!("\"{name}\" isn't a corner running that way.")),
                };
                // Corners: (u-, v-), (u+, v-), (u+, v+), (u-, v+).
                corners[match (on_u, on_v) {
                    (false, false) => 0,
                    (true, false) => 1,
                    (true, true) => 2,
                    (false, true) => 3,
                }] = true;
            }
            Ok(Edges::Along { axis, corners })
        }
    }
}

/// Put the tag named `name` (made if it's new) on a placed part, template or group.
fn tag_instance(doc: &mut Document, id: InstanceId, name: &str) -> bool {
    let existing = doc.scene().tags().find(|(_, t)| t.name == name && !t.folder).map(|(id, _)| id);
    let tag = existing.unwrap_or_else(|| doc.edit_scene(|scene, _| scene.add_tag(name.to_owned())));
    doc.selection_mut().clear();
    doc.selection_mut().extend([Entity::Instance(id)]);
    let done = doc.tag_selection(Some(tag)) > 0;
    doc.selection_mut().clear();
    done
}

/// An outline (and holes) in a board's length x width, cut through its thickness `size[2]`.
pub(crate) fn outline_model(outline: &[OutlinePoint], cutouts: &[Vec<OutlinePoint>], unit: f64, size: [f64; 3]) -> Result<crate::Model, String> {
    let ring = |points: &[OutlinePoint]| -> Result<Vec<DVec3>, String> {
        let at = |p: [f64; 2]| DVec3::new(p[0] / unit, p[1] / unit, 0.0);
        let mut out: Vec<DVec3> = Vec::new();
        for point in points {
            match point {
                OutlinePoint::Point(p) => out.push(at(*p)),
                OutlinePoint::Arc(arc) => {
                    let Some(&from) = out.last() else { return Err("an outline can't start with an arc (give the point it starts from first).".into()) };
                    let (through, to) = (at(arc.through), at(arc.arc_to));
                    let Some((center, normal)) = crate::curves::circle_through(from, through, to) else {
                        return Err(format!("the arc to [{}, {}] is straight (its three points are in a line).", arc.arc_to[0], arc.arc_to[1]));
                    };
                    let angle = crate::curves::angle_about(from - center, to - center, normal);
                    // As the Arc tool draws them: 12 sides, more past half a circle.
                    let segments = ((angle / (std::f64::consts::TAU / 24.0)).ceil() as u32).max(12);
                    let mut swept = crate::curves::sweep_points(center, from, normal, angle, segments);
                    if let Some(end) = swept.last_mut() {
                        *end = to;
                    }
                    out.extend(swept.into_iter().skip(1));
                }
            }
        }
        out.dedup_by(|a, b| a.distance(*b) < crate::EPS);
        if out.len() > 1 && out[0].distance(out[out.len() - 1]) < crate::EPS {
            out.pop();
        }
        if out.len() < 3 {
            return Err("an outline needs at least three points.".into());
        }
        let (l, w) = (size[0], size[1]);
        if out.iter().any(|p| p.x < -1e-3 || p.y < -1e-3 || p.x > l + 1e-3 || p.y > w + 1e-3) {
            return Err(format!("its outline goes outside its size ({} x {}): points run from [0, 0] to [length, width].", format_length(l), format_length(w)));
        }
        // Wound counter-clockwise, so the face looks up its thickness.
        let area: f64 = (0..out.len()).map(|i| out[i].x * out[(i + 1) % out.len()].y - out[(i + 1) % out.len()].x * out[i].y).sum();
        if area < 0.0 {
            out.reverse();
        }
        Ok(out)
    };
    let outer = ring(outline)?;
    let holes = cutouts.iter().map(|c| ring(c).map_err(|e| format!("a cutout: {e}"))).collect::<Result<Vec<_>, _>>()?;
    let mut model = crate::Model::default();
    let face = model.add_face_with_holes(&outer, &holes).ok_or("its outline crosses itself or folds back.")?;
    model.push_pull(face, size[2]).map_err(|e| format!("couldn't cut its outline through its thickness ({e:?})."))?;
    Ok(model)
}

/// A part's own faces as a model, taken as they are (they were faces in the drawing they came
/// from): corners at one place shared, each loop's edges added once, a face's front by its
/// winding. Faces too small to have a side are left out.
fn shape_model(shape: &Shape) -> crate::Model {
    let mut model = crate::Model::default();
    let mut corners: HashMap<[i64; 3], crate::VertexId> = HashMap::new();
    let mut edges: std::collections::HashSet<(crate::VertexId, crate::VertexId)> = std::collections::HashSet::new();
    let mut vertex = |model: &mut crate::Model, p: &[f64; 3]| -> crate::VertexId {
        let key = p.map(|c| (c * 100_000.0).round() as i64);
        *corners.entry(key).or_insert_with(|| model.push_vertex(DVec3::from_array(*p)))
    };
    for face in &shape.faces {
        let mut ring = |model: &mut crate::Model, points: &[[f64; 3]]| -> Vec<crate::VertexId> {
            let mut ids: Vec<crate::VertexId> = points.iter().map(|p| vertex(model, p)).collect();
            ids.dedup();
            if ids.len() > 1 && ids.first() == ids.last() {
                ids.pop();
            }
            ids
        };
        let outer = ring(&mut model, &face.outer);
        let holes: Vec<Vec<crate::VertexId>> = face.holes.iter().map(|h| ring(&mut model, h)).filter(|h| h.len() >= 3).collect();
        let points: Vec<DVec3> = outer.iter().map(|&v| model.vertex(v)).collect();
        let newell = (0..points.len()).fold(DVec3::ZERO, |n, i| n + points[i].cross(points[(i + 1) % points.len()]));
        let Some(normal) = newell.try_normalize().filter(|_| outer.len() >= 3) else { continue };
        for ids in std::iter::once(&outer).chain(&holes) {
            for i in 0..ids.len() {
                let (a, b) = (ids[i], ids[(i + 1) % ids.len()]);
                if a != b && edges.insert((a.min(b), a.max(b))) {
                    model.push_edge(a, b);
                }
            }
        }
        model.push_face(outer, holes, normal, None);
    }
    model
}

/// A direction by name, as a part is placed: `top` is up (+z), `front` toward you (-y), `right` +x.
fn direction(name: &str) -> Option<DVec3> {
    match name.trim().to_ascii_lowercase().as_str() {
        "top" | "up" => Some(DVec3::Z),
        "bottom" | "down" => Some(-DVec3::Z),
        "front" => Some(-DVec3::Y),
        "back" | "rear" => Some(DVec3::Y),
        "left" => Some(-DVec3::X),
        "right" => Some(DVec3::X),
        _ => None,
    }
}

/// Change a top-level instance's placement.
fn place(doc: &mut Document, id: InstanceId, change: impl FnOnce(DAffine3) -> DAffine3) {
    doc.edit_scene(|scene, _| {
        if let Some(inst) = scene.root.instance_mut(id) {
            inst.transform = change(inst.transform);
        }
    });
}

/// The lowest corner (smallest x, y, z) of a top-level instance's box.
fn lowest(doc: &Document, id: InstanceId) -> DVec3 {
    let corners = doc.scene().root.instance(id).map(|inst| doc.scene().instance_box_corners(inst)).unwrap_or_default();
    corners.iter().copied().reduce(DVec3::min).unwrap_or(DVec3::ZERO)
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
    let number = |key: &str| j.options.get(key).and_then(serde_json::Value::as_f64);
    let len = |key: &str, default: f64| number(key).map_or(default, |v| v / if mm { 25.4 } else { 1.0 });
    let kind = j.kind.to_ascii_lowercase().replace([' ', '-'], "_");
    // Each joint made with its sizes, kept on it in the Joinery panel's order (so Change, and a
    // recipe read back from the drawing, start from them).
    let (made, sizes): (Result<(), JoineryError>, Vec<f64>) = match kind.as_str() {
        // Parts at an angle to each other.
        "angled_mortise_and_tenon" | "angled_tenon" => {
            let fit = doc.angled_fit(part, into)?;
            // A third of the tenoned part's thickness, set in a quarter inch from its edges.
            let thickness = len("thickness", (fit.thickness.0 / 3.0).max(0.25));
            let shoulder = len("shoulder", 0.25);
            let fit = len("fit", 0.0);
            (doc.angled_joint(part, into, AngledJoint::MortiseAndTenon { thickness, shoulder, fit }), vec![thickness, shoulder, 0.0, fit])
        }
        "housing" | "angled_housing" => (doc.angled_joint(part, into, AngledJoint::Housing), vec![0.0, 0.0, 1.0]),
        _ => {
            let fit = doc.joint_fit(part, into)?;
            let count = |default: u32| number("count").map_or(default, |v| v.max(1.0) as u32);
            let (thick, wide) = fit.contact;
            match kind.as_str() {
                "let_in" | "dado" | "groove" | "rabbet" => {
                    let depth = len("depth", fit.let_in_depth);
                    (doc.let_in(part, into, depth).map(|_| ()), vec![depth])
                }
                "cut_overlap" => (doc.cut_overlap(part, into).map(|_| ()), Vec::new()),
                "mortise_and_tenon" | "tenon" => {
                    let mut tenon = fit.tenon;
                    tenon.length = len("length", tenon.length);
                    tenon.thickness = len("thickness", tenon.thickness);
                    tenon.shoulder = len("shoulder", tenon.shoulder);
                    tenon.fit = len("fit", tenon.fit);
                    let sizes = vec![tenon.length, tenon.thickness, tenon.shoulder, tenon.fit, 0.0, f64::from(u8::from(tenon.haunch > 0.0))];
                    (doc.mortise_and_tenon(part, into, tenon), sizes)
                }
                "dowels" => {
                    let mut dowels = fit.dowels;
                    dowels.count = count(dowels.count);
                    dowels.diameter = len("diameter", dowels.diameter);
                    dowels.depth = len("depth", dowels.depth);
                    dowels.fit = len("fit", dowels.fit);
                    let sizes = vec![f64::from(dowels.count), dowels.diameter, dowels.depth, dowels.fit];
                    (doc.dowels(part, into, dowels), sizes)
                }
                "biscuits" => {
                    let size = Slots::suggested_biscuit(thick, wide);
                    let mut slots = Slots::biscuits(size, wide);
                    slots.count = count(slots.count);
                    slots.fit = len("fit", 0.0);
                    let sizes = vec![size as f64, f64::from(slots.count), slots.fit];
                    (doc.slots(part, into, &slots), sizes)
                }
                "loose_tenons" => {
                    let size = Slots::suggested_loose_tenon(thick);
                    let mut slots = Slots::loose_tenons(size, wide);
                    slots.count = count(slots.count);
                    slots.fit = len("fit", 0.0);
                    let sizes = vec![size as f64, f64::from(slots.count), slots.fit];
                    (doc.slots(part, into, &slots), sizes)
                }
                "splines" | "spline" => (doc.slots(part, into, &Spline::for_stock(thick)), Vec::new()),
                "pocket_screws" | "pocket_holes" => {
                    let (n, screw) = (count(2), len("screw", 1.25));
                    (doc.pocket_screws(part, into, n, screw), vec![f64::from(n), screw])
                }
                "dovetails" | "dovetail" => {
                    let tails = number("tails").map_or(Dovetails::suggested(wide), |v| v.max(1.0) as u32);
                    let half_blind = j.options.get("half_blind").and_then(serde_json::Value::as_bool).unwrap_or(false);
                    let mut style = Dovetails::new(tails, half_blind);
                    style.pin = len("pin", style.pin);
                    style.half_pin = len("half_pin", style.half_pin);
                    (doc.dovetails(part, into, style), vec![f64::from(tails), f64::from(u8::from(half_blind)), style.pin, style.half_pin])
                }
                "box_joint" | "finger_joint" => {
                    let finger = len("finger", fit.finger);
                    (doc.box_joint(part, into, finger), vec![finger])
                }
                "half_lap" => (doc.half_lap(part, into), Vec::new()),
                other => {
                    return Err(JoineryError::Geometry(format!(
                        "\"{other}\" isn't a joint WoodSketch makes (let_in, cut_overlap, mortise_and_tenon, dowels, biscuits, loose_tenons, splines, pocket_screws, dovetails, box_joint, half_lap; at an angle, angled_mortise_and_tenon or housing)."
                    )));
                }
            }
        }
    };
    made?;
    if !sizes.is_empty() {
        doc.note_joint_sizes(sizes);
    }
    Ok(())
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
    fn edits_change_one_piece_and_keep_the_rest_in_step() {
        let mut recipe = parse(NIGHTSTAND).unwrap();
        // Move the bottom up and rename it: its joints and group follow the new name.
        recipe.change_part("Bottom", &serde_json::json!({ "at": [0.75, 0, 4], "name": "Lower shelf" })).unwrap();
        assert!(recipe.joints.iter().all(|j| j.part == "Lower shelf"));
        assert!(recipe.groups[0].parts.contains(&"Lower shelf".to_owned()));
        let doc = build(&recipe).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(doc.scene().joints.len(), 2);
        // A new part, a joint taken out, a part taken out with what names it.
        let shelf: PartSpec = serde_json::from_value(serde_json::json!({ "name": "Shelf", "material": "cherry", "size": [16.5, 14, 0.75], "at": [0.75, 0, 12] })).unwrap();
        recipe.add_part(shelf.clone(), false).unwrap();
        assert!(recipe.add_part(shelf, false).unwrap_err().contains("already a part named \"Shelf\""));
        assert_eq!(recipe.remove_joint("Lower shelf", "Right side", None), Ok(1));
        recipe.remove("Left side").unwrap();
        assert!(recipe.joints.is_empty() && !recipe.groups[0].parts.contains(&"Left side".to_owned()));
        assert!(recipe.remove("Leg").unwrap_err().contains("No part named \"Leg\""));
        assert!(recipe.change_part("Top", &serde_json::json!({ "size": "big" })).unwrap_err().contains("don't fit a part"));
        build(&recipe).unwrap_or_else(|e| panic!("{e:?}"));
    }

    #[test]
    fn the_schema_describes_every_part_of_a_recipe() {
        let schema = schema();
        let text = schema.to_string();
        for key in ["parts", "joints", "profiles", "templates", "groups", "copies", "reference", "turn"] {
            assert!(text.contains(&format!("\"{key}\"")), "{key}");
        }
        let part = &schema["$defs"]["PartSpec"];
        assert_eq!(part["additionalProperties"], false, "unknown fields are refused, as build does");
        assert!(part["required"].as_array().unwrap().iter().any(|r| r == "size"));
        assert!(text.contains("A board, placed as a box"), "the docs come along");
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
    fn outlines_with_curves_and_cutouts_and_compound_turns() {
        // A 600 x 150 x 19 mm apron with a curve along its bottom edge and a hand hold; a leg
        // splayed both ways.
        let recipe = parse(
            r#"{ "units": "mm", "materials": [{ "id": "oak", "library": "Red Oak" }], "parts": [
                { "name": "Apron", "material": "oak", "size": [600, 150, 19], "at": [0, 0, 0], "length": "x", "width": "z",
                  "outline": [[0, 0], { "arc_to": [600, 0], "through": [300, 50] }, [600, 150], [0, 150]],
                  "cutouts": [[[250, 100], [350, 100], [350, 130], [250, 130]]] },
                { "name": "Leg", "material": "oak", "size": [700, 45, 45], "at": [0, 300, 0], "length": "z", "width": "x",
                  "turn": [{ "axis": "x", "degrees": 8 }, { "axis": "y", "degrees": -8 }] }
            ] }"#,
        )
        .unwrap();
        let doc = build(&recipe).unwrap_or_else(|e| panic!("{e:?}"));
        let scene = doc.scene();
        let apron = scene.root.instances().find(|(_, i)| scene.definition(i.definition).unwrap().name == "Apron").unwrap().1;
        let model = &scene.definition(apron.definition).unwrap().container.model;
        assert!(model.faces().count() > 20, "a curved edge of many faces");
        assert!(model.faces().any(|(_, f)| f.holes.len() == 1), "the hand hold goes through");
        // The cut list still knows each part's own size.
        let parts: Vec<[f64; 3]> = crate::cutlist::cut_list(doc.scene()).materials.iter().flat_map(|m| m.parts.iter().map(|p| [p.length, p.width, p.thickness])).collect();
        assert!(parts.iter().any(|s| (s[0] - 700.0 / 25.4).abs() < 1e-3 && (s[1] - 45.0 / 25.4).abs() < 1e-3), "{parts:?}");
        // Reading it back keeps the outline and the turns.
        let (read, _) = crate::recipe_of::from_document(&doc);
        let again = build(&read).unwrap_or_else(|e| panic!("{e:?}"));
        let count = |d: &Document| d.scene().root.instances().map(|(_, i)| d.scene().definition(i.definition).unwrap().container.model.faces().count()).sum::<usize>();
        assert_eq!(count(&again), count(&doc));
        let bad = parse(
            r#"{ "parts": [{ "name": "A", "size": [10, 4, 1], "outline": [{ "arc_to": [10, 0], "through": [5, 1] }, [10, 4], [0, 4]] },
                { "name": "B", "size": [10, 4, 1], "outline": [[0, 0], [12, 0], [0, 4]] }] }"#,
        )
        .unwrap();
        let Err(errors) = build(&bad) else { panic!("should fail") };
        let all = errors.join("\n");
        assert!(all.contains("can't start with an arc") && all.contains("goes outside its size"), "{all}");
    }

    #[test]
    fn copies_turns_profiles_references_and_an_angled_joint() {
        let recipe = parse(
            r#"{
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "parts": [
                { "name": "Post", "material": "oak", "size": [48, 3.5, 1.5], "length": "z", "width": "x" },
                { "name": "Brace", "material": "oak", "size": [20, 3.5, 1.5], "at": [2.5, 0, 20], "length": "x", "width": "z",
                  "turn": { "axis": "y", "degrees": -45 } },
                { "name": "Leg", "material": "oak", "size": [28, 1.5, 1.5], "at": [0, 30, 0], "length": "z", "width": "x",
                  "grain": "length", "cut": "quartersawn", "reference": { "face": "front", "edge": "left" },
                  "copies": [{ "name": "Leg 2", "at": [20, 30, 0], "turn": 90 }] },
                { "name": "Shelf", "material": "oak", "size": [20, 8, 0.75], "at": [40, 0, 0] }
            ],
            "profiles": [
                { "part": "Shelf", "profile": "roundover", "size": "1/4\"", "around": "top" },
                { "part": "Leg", "profile": "chamfer", "size": 0.125, "along": "length" }
            ],
            "joints": [{ "kind": "angled_mortise_and_tenon", "part": "Brace", "into": "Post" }]
        }"#,
        )
        .unwrap();
        let doc = build(&recipe).unwrap_or_else(|e| panic!("{e:?}"));
        let scene = doc.scene();
        assert_eq!(scene.joints.len(), 1, "the brace tenoned into the post at 45 degrees");
        assert_eq!(scene.profiles.len(), 2);
        // Two legs, one design: the copy turned and put at its corner.
        let legs: Vec<&crate::Instance> = scene.root.instances().map(|(_, i)| i).filter(|i| scene.definition(i.definition).is_some_and(|d| d.name == "Leg")).collect();
        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].definition, legs[1].definition, "copies of one component");
        let lo = scene.instance_box_corners(legs[1]).into_iter().reduce(DVec3::min).unwrap();
        assert!(lo.abs_diff_eq(DVec3::new(20.0, 30.0, 0.0), 1e-9), "{lo}");
        let info = &scene.definition(legs[0].definition).unwrap().part;
        assert_eq!(info.cut, Cut::QuarterSawn);
        assert!(info.reference_face.is_some() && info.reference_edge.is_some());
        assert_eq!(info.grain, Some(PartAxis::X));
        // Mistakes in the new parts are named too.
        let bad = parse(r#"{ "parts": [{ "name": "A", "size": [10, 4, 1], "reference": { "face": "sideways" }, "turn": { "axis": "w", "degrees": 5 } }],
            "profiles": [{ "part": "A", "profile": "bead", "size": 0.25, "around": "top" }, { "part": "B", "profile": "cove", "size": 0.25 }] }"#).unwrap();
        let all = build(&bad).err().expect("errors").join("
");
        assert!(all.contains("turn axis \"w\"") && all.contains("reference face \"sideways\"") && all.contains("\"bead\" isn't a profile") && all.contains("no part named \"B\""), "{all}");
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
