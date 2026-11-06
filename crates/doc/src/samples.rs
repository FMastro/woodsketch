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

    /// Build the drawing.
    pub fn build(&self) -> Result<Document, String> {
        self.build_reporting(&mut |_, _, _| {})
    }

    /// Build it, saying after each piece how far along it is (`done`, `all`, what it just made).
    /// The furniture takes about 200 ms and the house a couple of seconds, which is why the app
    /// builds on a thread ([`ui_egui::building`]) and watches this rather than holding the frame.
    pub fn build_reporting(&self, progress: &mut dyn FnMut(usize, usize, &str)) -> Result<Document, String> {
        let recipe = self.recipe()?;
        recipe::build_reporting(&recipe, progress)
            .map_err(|problems| format!("{} couldn't be built: {}", self.name, problems.join("; ")))
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
    Sample {
        name: "Small house",
        shows: "A thousand parts: a framed 2 bed, 2 bath, tagged so you can peel it apart",
        recipe: include_str!("../../../samples/house.json"),
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
            // 990 rather than 995: the five doors start hidden, and a hidden part is not cut.
            ("Small house", 990, 72),
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

    /// The house is the one sample too big to take in at once, and the way into it is the Tags
    /// panel: hide Roofing and Roof sheathing and you are looking at the rafters, hide those and
    /// you are standing in the rooms. That rests entirely on its parts being tagged and the tags
    /// being in folders, which no count above would notice the loss of.
    #[test]
    fn the_house_comes_apart_by_tag() {
        let doc = by_name("Small house").expect("the house is a sample").build().unwrap();
        let scene = doc.scene();
        let folders: Vec<&str> = scene.tags().filter(|(_, t)| t.folder).map(|(_, t)| t.name.as_str()).collect();
        for wanted in ["Shell", "Roof", "Inside", "Outside"] {
            assert!(folders.contains(&wanted), "the house's tags go in folders: {folders:?}");
        }
        for wanted in ["Wall framing", "Roofing", "Drywall", "Windows"] {
            let tag = scene.tags().find(|(_, t)| t.name == wanted);
            let (_, tag) = tag.unwrap_or_else(|| panic!("the house has a \"{wanted}\" tag: {folders:?}"));
            assert!(tag.parent.is_some(), "\"{wanted}\" sits in a folder");
            assert!(!tag.folder, "\"{wanted}\" is a tag, not a folder");
        }
        // Everything placed carries one, or hiding a layer would leave pieces of it behind.
        let untagged = scene.root.instances().filter(|(_, i)| i.tag.is_none()).count();
        assert_eq!(untagged, 0, "every part of the house is tagged");
    }

    /// The house's door knobs are the one thing in it nobody cuts: they come in a box. So they
    /// belong in the shopping list and nowhere near the cutting diagrams, which is what marking
    /// them `hardware` buys. Getting that backwards would quietly put twenty metal discs into a
    /// sheet-goods layout, and no part count above would notice.
    #[test]
    fn the_houses_knobs_are_bought_not_cut() {
        let mut doc = by_name("Small house").expect("the house is a sample").build().unwrap();
        // The doors start hidden, and a hidden door's knobs are bought by nobody. Show them,
        // which is what anyone pricing the job would do, and then count.
        let doors = doc.scene().tags().find(|(_, t)| t.name == "Doors").expect("a Doors tag").0;
        doc.edit_scene(|scene, _| scene.tag_mut(doors).unwrap().visible = true);
        let list = crate::cutlist::cut_list(doc.scene());
        let bought = list.hardware(doc.scene());
        for (piece, count) in [("Passage knob", 10), ("Knob rose", 10)] {
            let found = bought.iter().find(|(name, _)| name == piece);
            let (_, n) = found.unwrap_or_else(|| panic!("the house buys {piece}: {bought:?}"));
            assert_eq!(*n, count, "{piece}: one on each side of all five doors");
        }
        let cut: Vec<&String> = list.materials.iter().flat_map(|m| &m.parts).flat_map(|p| &p.names).collect();
        assert!(!cut.iter().any(|n| n.contains("knob") || n.contains("rose")), "no knob is cut from anything: {cut:?}");
    }

    /// The doors start hidden, and nothing else inside does.
    ///
    /// Shut doors stand between you and every room in the house, which is the one thing anyone
    /// opens this sample to look at. They are a click away in the Tags panel, which is also the
    /// first thing the sample is for. The cost is that a hidden part is not a cut-list part, so
    /// the five doors are out of the counts above until the tag is shown.
    #[test]
    fn the_houses_doors_start_hidden() {
        let doc = by_name("Small house").expect("the house is a sample").build().unwrap();
        let scene = doc.scene();
        let doors = scene.tags().find(|(_, t)| t.name == "Doors").expect("the house has a Doors tag");
        assert!(!doors.1.visible, "the doors start hidden");
        for (_, tag) in scene.tags().filter(|(_, t)| t.name != "Doors") {
            assert!(tag.visible, "only the doors start hidden, not {}", tag.name);
        }
    }

    /// Nothing in either bathroom crosses a wall or stands in its doorway.
    ///
    /// Reported from walking the house: a tub sat 3-1/2" behind a bathroom door and ran through
    /// the wall into the next bathroom, and a toilet stood out through the other door into the
    /// bedroom. The fixtures were sized for rooms 60" clear and the rooms are 56-1/2".
    #[test]
    fn nothing_in_the_baths_stands_in_a_doorway() {
        // From `scripts/make-house-sample.py`: each bath's clear floor, and the floor just
        // inside its door, which is the part of a room nothing is allowed to stand on.
        const BATHS: [[f64; 4]; 2] = [[135.5, 171.5, 195.5, 282.5], [199.0, 171.5, 259.0, 282.5]];
        // Bath 1 is entered through its south wall, bath 2 through its east wall; 24" deep.
        const DOORWAYS: [[f64; 4]; 2] = [[150.0, 171.5, 184.0, 195.5], [235.0, 200.0, 259.0, 234.0]];
        const EPS: f64 = 1e-6;

        let doc = by_name("Small house").expect("the house is a sample").build().unwrap();
        let scene = doc.scene();
        let (bathrooms, _) = scene.tags().find(|(_, t)| t.name == "Bathrooms").expect("a Bathrooms tag");
        let mut checked = 0;
        for (_, instance) in scene.root.instances().filter(|(_, i)| i.tag == Some(bathrooms)) {
            let Some((lo, hi)) = scene.definition_bounds(instance.definition) else { continue };
            let corners = [
                glam::DVec3::new(lo.x, lo.y, lo.z),
                glam::DVec3::new(hi.x, lo.y, lo.z),
                glam::DVec3::new(lo.x, hi.y, lo.z),
                glam::DVec3::new(hi.x, hi.y, lo.z),
                glam::DVec3::new(lo.x, lo.y, hi.z),
                glam::DVec3::new(hi.x, lo.y, hi.z),
                glam::DVec3::new(lo.x, hi.y, hi.z),
                glam::DVec3::new(hi.x, hi.y, hi.z),
            ];
            let placed: Vec<glam::DVec3> = corners.iter().map(|c| instance.transform.transform_point3(*c)).collect();
            let x0 = placed.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
            let x1 = placed.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
            let y0 = placed.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
            let y1 = placed.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
            let name = scene.definition(instance.definition).map_or("?", |d| d.name.as_str());
            let box_ = format!("{name} at x {x0:.1}..{x1:.1}, y {y0:.1}..{y1:.1}");

            let inside = BATHS.iter().any(|[bx0, by0, bx1, by1]| {
                x0 >= bx0 - EPS && x1 <= bx1 + EPS && y0 >= by0 - EPS && y1 <= by1 + EPS
            });
            assert!(inside, "{box_} is not inside one bathroom's walls");
            for [dx0, dy0, dx1, dy1] in DOORWAYS {
                let overlaps = x0 < dx1 - EPS && x1 > dx0 + EPS && y0 < dy1 - EPS && y1 > dy0 + EPS;
                assert!(!overlaps, "{box_} stands in the floor inside a bathroom door");
            }
            checked += 1;
        }
        assert_eq!(checked, 6, "two baths, three fixtures each");
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
