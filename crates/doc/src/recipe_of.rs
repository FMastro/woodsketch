//! A drawing read back as a project recipe ([`crate::recipe`]): what an agent edits to change a
//! drawing made by hand, opened from SketchUp, or built from a recipe before. Building the recipe
//! gives the same parts, joints and profiles back.
//!
//! Parts come back at their size before any joints (a shelf let into a dado is its own length;
//! the joint lengthens it again), placed by their lowest corner, with a turn about one axis when
//! they're at an angle (or several, at a compound angle); shaped parts as an outline, or their faces; copies of one component as copies; templates with their sizes; groups by
//! their members. Tags come with their folders and whether
//! they're hidden, so hidden things come back hidden. What a recipe can't say is listed in the notes.

use std::collections::{HashMap, HashSet};

use glam::{DAffine3, DMat3, DVec3};

use crate::joinery::JointKind;
use crate::part::Cut;
use crate::profile::{Edges, Profile};
use crate::recipe::{CopySpec, GroupSpec, JointSpec, Length, MaterialSpec, OutlinePoint, PartSpec, ProfileSpec, Recipe, Reference, Shape, ShapeFace, TagSpec, TemplateSpec, Turn, Turns};
use crate::{DefinitionId, DefinitionKind, Document, InstanceId, MaterialId, MaterialKind, Scene};

/// The drawing as a recipe, and notes on what it leaves out.
pub fn from_document(doc: &Document) -> (Recipe, Vec<String>) {
    let scene = doc.scene();
    let mut out = Reader { scene, recipe: Recipe::default(), notes: Vec::new(), materials: HashMap::new(), names: HashSet::new(), parts: Vec::new(), tags: HashMap::new() };
    out.materials();
    out.tags();
    // Everything, hidden or not (its tags say so).
    let root: Vec<(InstanceId, DAffine3)> = scene.root.instances().map(|(id, i)| (id, i.transform)).collect();
    out.container(&scene.root, DAffine3::IDENTITY, &root);
    out.profiles();
    out.joints();
    // Once each (copies of one part say the same).
    let mut seen = HashSet::new();
    out.notes.retain(|n| seen.insert(n.clone()));
    (out.recipe, out.notes)
}

/// A part written to the recipe: its definition, where it is in the world, and its name there.
struct Placed {
    definition: DefinitionId,
    world: DAffine3,
    name: String,
}

struct Reader<'a> {
    scene: &'a Scene,
    recipe: Recipe,
    notes: Vec<String>,
    materials: HashMap<MaterialId, String>,
    names: HashSet<String>,
    parts: Vec<Placed>,
    /// Each tag's name in the recipe (made unique: SketchUp allows the same name in two folders).
    tags: HashMap<crate::TagId, String>,
}

