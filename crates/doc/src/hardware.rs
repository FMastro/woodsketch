//! Hardware placed as parts: drawer slides, hinges, pulls and knobs. Each is a group of plain
//! boxes (no maker's models) in a steel material, whose definition names the piece
//! ([`crate::Definition::hardware`]): the cut list leaves it out and the shopping list counts it.

use std::collections::BTreeMap;

use crate::units::format_length;
use crate::{Container, Material, MaterialId, MaterialKind, Scene};

/// Side-mount slide lengths sold (inches).
pub const SLIDE_LENGTHS: [f64; 8] = [10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0];
/// Side-mount slides take 1/2" on each side of the drawer box.
pub const SLIDE_SPACE: f64 = 0.5;

/// The longest slide that fits `depth` (the room behind the drawer front).
pub fn slide_for(depth: f64) -> Option<f64> {
    SLIDE_LENGTHS.iter().copied().rev().find(|l| *l <= depth + 1e-9)
}

/// The shopping-list name of a pair of side-mount slides.
pub fn slides_name(length: f64) -> String {
    format!("{} side-mount drawer slides (pair)", format_length(length))
}

pub const HINGE: &str = "Concealed hinge (35 mm cup)";
pub const PULL: &str = "Bar pull, 5\"";
pub const KNOB: &str = "Knob";

/// How many hinges a door `height` tall needs: two up to 40", three up to 60", then four.
pub fn hinges_for(height: f64) -> usize {
    if height <= 40.0 {
        2
    } else if height <= 60.0 {
        3
    } else {
        4
    }
}

/// The drawing's material for hardware (added if it isn't there).
pub(crate) fn steel(scene: &mut Scene) -> MaterialId {
    if let Some((id, _)) = scene.materials().find(|(_, m)| m.kind == MaterialKind::Hardware) {
        return id;
    }
    scene.add_material(Material::new("Steel", [150, 154, 160], MaterialKind::Hardware, false))
}

/// Every piece of hardware placed in the drawing (shown, counting copies): `name -> count`.
pub fn placed(scene: &Scene) -> BTreeMap<String, u32> {
    let mut total = BTreeMap::new();
    count(scene, &scene.root, &mut total, 0);
    total
}

fn count(scene: &Scene, container: &Container, total: &mut BTreeMap<String, u32>, depth: usize) {
    if depth > 64 {
        return;
    }
    for (_, instance) in container.instances() {
        if !scene.is_visible(instance) {
            continue;
        }
        let Some(definition) = scene.definition(instance.definition) else { continue };
        match &definition.hardware {
            Some(name) => *total.entry(name.clone()).or_default() += 1,
            None => count(scene, &definition.container, total, depth + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slides_are_the_longest_that_fit() {
        assert_eq!(slide_for(11.0), Some(10.0));
        assert_eq!(slide_for(18.0), Some(18.0));
        assert_eq!(slide_for(30.0), Some(24.0));
        assert_eq!(slide_for(9.0), None);
        assert_eq!(slides_name(18.0), "18\" side-mount drawer slides (pair)");
    }

    #[test]
    fn tall_doors_get_more_hinges() {
        assert_eq!([30.0, 48.0, 72.0].map(hinges_for), [2, 3, 4]);
    }
}
