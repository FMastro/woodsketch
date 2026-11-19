//! A component that remembers how it was made: the recipe it was built from, with the numbers
//! that recipe was given. Changing one of those numbers builds the piece again — a cabinet whose
//! width goes from 600 to 900 gets a wider back and a longer shelf, rather than being stretched.
//!
//! A template already works this way ([`crate::template::Made`] and `rebuild_template`), with its
//! parameters written in Rust. This is the same idea with the parameters as data, so a drawing can
//! hold a kind of component nobody compiled in (#9).
//!
//! Two rules it keeps, both decided before it was written:
//!
//! - **Resizing one copy makes it its own component.** Five cabinets from one design, and widening
//!   the third would otherwise silently widen the other four — and the cut list has to tell them
//!   apart anyway.
//! - **A component somebody has edited by hand refuses to rebuild** until told twice, because
//!   building it again throws that work away. Templates discard it; twenty minutes of hand-fitting
//!   is worth more than a parameter change.

use glam::{DAffine3, DVec3};
use serde::{Deserialize, Serialize};

use crate::clipboard::Clip;
use crate::recipe::Recipe;
use crate::scene::{DefinitionId, DefinitionKind, Scene};
use crate::{Document, Entity, InstanceId};

/// What a parametric component was built from, kept on its definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parametric {
    /// The recipe, parameters and all: its `parameters` are the numbers on screen.
    pub recipe: Recipe,
    /// The geometry it had when it was built, so a rebuild can tell whether anyone has changed it
    /// since.
    pub as_built: u64,
}

impl Document {
    /// Build `recipe` as one component beside the drawing, remembering the recipe so its numbers
    /// can be changed later. One undo step, and it ends up selected.
    pub fn add_parametric(&mut self, recipe: &Recipe) -> Result<InstanceId, String> {
        let built = crate::recipe::build(recipe).map_err(|problems| problems.join(" "))?;
        let clip = Clip::from_scene(built.scene().clone()).ok_or("That recipe builds nothing.")?;
        // To the right of everything there is, a foot away, as a template goes.
        let beside = self.bounds().map_or(DVec3::ZERO, |(lo, hi)| DVec3::new(hi.x + 12.0, lo.y, 0.0));
        let name = if recipe.name.is_empty() { "Component".to_owned() } else { recipe.name.clone() };
        self.as_one_step(|doc| {
            let group = doc.paste_as_one(&clip, beside, &name)?;
            doc.remember(group, recipe);
            let selection = doc.selection_mut();
            selection.clear();
            selection.extend([Entity::Instance(group)]);
            Ok(group)
        })
    }

    /// Read `instance` back as a recipe and keep it, so that from now on its numbers can be
    /// changed and it can be built again. The numbers are whatever it was drawn with: naming
    /// them is the next job, in the panel.
    ///
    /// Anything the recipe can't describe comes back as notes, and they matter here more than
    /// they do when reading a whole drawing: building it again is what the component is for, so
    /// a piece that wouldn't come back the same says so rather than waiting to surprise anyone.
    pub fn make_parametric(&mut self, instance: InstanceId) -> Result<Vec<String>, String> {
        if self.parametric_of(instance).is_some() {
            return Err("That already has numbers of its own.".into());
        }
        self.container().instance(instance).ok_or("That isn't in the group being edited.")?;
        // On its own in a drawing of its own, so the reader describes it and nothing else.
        let was = self.selection().iter().collect::<Vec<_>>();
        let selection = self.selection_mut();
        selection.clear();
        selection.extend([Entity::Instance(instance)]);
        let clip = self.copy_selection();
        let selection = self.selection_mut();
        selection.clear();
        selection.extend(was);
        let clip = clip.ok_or("There is nothing there to read.")?;
        let mut alone = Document::default();
        alone.paste(&clip, DVec3::ZERO);
        let (recipe, notes) = crate::recipe_of::from_document(&alone);
        if recipe.parts.is_empty() && recipe.templates.is_empty() {
            return Err("There are no parts in that to describe.".into());
        }
        self.remember(instance, &recipe);
        Ok(notes)
    }