impl Reader<'_> {
    /// A name not used yet: `Side`, then `Side 2`...
    fn unique(&mut self, name: &str) -> String {
        let base = if name.trim().is_empty() { "Part" } else { name.trim() };
        let mut name = base.to_owned();
        let mut n = 2;
        while self.names.contains(&name) {
            name = format!("{base} {n}");
            n += 1;
        }
        self.names.insert(name.clone());
        name
    }

    /// Every material, matched to the library by name where it is one.
    fn materials(&mut self) {
        let mut ids = HashSet::new();
        for (id, m) in self.scene.materials() {
            let base: String = m.name.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
            let mut key = base.trim_matches('_').to_owned();
            if key.is_empty() {
                key = "material".into();
            }
            let mut n = 2;
            let first = key.clone();
            while !ids.insert(key.clone()) {
                key = format!("{first}_{n}");
                n += 1;
            }
            let library = crate::library::LIBRARY.iter().find(|p| p.name == m.name && p.kind == m.kind).map(|p| p.name.to_owned());
            let kind = match m.kind {
                MaterialKind::SolidWood => "solid wood",
                MaterialKind::DimensionalLumber => "lumber",
                MaterialKind::SheetGood => "sheet",
                MaterialKind::Hardware => "hardware",
                MaterialKind::Other => "other",
            };
            let spec = match library {
                Some(name) => MaterialSpec { id: key.clone(), library: Some(name), ..Default::default() },
                None => MaterialSpec { id: key.clone(), library: None, name: Some(m.name.clone()), kind: Some(kind.into()), color: Some(m.color) },
            };
            self.recipe.materials.push(spec);
            self.materials.insert(id, key);
        }
    }

    /// The instances in a container (at `world`): templates, parts and groups, with copies of one
    /// part as copies. Returns the names given, for the group around them.
    fn container(&mut self, container: &crate::Container, world: DAffine3, instances: &[(InstanceId, DAffine3)]) -> Vec<String> {
        let scene = self.scene;
        let mut names = Vec::new();
        // Parts already written here, by definition: later ones are their copies.
        let mut first: HashMap<DefinitionId, (usize, DAffine3)> = HashMap::new();
        for &(id, transform) in instances {
            let Some(instance) = container.instance(id) else { continue };
            let Some(def) = scene.definition(instance.definition) else { continue };
            let placed = world * transform;
            if let Some(made) = &def.template {
                let name = self.unique(&def.name);
                let json = serde_json::to_value(made.template).unwrap_or_default();
                let sizes = json.as_object().and_then(|o| o.values().next()).and_then(|v| v.as_object()).cloned().unwrap_or_default();
                let lo = corners(scene, instance.definition, placed).into_iter().reduce(DVec3::min).unwrap_or(DVec3::ZERO);
                self.recipe.templates.push(TemplateSpec {
                    kind: made.template.name().to_lowercase().replace(' ', "_"),
                    name: Some(name.clone()),
                    at: length3(lo),
                    wood: made.wood.and_then(|w| self.materials.get(&w).cloned()),
                    sizes,
                    tag: self.tag_name(instance.tag),
                });
                names.push(name);
                continue;
            }
            if def.container.model.faces().next().is_none() {
                // A group of parts: its members, then the group.
                let inner: Vec<(InstanceId, DAffine3)> = def.container.instances().map(|(id, i)| (id, i.transform)).collect();
                let members = self.container(&def.container, placed, &inner);
                if !members.is_empty() {
                    let name = self.unique(&def.name);
                    let called = (name != def.name).then(|| def.name.clone());
                    self.recipe.groups.push(GroupSpec { name: name.clone(), parts: members, component: def.kind == DefinitionKind::Component, called, tag: self.tag_name(instance.tag), moves: def.moves });
                    names.push(name);
                }
                continue;
            }
            // Faces of its own and groups inside: its faces as a part, in a group with what's inside.
            if def.container.instances().next().is_some() {
                let inner: Vec<(InstanceId, DAffine3)> = def.container.instances().map(|(id, i)| (id, i.transform)).collect();
                let paint = instance.material.or_else(|| def.container.model.faces().next().and_then(|(_, f)| f.material));
                let mut members = Vec::new();
                if let Some(part) = self.part(instance.definition, placed, paint, None) {
                    members.push(part.name.clone());
                    self.parts.push(Placed { definition: instance.definition, world: placed, name: part.name.clone() });
                    self.recipe.parts.push(part);
                }
                members.extend(self.container(&def.container, placed, &inner));
                if !members.is_empty() {
                    let name = self.unique(&def.name);
                    let called = (name != def.name).then(|| def.name.clone());
                    self.recipe.groups.push(GroupSpec { name: name.clone(), parts: members, component: def.kind == DefinitionKind::Component, called, tag: self.tag_name(instance.tag), moves: def.moves });
                    names.push(name);
                }
                continue;
            }
            // A copy of a part already written, turned about z: written as its copy.
            if let Some(&(index, original)) = first.get(&instance.definition)
                && let Some(turn) = z_turn(original.matrix3, placed.matrix3)
            {
                let name = self.unique(&def.name);
                let lo = base_corners(scene, instance.definition, placed).into_iter().reduce(DVec3::min).unwrap_or(DVec3::ZERO);
                // Its own tag, if not the part's (`""`: none, where the part has one).
                let own = self.tag_name(instance.tag);
                let tag = (own != self.recipe.parts[index].tag).then(|| own.unwrap_or_default());
                self.recipe.parts[index].copies.push(CopySpec { name: name.clone(), at: length3(lo), turn, tag });
                self.parts.push(Placed { definition: instance.definition, world: placed, name: name.clone() });
                names.push(name);
                continue;
            }
            let paint = instance.material.or_else(|| def.container.model.faces().next().and_then(|(_, f)| f.material));
            if let Some(part) = self.part(instance.definition, placed, paint, instance.tag) {
                first.insert(instance.definition, (self.recipe.parts.len(), placed));
                names.push(part.name.clone());
                self.parts.push(Placed { definition: instance.definition, world: placed, name: part.name.clone() });
                self.recipe.parts.push(part);
            }
        }
        names
    }

    /// One part: its size before joints, the axes it lies along (and a turn), its details.
    fn part(&mut self, definition: DefinitionId, world: DAffine3, paint: Option<MaterialId>, tag: Option<crate::TagId>) -> Option<PartSpec> {
        let scene = self.scene;
        let def = scene.definition(definition)?;
        // Before its joints changed it.
        // (Joints at an angle keep the shapes from before them on the joint.)
        let angled = scene.joints.iter().find_map(|j| j.before.iter().find(|(d, _)| *d == definition).map(|(_, m)| m));
        let model = angled.or_else(|| scene.joint_bases.iter().find(|(d, _)| *d == definition).map(|(_, m)| m)).unwrap_or(&def.container.model);
        let (lo, hi) = model.bounds()?;
        let scale = DVec3::new(world.matrix3.x_axis.length(), world.matrix3.y_axis.length(), world.matrix3.z_axis.length());
        let extents = (hi - lo) * scale;
        let [l, w, t] = crate::part::axes(extents, &def.part);
        if extents.min_element() < 1.0 / 64.0 {
            self.notes.push(format!("{}: flat (a picture or cut-out figure), left out.", def.name));
            return None;
        }
        // A recipe part is a box; one shaped otherwise (angled ends, curves, cuts made by hand)
        // keeps its own faces as its shape. (A profile the recipe makes again doesn't count.)
        let boxy = model.faces().count() == 6 && model.faces().all(|(_, f)| f.normal.abs().max_element() > 1.0 - 1e-6);
        let shaped = !boxy && !scene.profiles.iter().any(|p| p.part == definition);
        let rotation = DMat3::from_cols(world.matrix3.x_axis / scale.x, world.matrix3.y_axis / scale.y, world.matrix3.z_axis / scale.z);
        let (along, across) = (rotation * DVec3::AXES[l], rotation * DVec3::AXES[w]);
        let name = self.unique(&def.name);
        // As the recipe builds it: along two axes, then turned about one; or, at a compound
        // angle, laid along x and y and turned about x, then y, then z.
        let (length_axis, width_axis, turn, spin) = match lay(along, across) {
            Some((a, b, turn)) => {
                let spin = turn.as_ref().map_or(DMat3::IDENTITY, |t| DMat3::from_axis_angle(DVec3::AXES[axis_index(&t.axis)], t.degrees.to_radians()));
                (a, b, turn.map(Turns::One), spin)
            }
            None => {
                let laid = DMat3::from_cols(along, across, along.cross(across));
                let (z, y, x) = glam::DQuat::from_mat3(&laid).to_euler(glam::EulerRot::ZYX);
                let turns: Vec<Turn> = [("x", x), ("y", y), ("z", z)]
                    .into_iter()
                    .map(|(axis, angle)| Turn { axis: axis.into(), degrees: (angle.to_degrees() * 1e6).round() / 1e6 })
                    .filter(|t| t.degrees != 0.0)
                    .collect();
                let spin = turns.iter().fold(DMat3::IDENTITY, |m, t| DMat3::from_axis_angle(DVec3::AXES[axis_index(&t.axis)], t.degrees.to_radians()) * m);
                (0, 1, Some(Turns::Several(turns)), spin)
            }
        };
        // Its lowest corner before the turn, turned (the turn is about that corner).
        let box_corners: Vec<DVec3> = (0..8)
            .map(|i| {
                let c = DVec3::new(if i & 1 == 0 { lo.x } else { hi.x }, if i & 2 == 0 { lo.y } else { hi.y }, if i & 4 == 0 { lo.z } else { hi.z });
                world.transform_point3(c)
            })
            .collect();
        let before = box_corners.iter().map(|c| spin.transpose() * *c).reduce(DVec3::min).unwrap_or(DVec3::ZERO);
        let at = spin * before;
        let size = [extents[l], extents[w], extents[t]];
        // The shape, in the frame the board will be built in: the box along its axes from `at`,
        // turned. An outline cut through its thickness when that's all it is, else its faces.
        let (mut shape, mut outline, mut cutouts) = (None, Vec::new(), Vec::new());
        if shaped {
            let (ld, wd) = (DVec3::AXES[length_axis], DVec3::AXES[width_axis]);
            let td = ld.cross(wd);
            let origin = if td.min_element() < 0.0 { at - td * size[2] } else { at };
            let board = crate::board::Board { origin, length_dir: ld, width_dir: wd, length: size[0], width: size[1], thickness: size[2] };
            let turned = DAffine3::from_translation(at) * DAffine3::from_mat3(spin) * DAffine3::from_translation(-at);
            let to_local = (turned * board.transform()).inverse() * world;
            // A mirrored placement turns faces inside out: wind them back.
            let flip = to_local.matrix3.determinant() < 0.0;
            let ring = |ids: &[crate::VertexId]| -> Vec<[f64; 3]> {
                let mut points: Vec<[f64; 3]> = ids.iter().map(|&v| to_local.transform_point3(model.vertex(v)).to_array().map(|c| (c * 100_000.0).round() / 100_000.0)).collect();
                if flip {
                    points.reverse();
                }
                points
            };
            let faces: Vec<ShapeFace> = model.faces().map(|(_, f)| ShapeFace { outer: ring(&f.outer), holes: f.holes.iter().map(|h| ring(h)).collect() }).collect();
            // (An outline only if it builds back: a loop that touches itself doesn't.)
            match extruded(&faces, size[2]).filter(|(o, c)| crate::recipe::outline_model(o, c, 1.0, size).is_ok()) {
                Some((o, c)) => (outline, cutouts) = (o, c),
                None => shape = Some(Shape { faces }),
            }
        }
        let info = &def.part;
        let side_name = |side: Option<crate::part::Side>| side.and_then(|s| direction_name(rotation * s.normal()));
        let reference = Reference { face: side_name(info.reference_face), edge: side_name(info.reference_edge), end: side_name(info.reference_end) };
        let reference = (reference != Reference::default()).then_some(reference);
        Some(PartSpec {
            called: (name != def.name).then(|| def.name.clone()),
            name,
            material: paint.and_then(|m| self.materials.get(&m).cloned()),
            size: size.map(|v| Length::Number(round(v))),
            at: length3(at),
            length: ["x", "y", "z"][length_axis].into(),
            width: ["x", "y", "z"][width_axis].into(),
            tag: tag.and_then(|t| scene.tag(t)).map(|t| t.name.clone()),
            free_to_move: info.free_to_move,
            grain: info.grain.filter(|g| g.index() == w).map(|_| "width".to_owned()),
            cut: match info.cut {
                Cut::FlatSawn => Some("flat_sawn".into()),
                Cut::QuarterSawn => Some("quartersawn".into()),
                Cut::Unknown => None,
            },
            turn,
            reference,
            copies: Vec::new(),
            hardware: def.hardware.clone(),
            shape,
            outline,
            cutouts,
        })
    }

    /// Every tag and folder, hidden or not.
    fn tags(&mut self) {
        let scene = self.scene;
        let mut used = HashSet::new();
        for (id, tag) in scene.tags() {
            let base = if tag.name.trim().is_empty() { "Tag" } else { tag.name.as_str() };
            let mut name = base.to_owned();
            let mut n = 2;
            while !used.insert(name.clone()) {
                name = format!("{base} {n}");
                n += 1;
            }
            self.tags.insert(id, name);
        }
        for (id, tag) in scene.tags() {
            let inside = tag.parent.and_then(|p| self.tags.get(&p)).cloned();
            self.recipe.tags.push(TagSpec { name: self.tags[&id].clone(), hidden: !tag.visible, folder: tag.folder, inside });
        }
    }

    fn tag_name(&self, tag: Option<crate::TagId>) -> Option<String> {
        tag.and_then(|t| self.tags.get(&t)).cloned()
    }

    /// Profiles on the parts written, by the face they go around or the way their edges run.
    fn profiles(&mut self) {
        for record in &self.scene.profiles {
            let Some(placed) = self.parts.iter().find(|p| p.definition == record.part) else { continue };
            let def = self.scene.definition(placed.definition);
            let profile = match record.profile {
                Profile::Roundover => "roundover",
                Profile::Chamfer => "chamfer",
                Profile::Cove => "cove",
                Profile::Ogee => "ogee",
            };
            // Its own directions by where they face once placed (the nearest, as the recipe reads them).
            let facing = |axis: usize, positive: bool| {
                let mut normal = DVec3::ZERO;
                normal[axis] = if positive { 1.0 } else { -1.0 };
                nearest_name(placed.world.matrix3 * normal)
            };
            let (around, along, edges) = match record.edges {
                Edges::AroundFace { axis, positive, sides } => {
                    let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                    // Sides: u-, u+, v-, v+.
                    let named = [(u, false), (u, true), (v, false), (v, true)].into_iter().zip(sides).filter(|(_, on)| *on).map(|((a, p), _)| facing(a, p).to_owned());
                    let edges = if sides.iter().all(|s| *s) { Vec::new() } else { named.collect() };
                    (Some(facing(axis, positive).to_owned()), None, edges)
                }
                Edges::Along { axis, corners } => {
                    // Along its length, width or thickness: the part's own axes.
                    let extents = def.and_then(|d| crate::part::extents(d, placed.world.matrix3));
                    let axes = match (def, extents) {
                        (Some(d), Some(e)) => crate::part::axes(e, &d.part),
                        _ => [0, 1, 2],
                    };
                    let way = axes.iter().position(|a| *a == axis).map(|k| ["length", "width", "thickness"][k].to_owned());
                    let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                    // Corners: (u-, v-), (u+, v-), (u+, v+), (u-, v+).
                    let named = [(false, false), (true, false), (true, true), (false, true)].into_iter().zip(corners).filter(|(_, on)| *on).map(|((pu, pv), _)| format!("{} {}", facing(u, pu), facing(v, pv)));
                    let edges = if corners.iter().all(|c| *c) { Vec::new() } else { named.collect() };
                    (None, way, edges)
                }
            };
            self.recipe.profiles.push(ProfileSpec { part: placed.name.clone(), profile: profile.into(), size: Length::Number(round(record.size)), around, along, edges });
        }
    }

    /// Joints, between parts written that meet, with the sizes they were made with.
    fn joints(&mut self) {
        let mut used: HashSet<(usize, usize, usize)> = HashSet::new();
        for (index, joint) in self.scene.joints.iter().enumerate() {
            let boxes = |d: DefinitionId| -> Vec<(usize, (DVec3, DVec3))> {
                self.parts
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.definition == d)
                    .map(|(i, p)| (i, corners(self.scene, p.definition, p.world).into_iter().fold((DVec3::INFINITY, DVec3::NEG_INFINITY), |(lo, hi), c| (lo.min(c), hi.max(c)))))
                    .collect()
            };
            let (cuts, intos) = (boxes(joint.cut), boxes(joint.into));
            // A pair that meets (boxes touching or overlapping), not given this kind of joint yet.
            let pair = cuts.iter().flat_map(|a| intos.iter().map(move |b| (a, b))).find(|((i, (alo, ahi)), (j, (blo, bhi)))| {
                i != j && (*alo - *bhi).max_element() <= 1e-3 && (*blo - *ahi).max_element() <= 1e-3 && !used.contains(&(*i, *j, joint.kind as usize))
            });
            let Some(((i, _), (j, _))) = pair else {
                self.notes.push(format!("joints[{index}] ({}): its parts aren't both written, or don't meet.", joint.kind.name()));
                continue;
            };
            used.insert((*i, *j, joint.kind as usize));
            let s = &joint.sizes;
            let size = |k: usize| s.get(k).copied();
            let mut options = serde_json::Map::new();
            let angled = !joint.before.is_empty();
            // Cut into every copy of a shared component, not only the one named.
            if angled && size(4).is_some_and(|v| v > 0.5) {
                options.insert("every_copy".into(), serde_json::json!(true));
            }
            let mut put = |key: &str, value: Option<f64>| {
                if let Some(v) = value {
                    options.insert(key.into(), serde_json::json!(round(v)));
                }
            };
            let kind = match joint.kind {
                JointKind::MortiseAndTenon | JointKind::Notch if angled && size(2).is_some_and(|h| h > 0.5) => "housing",
                JointKind::MortiseAndTenon if angled => {
                    put("thickness", size(0));
                    put("shoulder", size(1));
                    put("fit", size(3).filter(|v| *v > 0.0));
                    "angled_mortise_and_tenon"
                }
                JointKind::Dado | JointKind::Groove | JointKind::Rabbet => {
                    put("depth", size(0));
                    "let_in"
                }
                JointKind::Notch | JointKind::Mortise => "cut_overlap",
                JointKind::HalfLap => "half_lap",
                JointKind::MortiseAndTenon => {
                    put("length", size(0));
                    put("thickness", size(1));
                    put("shoulder", size(2));
                    put("fit", size(3).filter(|v| *v > 0.0));
                    "mortise_and_tenon"
                }
                JointKind::Dowels => {
                    put("count", size(0));
                    put("diameter", size(1));
                    put("depth", size(2));
                    put("fit", size(3).filter(|v| *v > 0.0));
                    "dowels"
                }
                JointKind::BoxJoint => {
                    put("finger", size(0));
                    "box_joint"
                }
                JointKind::Biscuits => {
                    put("count", size(1));
                    put("fit", size(2).filter(|v| *v > 0.0));
                    "biscuits"
                }
                JointKind::LooseTenons => {
                    put("count", size(1));
                    put("fit", size(2).filter(|v| *v > 0.0));
                    "loose_tenons"
                }
                JointKind::PocketScrews => {
                    put("count", size(0));
                    put("screw", size(1));
                    "pocket_screws"
                }
                JointKind::Dovetails => {
                    put("tails", size(0));
                    put("pin", size(2).filter(|v| (v - crate::joinery::DOVETAIL_PIN).abs() > 1e-9));
                    put("half_pin", size(3).filter(|v| (v - crate::joinery::DOVETAIL_HALF_PIN).abs() > 1e-9));
                    if size(1).is_some_and(|v| v > 0.5) {
                        options.insert("half_blind".into(), serde_json::json!(true));
                    }
                    "dovetails"
                }
                JointKind::Splines => "splines",
            };
            self.recipe.joints.push(JointSpec { kind: kind.into(), part: self.parts[*i].name.clone(), into: self.parts[*j].name.clone(), options });
        }
    }
}

