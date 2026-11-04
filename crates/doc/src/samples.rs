//! Finished pieces built into the app, for the Start page's Samples and for anyone who wants to
//! see what a drawing looks like before making one.
//!
//! Each is a **recipe**, not a saved drawing, for three reasons:
//!
//! - Size: the nightstand's recipe is 272 bytes against the 520 KB drawing it builds.
//! - They don't go stale. A saved drawing freezes the geometry of the day it was saved; a recipe
//!   is built by today's joinery, so a sample always shows the app as it is now.
//! - [`every_sample_builds`](tests::every_sample_builds) builds them all, which is the widest
//!   end-to-end test here for the price: a joinery change that breaks one fails in CI rather
//!   than in somebody's hands.
//!
//! Building one takes about 200 ms, so it can happen on the click that asks for it.

use crate::Document;
use crate::recipe::{self, Recipe};

/// A piece someone can open and take apart, to see how a drawing is put together.
pub struct Sample {
    /// What it is, as the card's title.
    pub name: &'static str,
    /// What it shows off, as the card's line under the title. Name the feature, not the object:
    /// someone reading the cards is deciding which one teaches them the most.
    pub shows: &'static str,
    /// Its recipe, built into the binary.
    pub recipe: &'static str,
}

impl Sample {
    /// Read the recipe (it is checked at build time by the tests, so this only fails if the file
    /// in the repository is broken).
    pub fn recipe(&self) -> Result<Recipe, String> {
        serde_json::from_str(self.recipe).map_err(|e| format!("{}'s recipe couldn't be read: {e}", self.name))
    }

    /// Build the drawing. Slow enough to notice (~200 ms) but not slow enough to need a thread.
    pub fn build(&self) -> Result<Document, String> {
        let recipe = self.recipe()?;
        recipe::build(&recipe).map_err(|problems| format!("{} couldn't be built: {}", self.name, problems.join("; ")))
    }
}

/// Every sample, simplest first: that is the order the Start page shows them in, and the order
/// somebody working through them should take.
pub const SAMPLES: &[Sample] = &[
    Sample {
        name: "Bookshelf",
        shows: "Dadoed shelves in an oak case, with the cut list and cutting diagrams",
        recipe: include_str!("../../../samples/bookshelf.json"),
    },
    Sample {
        name: "Nightstand",
        shows: "Two dovetailed drawers on side-mount slides, with the hardware counted",
        recipe: include_str!("../../../samples/nightstand.json"),
    },
    Sample {
        name: "Workbench",
        shows: "A mortise-and-tenon base under a glued-up maple top, with the movement check",
        recipe: include_str!("../../../samples/workbench.json"),
    },
    Sample {
        name: "Dining table",
        shows: "Aprons tenoned into cherry legs, a rounded-over top let to move with the seasons",
        recipe: include_str!("../../../samples/table.json"),
    },
    Sample {
        name: "Kitchen run",
        shows: "Five cabinets, seventy parts: the cut list, the sheets to buy and the hardware",
        recipe: include_str!("../../../samples/kitchen.json"),
    },
];

/// The sample called `name`, for opening one by name.
pub fn by_name(name: &str) -> Option<&'static Sample> {
    SAMPLES.iter().find(|s| s.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every sample builds, and builds into something recognizable. The counts are deliberately
    /// exact: a joinery change that quietly stops cutting a dado should fail here, which is the
    /// whole reason the samples are recipes rather than saved drawings.
    #[test]
    fn every_sample_builds() {
        // name, parts in the cut list, joints
        let expected = [
            ("Bookshelf", 8, 10),
            ("Nightstand", 18, 22),
            ("Workbench", 13, 16),
            ("Dining table", 9, 8),
            ("Kitchen run", 75, 86),
        ];
        assert_eq!(expected.len(), SAMPLES.len(), "every sample needs its counts here");
        for (sample, (name, parts, joints)) in SAMPLES.iter().zip(expected) {
            assert_eq!(sample.name, name);
            let doc = sample.build().unwrap_or_else(|e| panic!("{e}"));
            let list = crate::cutlist::cut_list(doc.scene());
            let counted: usize = list.materials.iter().flat_map(|m| &m.parts).map(|p| p.count).sum();
            assert_eq!(counted, parts, "{name}: parts in the cut list");
            assert_eq!(doc.scene().joints.len(), joints, "{name}: joints");
        }
    }

    /// The cards are read before anything is built, so the text has to stand on its own.
    #[test]
    fn every_sample_says_what_it_shows() {
        for sample in SAMPLES {
            assert!(!sample.name.is_empty());
            assert!(sample.shows.len() > 20, "{}: say what it demonstrates", sample.name);
            assert!(!sample.shows.ends_with('.'), "{}: the card's line isn't a sentence", sample.name);
        }
    }

    /// Nothing about a sample should be a surprise to the wood: a sample that opens with a
    /// warning on its movement tab reads as a design that was got wrong, whatever the warning
    /// says. Where one is genuinely held (the workbench's rails, tenoned into legs that run the
    /// other way), it is listed here with why it's right anyway.
    #[test]
    fn samples_have_no_movement_warnings_they_shouldnt() {
        // Sample, and the warnings it is allowed to carry.
        let allowed = [("Workbench", 2)];
        for sample in SAMPLES {
            let doc = sample.build().unwrap();
            let scene = doc.scene();
            let report = crate::movement::check(scene, scene.moisture_swing.unwrap_or(crate::movement::DEFAULT_SWING));
            let expected = allowed.iter().find(|(name, _)| *name == sample.name).map_or(0, |(_, n)| *n);
            assert_eq!(
                report.conflicts.len(),
                expected,
                "{}: {:?}",
                sample.name,
                report.conflicts.iter().map(|c| format!("{c:?}")).collect::<Vec<_>>()
            );
        }
    }

    /// Samples carry their own units, so opening one shows the sizes its recipe asked for.
    #[test]
    fn samples_are_in_inches() {
        for sample in SAMPLES {
            assert_eq!(sample.build().unwrap().scene().units, Some(crate::units::Units::Inches), "{}", sample.name);
        }
    }
}