    /// What `instance` was built from, if it is a parametric component.
    pub fn parametric_of(&self, instance: InstanceId) -> Option<&Parametric> {
        let definition = self.container().instance(instance)?.definition;
        self.scene().definition(definition)?.parametric.as_deref()
    }

    /// Whether `instance` has been changed since it was built, so rebuilding it would lose work.
    pub fn parametric_changed_by_hand(&self, instance: InstanceId) -> bool {
        let Some(placed) = self.container().instance(instance) else {
            return false;
        };
        let Some(made) = self.parametric_of(instance) else {
            return false;
        };
        fingerprint(self.scene(), placed.definition) != made.as_built
    }

    /// How many places this component's design is used, which is how many would change with it.
    pub fn parametric_copies(&self, instance: InstanceId) -> usize {
        let Some(placed) = self.container().instance(instance) else {
            return 0;
        };
        self.scene().instance_count(placed.definition)
    }
    /// Build `recipe` again in place of `instance`, keeping where it sits, its tag and its name.
    /// One undo step.
    ///
    /// Refused when the component has been changed by hand, unless `anyway`. Where the design is
    /// placed more than once, only this placement changes: it becomes a component of its own, and
    /// the others are left as they were.
    pub fn rebuild_parametric(&mut self, instance: InstanceId, recipe: &Recipe, anyway: bool) -> Result<InstanceId, String> {
        self.parametric_of(instance).ok_or("That isn't a component built from a recipe.")?;
        if !anyway && self.parametric_changed_by_hand(instance) {
            return Err("This one has been changed by hand since it was built, and building it again would lose that.".into());
        }
        let built = crate::recipe::build(recipe).map_err(|problems| problems.join(" "))?;
        let clip = Clip::from_scene(built.scene().clone()).ok_or("That recipe builds nothing.")?;
        let old = self.container().instance(instance).ok_or("That isn't in the group being edited.")?;
        let (transform, tag, material) = (old.transform, old.tag, old.material);
        let name = self.instance_name(instance).unwrap_or("Component").to_owned();
        self.as_one_step(|doc| {
            // Its own design from here on, so the other placements of the old one stay as they are.
            doc.edit_scene(|scene, path| {
                scene.make_unique(path, instance);
            });
            doc.erase(&[Entity::Instance(instance)]);
            let group = doc.paste_as_one(&clip, transform.translation, &name)?;
            doc.edit_scene(|scene, path| {
                if let Some(placed) = scene.container_mut(path).and_then(|c| c.instance_mut(group)) {
                    placed.transform = transform;
                    placed.tag = tag;
                    placed.material = material;
                }
            });
            doc.remember(group, recipe);
            let selection = doc.selection_mut();
            selection.clear();
            selection.extend([Entity::Instance(group)]);
            Ok(group)
        })
    }

    /// Paste everything in `clip` at `to` as a single group named `name`. A recipe that already
    /// puts its parts in one group is that group, rather than a group holding a group.
    fn paste_as_one(&mut self, clip: &Clip, to: DVec3, name: &str) -> Result<InstanceId, String> {
        let pasted = self.paste(clip, to);
        let instances: Vec<InstanceId> = pasted.iter().filter_map(|e| if let Entity::Instance(i) = e { Some(*i) } else { None }).collect();
        let group = match instances[..] {
            [one] if pasted.len() == 1 => one,
            [] => return Err("That recipe builds nothing.".into()),
            _ => {
                let selection = self.selection_mut();
                selection.clear();
                selection.extend(pasted);
                self.group_selection(DefinitionKind::Group).ok_or("Nothing was built.")?
            }
        };
        self.rename_instance(group, name);
        Ok(group)
    }

    /// Keep the recipe on the component's own design, with the geometry it was built with.
    fn remember(&mut self, group: InstanceId, recipe: &Recipe) {
        let definition = self.container().instance(group).map(|placed| placed.definition);
        let Some(definition) = definition else { return };
        let as_built = fingerprint(self.scene(), definition);
        let made = Parametric { recipe: recipe.clone(), as_built };
        self.edit_scene(|scene, _| {
            if let Some(design) = scene.definition_mut(definition) {
                design.parametric = Some(Box::new(made));
            }
        });
    }
}