/// The world corners of a part's box before its joints changed it, placed by `world`.
fn base_corners(scene: &Scene, definition: DefinitionId, world: DAffine3) -> Vec<DVec3> {
    let angled = scene.joints.iter().find_map(|j| j.before.iter().find(|(d, _)| *d == definition).map(|(_, m)| m));
    let base = angled.or_else(|| scene.joint_bases.iter().find(|(d, _)| *d == definition).map(|(_, m)| m)).and_then(|m| m.bounds());
    let Some((lo, hi)) = base.or_else(|| scene.definition_bounds(definition)) else { return Vec::new() };
    (0..8)
        .map(|i| world.transform_point3(DVec3::new(if i & 1 == 0 { lo.x } else { hi.x }, if i & 2 == 0 { lo.y } else { hi.y }, if i & 4 == 0 { lo.z } else { hi.z })))
        .collect()
}

/// The world corners of a definition's box placed by `world`.
fn corners(scene: &Scene, definition: DefinitionId, world: DAffine3) -> Vec<DVec3> {
    let Some((lo, hi)) = scene.definition_bounds(definition) else { return Vec::new() };
    (0..8)
        .map(|i| world.transform_point3(DVec3::new(if i & 1 == 0 { lo.x } else { hi.x }, if i & 2 == 0 { lo.y } else { hi.y }, if i & 4 == 0 { lo.z } else { hi.z })))
        .collect()
}

/// How a box whose length runs `along` and width `across` (unit) is laid out in a recipe: the axes
/// its length and width take, and a turn about one axis (none when square to them). A box is the
/// same either way along each axis, so either sign does.
/// A shape (in its board's frame) that's an outline cut straight through its thickness: one face
/// at z = 0, one the same at z = `thickness`, and only sides between. Its outline and holes.
fn extruded(faces: &[ShapeFace], thickness: f64) -> Option<(Vec<OutlinePoint>, Vec<Vec<OutlinePoint>>)> {
    const EPS: f64 = 1e-4;
    let at = |z: f64, face: &ShapeFace| face.outer.iter().chain(face.holes.iter().flatten()).all(|p| (p[2] - z).abs() < EPS);
    if faces.iter().flat_map(|f| f.outer.iter().chain(f.holes.iter().flatten())).any(|p| p[2].abs() > EPS && (p[2] - thickness).abs() > EPS) {
        return None;
    }
    let bottoms: Vec<&ShapeFace> = faces.iter().filter(|f| at(0.0, f)).collect();
    let tops: Vec<&ShapeFace> = faces.iter().filter(|f| at(thickness, f)).collect();
    let ([bottom], [top]) = (bottoms.as_slice(), tops.as_slice()) else { return None };
    let plan = |face: &ShapeFace| {
        let mut points: Vec<[i64; 2]> = face.outer.iter().chain(face.holes.iter().flatten()).map(|p| [p[0], p[1]].map(|c| (c * 10_000.0).round() as i64)).collect();
        points.sort_unstable();
        points
    };
    if plan(bottom) != plan(top) {
        return None;
    }
    let flat = |ring: &[[f64; 3]]| ring.iter().map(|p| OutlinePoint::Point([p[0], p[1]])).collect::<Vec<_>>();
    Some((flat(&bottom.outer), bottom.holes.iter().map(|h| flat(h)).collect()))
}