/// A number that says what a component's geometry was when it was built.
///
/// Every vertex of every part, where it sits in the component, and how many edges and faces each
/// part has. Sorted, so nothing depends on the order the scene happens to hold things in, and
/// nothing is taken from the ids, which are handed out afresh on every build.
///
/// Worked out here rather than with the standard library's hasher, whose numbers are allowed to
/// change between Rust versions: this one is written into the drawing and has to mean the same
/// thing next year, or every component would claim to have been edited by hand after an update.
fn fingerprint(scene: &Scene, definition: DefinitionId) -> u64 {
    let mut numbers = Vec::new();
    gather(scene, definition, DAffine3::IDENTITY, &mut numbers, 0);
    numbers.sort_unstable();
    // FNV-1a, which is a few lines and the same everywhere.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for number in numbers {
        for byte in number.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}

fn gather(scene: &Scene, definition: DefinitionId, at: DAffine3, numbers: &mut Vec<u64>, depth: usize) {
    // A drawing cannot nest this far, and a definition that somehow holds itself stops here
    // rather than running for ever.
    if depth > 32 {
        return;
    }
    let Some(design) = scene.definition(definition) else {
        return;
    };
    let model = &design.container.model;
    for (_, point) in model.vertices() {
        numbers.push(point_number(at.transform_point3(point)));
    }
    // The counts as well, so erasing a face that leaves its corners behind is still a change.
    numbers.push(0xface_u64 << 48 | model.faces().count() as u64);
    numbers.push(0xedda_u64 << 48 | model.edges().count() as u64);
    for (_, placed) in design.container.instances() {
        gather(scene, placed.definition, at * placed.transform, numbers, depth + 1);
    }
}