fn lay(along: DVec3, across: DVec3) -> Option<(usize, usize, Option<Turn>)> {
    const EPS: f64 = 1e-6;
    let mut best: Option<(f64, usize, usize, Option<Turn>)> = None;
    for k in 0..3 {
        for a in 0..3 {
            for b in (0..3).filter(|b| *b != a) {
                for (sa, sb) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                    let (l, w) = (along * sa, across * sb);
                    let (ea, eb) = (DVec3::AXES[a], DVec3::AXES[b]);
                    // A turn about axis k keeps each direction's k part.
                    if (l[k] - ea[k]).abs() > EPS || (w[k] - eb[k]).abs() > EPS {
                        continue;
                    }
                    // The angle that takes ea to l about k (or eb to w, if ea is along k).
                    let axis = DVec3::AXES[k];
                    let (from, to) = if a == k { (eb, w) } else { (ea, l) };
                    let angle = axis.dot(from.cross(to)).atan2(from.dot(to));
                    let spin = DMat3::from_axis_angle(axis, angle);
                    if (spin * ea - l).length() > EPS || (spin * eb - w).length() > EPS {
                        continue;
                    }
                    let degrees = round(angle.to_degrees());
                    let turn = (degrees.abs() > 1e-6).then(|| Turn { axis: ["x", "y", "z"][k].into(), degrees });
                    if best.as_ref().is_none_or(|(d, ..)| degrees.abs() < *d) {
                        best = Some((degrees.abs(), a, b, turn));
                    }
                }
            }
        }
    }
    best.map(|(_, a, b, turn)| (a, b, turn))
}

/// The turn about z (degrees) that takes a part placed by `original` to `copy`, if that's all it is.
fn z_turn(original: DMat3, copy: DMat3) -> Option<f64> {
    let relative = copy * original.inverse();
    let angle = relative.x_axis.y.atan2(relative.x_axis.x);
    let spin = DMat3::from_rotation_z(angle);
    ((relative - spin).to_cols_array().iter().all(|v| v.abs() < 1e-6)).then(|| round(angle.to_degrees()))
}

fn axis_index(name: &str) -> usize {
    match name {
        "y" => 1,
        "z" => 2,
        _ => 0,
    }
}

/// `top`, `front`... for a direction square to the axes.
/// The side a direction faces most: `top`, `front`...
fn nearest_name(d: DVec3) -> &'static str {
    let named = [(DVec3::Z, "top"), (-DVec3::Z, "bottom"), (-DVec3::Y, "front"), (DVec3::Y, "back"), (-DVec3::X, "left"), (DVec3::X, "right")];
    named.iter().max_by(|a, b| a.0.dot(d).total_cmp(&b.0.dot(d))).map_or("top", |(_, name)| name)
}

fn direction_name(d: DVec3) -> Option<String> {
    let named = [(DVec3::Z, "top"), (-DVec3::Z, "bottom"), (-DVec3::Y, "front"), (DVec3::Y, "back"), (-DVec3::X, "left"), (DVec3::X, "right")];
    named.iter().find(|(n, _)| n.dot(d) > 1.0 - 1e-6).map(|(_, name)| (*name).to_owned())
}

/// Lengths kept to 1/10000" (no float dust in the recipe).
fn round(v: f64) -> f64 {
    let r = (v * 10000.0).round() / 10000.0;
    if r == 0.0 { 0.0 } else { r }
}

fn length3(p: DVec3) -> [Length; 3] {
    [p.x, p.y, p.z].map(|v| Length::Number(round(v)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::{build, parse};

    /// What a drawing makes: its parts (sorted) and joints and profiles counted.
    fn made(doc: &Document) -> (String, usize, usize) {
        let mut text = crate::recipe::summary(doc);
        text = text.lines().filter(|l| !l.starts_with("Overall")).collect::<Vec<_>>().join("\n");
        (text, doc.scene().joints.len(), doc.scene().profiles.len())
    }

    fn round_trip(json: &str) -> (Recipe, Vec<String>) {
        let doc = build(&parse(json).unwrap()).unwrap_or_else(|e| panic!("{e:?}"));
        let (recipe, notes) = from_document(&doc);
        let again = build(&recipe).unwrap_or_else(|e| panic!("{e:?}\n{}", serde_json::to_string_pretty(&recipe).unwrap()));
        assert_eq!(made(&again), made(&doc), "the same drawing again:\n{}", serde_json::to_string_pretty(&recipe).unwrap());
        assert_eq!(doc.scene().bounds(), again.scene().bounds());
        (recipe, notes)
    }

    #[test]
    fn a_built_drawing_reads_back_as_a_recipe_that_builds_it_again() {
        let (recipe, notes) = round_trip(
            r#"{
            "materials": [{ "id": "oak", "library": "Red Oak" }, { "id": "ply", "name": "Shop ply", "kind": "sheet", "color": [200, 180, 140] }],
            "parts": [
                { "name": "Side", "material": "oak", "size": [30, 11.25, 0.75], "length": "z", "width": "y", "cut": "quartersawn",
                  "reference": { "face": "left", "edge": "front" } },
                { "name": "Side 2", "material": "oak", "size": [30, 11.25, 0.75], "at": [20.75, 0, 0], "length": "z", "width": "y" },
                { "name": "Shelf", "material": "oak", "size": [20, 11.25, 0.75], "at": [0.75, 0, 12] },
                { "name": "Back", "material": "ply", "size": [20, 30, 0.25], "at": [0.75, 11.25, 0], "length": "x", "width": "z" }
            ],
            "profiles": [{ "part": "Shelf", "profile": "roundover", "size": 0.25, "around": "top" }],
            "joints": [
                { "kind": "let_in", "part": "Shelf", "into": "Side", "options": { "depth": 0.375 } },
                { "kind": "dado", "part": "Shelf", "into": "Side 2" }
            ],
            "groups": [{ "name": "Bookcase", "parts": ["Side", "Side 2", "Shelf", "Back"] }]
        }"#,
        );
        assert!(notes.is_empty(), "{notes:?}");
        assert_eq!(recipe.materials.iter().filter(|m| m.library.is_some()).count(), 1, "Red Oak from the library");
        let shelf = recipe.parts.iter().find(|p| p.name == "Shelf").unwrap();
        assert_eq!(shelf.size[0], Length::Number(20.0), "its size before the dados lengthened it");
        assert_eq!(recipe.joints[0].options.get("depth"), Some(&serde_json::json!(0.375)), "the depth it was made with");
        assert_eq!(recipe.groups.len(), 1);
    }

    #[test]
    fn copies_turns_and_templates_read_back() {
        let (recipe, _) = round_trip(
            r#"{
            "materials": [{ "id": "oak", "library": "Red Oak" }, { "id": "cherry", "library": "Cherry" }],
            "parts": [
                { "name": "Post", "material": "oak", "size": [48, 3.5, 1.5], "length": "z", "width": "x" },
                { "name": "Brace", "material": "oak", "size": [20, 3.5, 1.5], "at": [2.5, 0, 20], "length": "x", "width": "z",
                  "turn": { "axis": "y", "degrees": -45 } },
                { "name": "Leg", "material": "oak", "size": [28, 1.5, 1.5], "at": [0, 30, 0], "length": "z", "width": "x",
                  "copies": [{ "name": "Leg 2", "at": [20, 30, 0], "turn": 90 }] }
            ],
            "templates": [{ "kind": "table", "name": "Side table", "at": [60, 0, 0], "wood": "cherry", "sizes": { "width": 30 } }],
            "joints": [{ "kind": "angled_mortise_and_tenon", "part": "Brace", "into": "Post" }]
        }"#,
        );
        let brace = recipe.parts.iter().find(|p| p.name == "Brace").unwrap();
        assert_eq!(brace.turn.as_ref().map(|t| (t.list()[0].axis.as_str(), t.list()[0].degrees.abs())), Some(("y", 45.0)), "{:?}", brace.turn);
        let leg = recipe.parts.iter().find(|p| p.name == "Leg").unwrap();
        assert_eq!(leg.copies.len(), 1, "the second leg is its copy");
        assert_eq!(recipe.templates.len(), 1);
        assert_eq!(recipe.templates[0].sizes.get("width"), Some(&serde_json::json!(30.0)));
        assert_eq!(recipe.joints[0].kind, "angled_mortise_and_tenon");
    }

    #[test]
    fn an_angled_joint_on_every_copy_reads_back() {
        let json = r#"{
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "parts": [
                { "name": "Post", "material": "oak", "size": [48, 3.5, 1.5], "length": "z", "width": "x",
                  "copies": [{ "name": "Post 2", "at": [40, 0, 0] }] },
                { "name": "Brace", "material": "oak", "size": [20, 3.5, 1.5], "at": [2.5, 0, 20], "length": "x", "width": "z",
                  "turn": { "axis": "y", "degrees": -45 } }
            ],
            "joints": [{ "kind": "housing", "part": "Brace", "into": "Post", "options": { "every_copy": true } }]
        }"#;
        let (recipe, _) = round_trip(json);
        let doc = build(&parse(json).unwrap()).unwrap();
        let scene = doc.scene();
        let posts: Vec<_> = scene.root.instances().map(|(_, i)| i.definition).filter(|d| scene.definition(*d).is_some_and(|d| d.name == "Post")).collect();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0], posts[1], "both posts are still one component");
        let post = recipe.parts.iter().find(|p| p.name == "Post").unwrap();
        assert_eq!(post.copies.len(), 1, "read back as a copy");
        assert_eq!(recipe.joints[0].kind, "housing");
        assert_eq!(recipe.joints[0].options.get("every_copy"), Some(&serde_json::json!(true)));
    }

    /// A 10" x 2" x 1" board with one end cut at an angle (the top edge 8" long), as a shape.
    fn angled_end() -> serde_json::Value {
        let p = |x: f64, y: f64, z: f64| serde_json::json!([x, y, z]);
        let (a, b, c, d) = (|z| p(0.0, 0.0, z), |z| p(10.0, 0.0, z), |z| p(8.0, 2.0, z), |z| p(0.0, 2.0, z));
        serde_json::json!({ "faces": [
            { "outer": [a(0.0), d(0.0), c(0.0), b(0.0)] },
            { "outer": [a(1.0), b(1.0), c(1.0), d(1.0)] },
            { "outer": [a(0.0), b(0.0), b(1.0), a(1.0)] },
            { "outer": [b(0.0), c(0.0), c(1.0), b(1.0)] },
            { "outer": [c(0.0), d(0.0), d(1.0), c(1.0)] },
            { "outer": [d(0.0), a(0.0), a(1.0), d(1.0)] }
        ] })
    }

    /// Every corner of every part, in the world, sorted: two drawings the same shape and place.
    fn corners_of(doc: &Document) -> Vec<[i64; 3]> {
        let scene = doc.scene();
        let mut out: Vec<[i64; 3]> = scene
            .root
            .instances()
            .flat_map(|(_, i)| {
                let model = &scene.definition(i.definition).unwrap().container.model;
                model.vertices().map(|(_, p)| i.transform.transform_point3(p).to_array().map(|c| (c * 1000.0).round() as i64)).collect::<Vec<_>>()
            })
            .collect();
        out.sort_unstable();
        out
    }

    #[test]
    fn shaped_parts_keep_their_shape_through_a_read_back() {
        let shape = angled_end();
        let recipe = serde_json::json!({
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "parts": [
                { "name": "Bracket", "material": "oak", "size": [10, 2, 1], "at": [5, 5, 0], "shape": shape },
                { "name": "Brace", "material": "oak", "size": [10, 2, 1], "at": [20, 0, 0], "length": "x", "width": "z",
                  "turn": { "axis": "y", "degrees": -30 }, "shape": shape }
            ]
        });
        let doc = build(&parse(&recipe.to_string()).unwrap()).unwrap_or_else(|e| panic!("{e:?}"));
        let (read, notes) = from_document(&doc);
        assert!(notes.is_empty(), "{notes:?}");
        assert!(read.parts.iter().all(|p| p.shape.is_none() && p.outline.len() == 4), "both read back as outlines: {:?}", read.parts[0]);
        let again = build(&read).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(corners_of(&again), corners_of(&doc), "the same shapes in the same places");
        // And after moving one, the other is untouched.
        let mut moved = read.clone();
        moved.change_part("Bracket", &serde_json::json!({ "at": [5, 5, 3] })).unwrap();
        let after = build(&moved).unwrap();
        let brace = |d: &Document| corners_of(d).into_iter().filter(|c| c[0] >= 18_000).collect::<Vec<_>>();
        assert_eq!(brace(&after), brace(&doc));
    }

    #[test]
    fn a_part_turned_about_two_axes_is_kept_where_it_is() {
        let mut doc = Document::default();
        let oak = doc.add_preset(crate::library::LIBRARY.iter().find(|p| p.name == "Red Oak").unwrap());
        let length_dir = DVec3::new(1.0, 1.0, 1.0).normalize();
        let width_dir = DVec3::new(-1.0, 1.0, 0.0).normalize();
        doc.add_board(&crate::board::Board { origin: DVec3::new(3.0, 4.0, 5.0), length_dir, width_dir, length: 20.0, width: 4.0, thickness: 1.0 }, Some(oak)).unwrap();
        let (read, notes) = from_document(&doc);
        assert!(notes.is_empty(), "{notes:?}");
        let part = &read.parts[0];
        assert!(part.shape.is_none() && part.turn.as_ref().is_some_and(|t| t.list().len() > 1), "a box at a compound angle: {part:?}");
        assert_eq!(part.size.clone().map(|l| match l { Length::Number(n) => n, Length::Text(_) => 0.0 }), [20.0, 4.0, 1.0], "its own size");
        let again = build(&read).unwrap_or_else(|e| panic!("{e:?}"));
        let (a, b) = (corners_of(&again), corners_of(&doc));
        assert!(a.len() == b.len() && a.iter().zip(&b).all(|(p, q)| (0..3).all(|k| (p[k] - q[k]).abs() <= 1)), "in the same place: {a:?} {b:?}");
    }

    #[test]
    fn tags_hardware_and_profiles_on_some_edges_read_back() {
        let recipe = serde_json::json!({
            "materials": [{ "id": "oak", "library": "Red Oak" }],
            "tags": [
                { "name": "Case", "folder": true },
                { "name": "Sides", "inside": "Case" },
                { "name": "Jigs", "hidden": true }
            ],
            "parts": [
                { "name": "Side", "material": "oak", "size": [30, 12, 0.75], "length": "z", "width": "y", "tag": "Sides",
                  "copies": [{ "name": "Spare side", "at": [60, 0, 0], "tag": "Jigs" }] },
                { "name": "Top", "material": "oak", "size": [20, 12, 0.75], "at": [0, 0, 30] },
                { "name": "Stand", "material": "oak", "size": [10, 4, 0.75], "at": [40, 0, 0] },
                { "name": "Pull", "size": [4, 0.5, 0.5], "at": [5, -1, 31], "hardware": "Bar pull" }
            ],
            "profiles": [
                { "part": "Top", "profile": "roundover", "size": 0.25, "around": "top", "edges": ["front", "left"] },
                { "part": "Side", "profile": "chamfer", "size": 0.125, "along": "length", "edges": ["front right"] }
            ],
            "groups": [{ "name": "Jig", "parts": ["Stand"], "tag": "Jigs", "moves": true }]
        });
        let doc = build(&parse(&recipe.to_string()).unwrap()).unwrap_or_else(|e| panic!("{e:?}"));
        let (read, notes) = from_document(&doc);
        assert!(notes.is_empty(), "{notes:?}");
        assert_eq!(read.tags.len(), 3, "{:?}", read.tags);
        assert!(read.tags.iter().any(|t| t.name == "Sides" && t.inside.as_deref() == Some("Case")));
        assert!(read.tags.iter().any(|t| t.name == "Jigs" && t.hidden));
        assert!(read.groups.iter().any(|g| g.name == "Jig" && g.tag.as_deref() == Some("Jigs") && g.moves), "the hidden jig comes back");
        assert!(read.parts.iter().any(|p| p.hardware.as_deref() == Some("Bar pull")));
        let side = read.parts.iter().find(|p| p.name == "Side").unwrap();
        assert_eq!(side.copies[0].tag.as_deref(), Some("Jigs"), "the copy keeps its own tag");
        let mut edges: Vec<Vec<String>> = read.profiles.iter().map(|p| p.edges.clone()).collect();
        edges.sort();
        assert_eq!(edges, [vec!["front right".to_owned()], vec!["left".to_owned(), "front".to_owned()]]);
        let again = build(&read).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(again.scene().profiles.iter().map(|p| p.edges).collect::<Vec<_>>(), doc.scene().profiles.iter().map(|p| p.edges).collect::<Vec<_>>());
        assert_eq!(crate::recipe::summary(&again), crate::recipe::summary(&doc));
        let (twice, _) = from_document(&again);
        assert_eq!(serde_json::to_value(&twice).unwrap(), serde_json::to_value(&read).unwrap(), "reads back the same again");
    }

    #[test]
    fn a_group_with_faces_of_its_own_keeps_what_is_inside() {
        // A bed: a frame drawn right in its group, with a mattress group inside.
        let mut doc = build(&parse(r#"{ "parts": [{ "name": "Mattress", "size": [75, 39, 8], "at": [0, 0, 10] }] }"#).unwrap()).unwrap();
        let mattress = doc.scene().root.instances().next().unwrap().0;
        doc.selection_mut().extend([crate::Entity::Instance(mattress)]);
        let bed = doc.group_selection(DefinitionKind::Group).unwrap();
        doc.rename_instance(bed, "Bed");
        doc.enter(bed);
        doc.edit(|m| {
            let face = m.add_rectangle(DVec3::ZERO, DVec3::X * 76.0, DVec3::Y * 40.0, DVec3::Z).unwrap();
            m.push_pull(face, 2.0).unwrap();
        });
        doc.exit();
        let (read, _) = from_document(&doc);
        assert_eq!(read.groups.len(), 1);
        assert_eq!(read.groups[0].parts.len(), 2, "its own faces and the mattress: {:?}", read.groups[0]);
        let again = build(&read).unwrap_or_else(|e| panic!("{e:?}"));
        assert!(crate::recipe::summary(&again).contains("Mattress"), "the mattress is still there");
    }

    #[test]
    fn laying_out_a_turned_box() {
        // Length along +x turned 45 degrees about y (down toward -z is -45 about y here).
        let d = std::f64::consts::FRAC_1_SQRT_2;
        let (a, b, turn) = lay(DVec3::new(d, 0.0, d), DVec3::new(-d, 0.0, d)).unwrap();
        assert_eq!((a, b), (0, 2));
        assert_eq!(turn.map(|t| (t.axis, t.degrees.abs())), Some(("y".to_owned(), 45.0)));
        assert!(lay(DVec3::Z, DVec3::X).unwrap().2.is_none(), "square: no turn");
        assert_eq!(z_turn(DMat3::IDENTITY, DMat3::from_rotation_z(std::f64::consts::FRAC_PI_2)), Some(90.0));
        assert_eq!(z_turn(DMat3::IDENTITY, DMat3::from_rotation_x(0.3)), None);
    }

}