/// A point as one number, to a millionth of an inch: closer than any joint cares about, and far
/// enough from the last bit that building the same recipe twice gives the same answer.
fn point_number(point: DVec3) -> u64 {
    let round = |v: f64| (v * 1e6).round() as i64 as u64;
    round(point.x).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ round(point.y).wrapping_mul(0xbf58_476d_1ce4_e5b9) ^ round(point.z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::parse;

    /// A door frame whose panel is worked out from its width: the smallest thing that is worth
    /// resizing rather than redrawing.
    fn door(width: f64) -> Recipe {
        parse(&format!(
            r#"{{
            "name": "Door",
            "parameters": [
                {{ "name": "width", "value": {width} }},
                {{ "name": "height", "value": 24 }},
                {{ "name": "stile", "value": 2.25 }}
            ],
            "materials": [{{ "id": "oak", "library": "White Oak" }}],
            "parts": [
                {{ "name": "Left stile", "called": "Stile", "material": "oak", "size": ["height", "stile", 0.75], "at": [0, 0, 0], "length": "z", "width": "x" }},
                {{ "name": "Right stile", "called": "Stile", "material": "oak", "size": ["height", "stile", 0.75], "at": ["width - stile", 0, 0], "length": "z", "width": "x" }},
                {{ "name": "Panel", "material": "oak", "size": ["height", "width - 2*stile", 0.25], "at": ["stile", 0.25, 0], "length": "z", "width": "x" }}
            ],
            "groups": [{{ "name": "Door", "parts": ["Left stile", "Right stile", "Panel"] }}]
        }}"#
        ))
        .expect("the recipe reads")
    }

    fn width_of(doc: &Document) -> f64 {
        let (lo, hi) = doc.bounds().expect("geometry");
        hi.x - lo.x
    }

    /// The size of the part called `name`, the long way first, wherever it sits.
    fn part(doc: &Document, group: InstanceId, name: &str) -> [f64; 3] {
        let scene = doc.scene();
        let design = scene.definition(doc.container().instance(group).expect("placed").definition).expect("a design");
        let inner = design.container.instances().find(|(_, placed)| scene.definition(placed.definition).is_some_and(|d| d.name == name)).expect("the part");
        let (lo, hi) = scene.definition_bounds(inner.1.definition).expect("its size");
        let mut size = [hi.x - lo.x, hi.y - lo.y, hi.z - lo.z];
        size.sort_by(f64::total_cmp);
        size
    }

    /// Sizes match to a thousandth, smallest first, so which side is longest doesn't matter.
    fn same(got: [f64; 3], wanted: [f64; 3]) -> bool {
        got.iter().zip(&wanted).all(|(a, b)| (a - b).abs() < 1e-3)
    }

    #[test]
    fn a_component_remembers_its_recipe_and_is_built_again_from_it() {
        let mut doc = Document::default();
        let group = doc.add_parametric(&door(16.0)).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(doc.instance_name(group), Some("Door"));
        assert!((width_of(&doc) - 16.0).abs() < 1e-9, "16\" wide: {}", width_of(&doc));
        assert!(same(part(&doc, group, "Panel"), [0.25, 11.5, 24.0]), "the panel is the opening wide and the full height: {:?}", part(&doc, group, "Panel"));
        // It kept what it was built from, with the numbers it was given.
        let kept = doc.parametric_of(group).expect("it remembers").recipe.clone();
        assert_eq!(kept.parameters.len(), 3);
        assert!(!doc.parametric_changed_by_hand(group), "nobody has touched it");
        // One number, built again: the panel follows and the stiles stay as they are.
        let group = doc.rebuild_parametric(group, &door(30.0), false).unwrap_or_else(|e| panic!("{e}"));
        assert!((width_of(&doc) - 30.0).abs() < 1e-9, "30\" wide now: {}", width_of(&doc));
        assert!(same(part(&doc, group, "Panel"), [0.25, 24.0, 25.5]), "a wider opening: {:?}", part(&doc, group, "Panel"));
        assert!(same(part(&doc, group, "Stile"), [0.75, 2.25, 24.0]), "the stiles don't stretch: {:?}", part(&doc, group, "Stile"));
        assert_eq!(doc.instance_name(group), Some("Door"), "and it is still called what it was");
        // Building it again is one undo step, as a template's is.
        assert!(doc.undo());
        assert!((width_of(&doc) - 16.0).abs() < 1e-9, "back to 16\": {}", width_of(&doc));
    }

    #[test]
    fn a_component_changed_by_hand_will_not_be_built_over_without_being_told_twice() {
        let mut doc = Document::default();
        let group = doc.add_parametric(&door(16.0)).unwrap_or_else(|e| panic!("{e}"));
        // Inside it, a part moved by hand -- a fitting nobody wants thrown away.
        assert!(doc.enter(group), "into the component");
        let (inside, _) = doc.container().instances().next().expect("a part inside");
        doc.transform_entities(&[Entity::Instance(inside)], &[DAffine3::from_translation(DVec3::new(0.0, 0.0, 1.0))], false).expect("moved");
        doc.exit();
        assert!(doc.parametric_changed_by_hand(group), "it knows");
        let refused = doc.rebuild_parametric(group, &door(30.0), false).expect_err("refused");
        assert!(refused.contains("changed by hand"), "{refused}");
        assert!((width_of(&doc) - 16.0).abs() < 1e-9, "and nothing happened");
        // Told twice, it builds and the hand work is gone, which is what the warning said.
        let group = doc.rebuild_parametric(group, &door(30.0), true).unwrap_or_else(|e| panic!("{e}"));
        assert!((width_of(&doc) - 30.0).abs() < 1e-9);
        assert!(!doc.parametric_changed_by_hand(group), "built afresh");
    }

    #[test]
    fn a_saved_component_can_still_be_built_again_when_it_is_opened() {
        // The recipe and the numbers travel with the drawing, or resizing would only work in
        // the session that made it.
        let mut doc = Document::default();
        let group = doc.add_parametric(&door(16.0)).unwrap_or_else(|e| panic!("{e}"));
        let written = crate::file::to_string(doc.scene());
        let scene = crate::file::from_str(&written).expect("read back");
        let mut doc = Document::from_scene(scene);
        let (group_again, _) = doc.container().instances().next().expect("the component");
        assert_eq!(group_again, group, "the same placement");
        let kept = doc.parametric_of(group_again).expect("it still remembers its recipe");
        assert_eq!(kept.recipe.parameters.len(), 3);
        assert!(!doc.parametric_changed_by_hand(group_again), "and still says nobody has touched it");
        let group_again = doc.rebuild_parametric(group_again, &door(30.0), false).unwrap_or_else(|e| panic!("{e}"));
        assert!(same(part(&doc, group_again, "Panel"), [0.25, 24.0, 25.5]), "resized after a round trip");
    }

    #[test]
    fn a_group_somebody_drew_can_be_given_numbers_of_its_own() {
        // Nothing is built from a recipe here: two boards drawn by hand, read back into one so
        // that from then on they can be resized together (#9).
        use crate::board::Board;
        let mut doc = Document::default();
        let oak = doc.add_preset(crate::library::LIBRARY.iter().find(|p| p.name == "Red Oak").expect("oak"));
        let board = |doc: &mut Document, at: DVec3, length: f64| {
            doc.add_board(&Board { origin: at, length_dir: DVec3::X, width_dir: DVec3::Y, length, width: 3.5, thickness: 0.75 }, Some(oak)).expect("a board")
        };
        let one = board(&mut doc, DVec3::ZERO, 24.0);
        let two = board(&mut doc, DVec3::new(0.0, 4.0, 0.0), 24.0);
        doc.selection_mut().extend([Entity::Instance(one), Entity::Instance(two)]);
        let group = doc.group_selection(crate::scene::DefinitionKind::Group).expect("grouped");
        doc.rename_instance(group, "Frame");
        assert!(doc.parametric_of(group).is_none(), "nothing to change yet");
        let notes = doc.make_parametric(group).unwrap_or_else(|e| panic!("{e}"));
        assert!(notes.is_empty(), "two plain boards read back whole: {notes:?}");
        let kept = doc.parametric_of(group).expect("it has numbers now").recipe.clone();
        assert_eq!(kept.parts.len(), 2, "both boards");
        assert!(kept.parameters.is_empty(), "none of them named yet: that is the next job");
        assert!(!doc.parametric_changed_by_hand(group), "read back, not rebuilt");
        // Naming one and using it: the point of the whole thing.
        let mut recipe = kept;
        recipe.parameters.push(crate::recipe::ParameterSpec {
            name: "length".into(),
            value: crate::recipe::Length::Number(36.0),
            label: None,
            choices: Vec::new(),
        });
        for part in &mut recipe.parts {
            part.size[0] = crate::recipe::Length::Text("length".into());
        }
        let group = doc.rebuild_parametric(group, &recipe, false).unwrap_or_else(|e| panic!("{e}"));
        let (lo, hi) = doc.bounds().expect("geometry");
        assert!((hi.x - lo.x - 36.0).abs() < 1e-6, "both boards are 36\" now: {}", hi.x - lo.x);
        assert_eq!(doc.instance_name(group), Some("Frame"), "and it kept its name");
        // Asked twice, it says no: it already has numbers.
        assert!(doc.make_parametric(group).is_err());
    }

    #[test]
    fn resizing_one_copy_leaves_the_others_as_they_were() {
        let mut doc = Document::default();
        let group = doc.add_parametric(&door(16.0)).unwrap_or_else(|e| panic!("{e}"));
        doc.selection_mut().clear();
        doc.selection_mut().extend([Entity::Instance(group)]);
        let clip = doc.copy_selection().expect("a clip");
        let pasted = doc.paste(&clip, DVec3::new(40.0, 0.0, 0.0));
        let Some(&Entity::Instance(copy)) = pasted.first() else { panic!("a copy") };
        assert_eq!(doc.parametric_copies(group), 2, "one design in two places");
        // Widening the copy makes it its own design rather than widening both.
        let copy = doc.rebuild_parametric(copy, &door(30.0), false).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(doc.parametric_copies(group), 1, "the first is its own again");
        assert_eq!(doc.parametric_copies(copy), 1, "and so is the one that changed");
        assert!(same(part(&doc, copy, "Panel"), [0.25, 24.0, 25.5]), "the copy is wider: {:?}", part(&doc, copy, "Panel"));
        assert!(same(part(&doc, group, "Panel"), [0.25, 11.5, 24.0]), "the original is not: {:?}", part(&doc, group, "Panel"));
    }
}
