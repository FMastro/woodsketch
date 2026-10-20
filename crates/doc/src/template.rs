//! Templates: common furniture assemblies built from boards and real joints, so the cut list,
//! build sheets, joint list and movement checks work on them as on parts drawn by hand.
//!
//! - **Cabinet** (or bookcase): sides, top, bottom and shelves between them, a 1/4" plywood
//!   back; dados and rabbets, biscuits, pocket screws or plain butt joints.
//! - **Drawer**: a 1/2" box (front, back, sides) with dovetails, box joints or rabbets, and a
//!   1/4" plywood bottom in grooves.
//! - **Face frame**: stiles and rails, with mortise and tenon, pocket screws or dowels.
//! - **Table**: one leg component turned to each corner, apron components joined to the legs,
//!   a top overhanging 1".
//!
//! Each is one group named for it, made in one undo step, and remembers its sizes
//! ([`Made`] on its definition) so [`Document::rebuild_template`] can change them.

use glam::{DAffine3, DVec3};
use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::joinery::{Dovetails, Dowels, JoineryError, Slots, Tenon, pocket_screw_for};
use crate::library::LIBRARY;
use crate::{DefinitionKind, Document, Entity, InstanceId, MaterialId};

/// Case and frame stock: 3/4".
const STOCK: f64 = 0.75;
/// Drawer box stock: 1/2".
const DRAWER_STOCK: f64 = 0.5;
/// Backs and drawer bottoms: 1/4" plywood.
const PANEL: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Template {
    Cabinet(Cabinet),
    Drawer(Drawer),
    FaceFrame(FaceFrame),
    Table(Table),
}

/// Overall sizes, in inches. Width along X (red), depth along Y (green) from the front, height
/// up Z (blue).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cabinet {
    pub width: f64,
    pub height: f64,
    /// Including the back.
    pub depth: f64,
    pub shelves: u32,
    pub back: bool,
    pub joints: CaseJoints,
    /// What fills each opening between shelves, from the bottom (openings past the last are
    /// ignored).
    #[serde(default)]
    pub fills: [Fill; MAX_OPENINGS],
    #[serde(default)]
    pub fronts: Fronts,
    #[serde(default)]
    pub doors: DoorStyle,
    #[serde(default)]
    pub pulls: Pulls,
    /// The drawer boxes' corners.
    #[serde(default)]
    pub drawer_joints: DrawerJoints,
    /// The case's stock: sides, top, bottom and shelves.
    #[serde(default = "three_quarters")]
    pub stock: f64,
    /// Shelves between the top and bottom sit loose on shelf pins (and aren't joined).
    #[serde(default)]
    pub adjustable: bool,
    /// Which side a single door hinges on.
    #[serde(default)]
    pub hinge: HingeSide,
}

/// Which side a door hinges on, seen from the front.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HingeSide {
    #[default]
    Left,
    Right,
}

impl HingeSide {
    pub const ALL: [Self; 2] = [Self::Left, Self::Right];
    pub fn name(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

fn three_quarters() -> f64 {
    STOCK
}

fn half() -> f64 {
    DRAWER_STOCK
}

fn leg_size() -> f64 {
    1.75
}

fn apron_width() -> f64 {
    4.0
}

/// The most openings a cabinet has (20 shelves).
pub const MAX_OPENINGS: usize = 21;

/// What fills an opening of a cabinet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fill {
    #[default]
    Open,
    /// A drawer on side-mount slides, with a front.
    Drawer,
    /// A door, or a pair when the opening is wider than 24".
    Doors,
}

/// How drawer fronts and doors sit on a cabinet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fronts {
    /// Over the cabinet's edges (full overlay), 1/16" apart.
    #[default]
    Overlay,
    /// Inside the openings, flush with the front, 1/16" gap all round.
    Inset,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorStyle {
    /// One 3/4" board.
    #[default]
    Slab,
    /// Stiles and rails, stub-tenoned, with a 1/4" plywood panel in grooves.
    FrameAndPanel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pulls {
    #[default]
    Bar,
    Knob,
    None,
}

impl Fill {
    pub const ALL: [Self; 3] = [Self::Open, Self::Drawer, Self::Doors];
    pub fn name(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Drawer => "Drawer",
            Self::Doors => "Door",
        }
    }
}

impl Fronts {
    pub const ALL: [Self; 2] = [Self::Overlay, Self::Inset];
    pub fn name(self) -> &'static str {
        match self {
            Self::Overlay => "Overlay",
            Self::Inset => "Inset",
        }
    }
}

impl DoorStyle {
    pub const ALL: [Self; 2] = [Self::Slab, Self::FrameAndPanel];
    pub fn name(self) -> &'static str {
        match self {
            Self::Slab => "Slab",
            Self::FrameAndPanel => "Frame and panel",
        }
    }
}

impl Pulls {
    pub const ALL: [Self; 3] = [Self::Bar, Self::Knob, Self::None];
    pub fn name(self) -> &'static str {
        match self {
            Self::Bar => "Bar pulls",
            Self::Knob => "Knobs",
            Self::None => "None",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Drawer {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub joints: DrawerJoints,
    /// The box's stock.
    #[serde(default = "half")]
    pub stock: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FaceFrame {
    pub width: f64,
    pub height: f64,
    /// Stile width (the uprights).
    pub stile: f64,
    /// Rail width (top and bottom).
    pub rail: f64,
    pub joints: FrameJoints,
    #[serde(default = "three_quarters")]
    pub stock: f64,
    /// Rails between the top and bottom ones, evenly spaced.
    #[serde(default)]
    pub middle_rails: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub width: f64,
    pub depth: f64,
    /// To the top of the top.
    pub height: f64,
    pub joints: FrameJoints,
    /// The legs' square size.
    #[serde(default = "leg_size")]
    pub leg: f64,
    /// The aprons' width (they're 3/4" thick).
    #[serde(default = "apron_width")]
    pub apron: f64,
    /// The top's thickness.
    #[serde(default = "three_quarters")]
    pub top: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseJoints {
    /// Shelves in dados; the top and bottom in rabbets.
    Dados,
    Biscuits,
    PocketScrews,
    Butt,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawerJoints {
    #[default]
    Dovetails,
    BoxJoints,
    Rabbets,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameJoints {
    MortiseAndTenon,
    PocketScrews,
    Dowels,
}

impl CaseJoints {
    pub const ALL: [Self; 4] = [Self::Dados, Self::Biscuits, Self::PocketScrews, Self::Butt];
    pub fn name(self) -> &'static str {
        match self {
            Self::Dados => "Dados and rabbets",
            Self::Biscuits => "Biscuits",
            Self::PocketScrews => "Pocket screws",
            Self::Butt => "Butt joints",
        }
    }
}

impl DrawerJoints {
    pub const ALL: [Self; 3] = [Self::Dovetails, Self::BoxJoints, Self::Rabbets];
    pub fn name(self) -> &'static str {
        match self {
            Self::Dovetails => "Dovetails",
            Self::BoxJoints => "Box joints",
            Self::Rabbets => "Rabbets",
        }
    }
}

impl FrameJoints {
    pub const ALL: [Self; 3] = [Self::MortiseAndTenon, Self::PocketScrews, Self::Dowels];
    pub fn name(self) -> &'static str {
        match self {
            Self::MortiseAndTenon => "Mortise and tenon",
            Self::PocketScrews => "Pocket screws",
            Self::Dowels => "Dowels",
        }
    }
}

impl Default for Cabinet {
    fn default() -> Self {
        Self {
            width: 30.0,
            height: 36.0,
            depth: 12.0,
            shelves: 2,
            back: true,
            joints: CaseJoints::Dados,
            fills: [Fill::Open; MAX_OPENINGS],
            fronts: Fronts::Overlay,
            doors: DoorStyle::Slab,
            pulls: Pulls::Bar,
            drawer_joints: DrawerJoints::Dovetails,
            stock: STOCK,
            adjustable: false,
            hinge: HingeSide::Left,
        }
    }
}

impl Default for Drawer {
    fn default() -> Self {
        Self { width: 16.0, height: 5.0, depth: 14.0, joints: DrawerJoints::Dovetails, stock: DRAWER_STOCK }
    }
}

impl Default for FaceFrame {
    fn default() -> Self {
        Self { width: 30.0, height: 36.0, stile: 1.5, rail: 1.5, joints: FrameJoints::MortiseAndTenon, stock: STOCK, middle_rails: 0 }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self { width: 36.0, depth: 20.0, height: 30.0, joints: FrameJoints::MortiseAndTenon, leg: 1.75, apron: 4.0, top: STOCK }
    }
}

impl Template {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cabinet(_) => "Cabinet",
            Self::Drawer(_) => "Drawer",
            Self::FaceFrame(_) => "Face frame",
            Self::Table(_) => "Table",
        }
    }

    /// One of each, with starting sizes.
    pub fn all() -> [Self; 4] {
        [Self::Cabinet(Cabinet::default()), Self::Drawer(Drawer::default()), Self::FaceFrame(FaceFrame::default()), Self::Table(Table::default())]
    }

    /// Why these sizes can't be built, if they can't.
    pub fn check(&self) -> Result<(), String> {
        let small = |what: &str, min: f64| format!("The {what} must be over {}.", crate::units::format_length(min));
        match *self {
            Self::Cabinet(c) => {
                if !(0.25..=2.0).contains(&c.stock) {
                    return Err("The stock must be 1/4\" to 2\" thick.".into());
                }
                if c.width <= 2.0 * c.stock + 1.0 {
                    return Err(small("width", 2.0 * c.stock + 1.0));
                }
                if c.depth <= 2.0 {
                    return Err(small("depth", 2.0));
                }
                let room = c.height - 2.0 * c.stock - f64::from(c.shelves) * c.stock;
                if room <= f64::from(c.shelves + 1) {
                    return Err("Too many shelves for the height.".into());
                }
            }
            Self::Drawer(d) => {
                if !(0.25..=1.0).contains(&d.stock) {
                    return Err("The box stock must be 1/4\" to 1\" thick.".into());
                }
                if d.width <= 4.0 || d.depth <= 4.0 {
                    return Err("The width and depth must be over 4\".".into());
                }
                if d.height <= 2.0 {
                    return Err(small("height", 2.0));
                }
            }
            Self::FaceFrame(f) => {
                if f.stile <= 0.5 || f.rail <= 0.5 {
                    return Err("Stiles and rails must be over 1/2\" wide.".into());
                }
                if f.width <= 2.0 * f.stile + 1.0 {
                    return Err(small("width", 2.0 * f.stile + 1.0));
                }
                if f.height <= 2.0 * f.rail + 1.0 {
                    return Err(small("height", 2.0 * f.rail + 1.0));
                }
                if !(0.25..=2.0).contains(&f.stock) {
                    return Err("The stock must be 1/4\" to 2\" thick.".into());
                }
                if f.height <= f64::from(f.middle_rails + 2) * f.rail + f64::from(f.middle_rails + 1) {
                    return Err("Too many middle rails for the height.".into());
                }
            }
            Self::Table(t) => {
                if t.width <= 12.0 || t.depth <= 12.0 {
                    return Err("The width and depth must be over 12\".".into());
                }
                if t.height <= 10.0 {
                    return Err(small("height", 10.0));
                }
                if !(1.0..=6.0).contains(&t.leg) {
                    return Err("Legs must be 1\" to 6\" square.".into());
                }
                if t.apron < 1.5 {
                    return Err(small("apron width", 1.5));
                }
                if !(0.25..=3.0).contains(&t.top) {
                    return Err("The top must be 1/4\" to 3\" thick.".into());
                }
            }
        }
        Ok(())
    }
}

/// What a template group was made from, kept on its definition.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Made {
    pub template: Template,
    pub wood: Option<MaterialId>,
}

impl Document {
    /// Build `template` in `wood` beside the drawing, as one group and one undo step, and
    /// select it.
    pub fn add_template(&mut self, template: &Template, wood: Option<MaterialId>) -> Result<InstanceId, String> {
        template.check()?;
        // To the right of everything there is, a foot away.
        let to = self.bounds().map_or(DVec3::ZERO, |(lo, hi)| DVec3::new(hi.x + 12.0, lo.y, 0.0));
        let to = self.context_transform().inverse().transform_point3(to);
        self.as_one_step(|doc| {
            let group = doc.build(template, wood)?;
            doc.place(group, DAffine3::from_translation(to));
            Ok(group)
        })
    }

    /// Build `template` again in place of `instance` (a template group), in `wood`, as one undo
    /// step. Changes made inside it by hand are lost.
    pub fn rebuild_template(&mut self, instance: InstanceId, template: &Template, wood: Option<MaterialId>) -> Result<InstanceId, String> {
        template.check()?;
        self.template_of(instance).ok_or("That isn't made from a template.")?;
        let old = self.container().instance(instance).ok_or("That isn't in the group being edited.")?;
        let (transform, tag, material) = (old.transform, old.tag, old.material);
        self.as_one_step(|doc| {
            doc.erase(&[Entity::Instance(instance)]);
            let group = doc.build(template, wood)?;
            doc.place(group, transform);
            doc.edit_scene(|scene, path| {
                if let Some(i) = scene.container_mut(path).and_then(|c| c.instance_mut(group)) {
                    i.tag = tag;
                    i.material = material;
                }
            });
            Ok(group)
        })
    }

    /// What the group `instance` was made from, if it's a template.
    pub fn template_of(&self, instance: InstanceId) -> Option<Made> {
        let definition = self.container().instance(instance)?.definition;
        self.scene().definition(definition)?.template
    }

    /// Move the just-made `group` (its corner at the origin) by `transform`.
    fn place(&mut self, group: InstanceId, transform: DAffine3) {
        self.edit_scene(|scene, path| {
            if let Some(i) = scene.container_mut(path).and_then(|c| c.instance_mut(group)) {
                i.transform = transform * i.transform;
            }
        });
    }

    /// Build the parts at the origin and group them, remembering the template.
    fn build(&mut self, template: &Template, wood: Option<MaterialId>) -> Result<InstanceId, String> {
        let mut parts = Parts { doc: self, wood, made: Vec::new() };
        match *template {
            Template::Cabinet(c) => parts.cabinet(c),
            Template::Drawer(d) => parts.drawer(d, DVec3::ZERO),
            Template::FaceFrame(f) => parts.face_frame(f),
            Template::Table(t) => parts.table(t),
        }
        .map_err(|e| format!("Couldn't build the {}: {e}", template.name().to_lowercase()))?;
        let made = parts.made;
        self.selection_mut().clear();
        self.selection_mut().extend(made.into_iter().map(Entity::Instance));
        let group = self.group_selection(DefinitionKind::Group).ok_or("Nothing was built.")?;
        self.rename_instance(group, template.name());
        let record = Made { template: *template, wood };
        self.edit_scene(|scene, path| {
            let definition = scene.container(path).and_then(|c| c.instance(group)).map(|i| i.definition);
            if let Some(d) = definition.and_then(|d| scene.definition_mut(d)) {
                d.template = Some(record);
            }
        });
        let selection = self.selection_mut();
        selection.clear();
        selection.extend([Entity::Instance(group)]);
        Ok(group)
    }
}

/// Builds one template's parts, keeping the top-level ones made.
struct Parts<'a> {
    doc: &'a mut Document,
    wood: Option<MaterialId>,
    made: Vec<InstanceId>,
}

impl Parts<'_> {
    fn plywood(&mut self) -> MaterialId {
        let preset = LIBRARY.iter().find(|p| p.name == "Plywood").expect("Plywood is in the library");
        self.doc.add_preset(preset)
    }

    /// A board `[length, width, thickness]`: thickness runs along `length_dir × width_dir`.
    fn board(&mut self, name: &str, material: Option<MaterialId>, origin: DVec3, length_dir: DVec3, width_dir: DVec3, [length, width, thickness]: [f64; 3]) -> InstanceId {
        let id = self.doc.add_board(&Board { origin, length_dir, width_dir, length, width, thickness }, material).expect("templates check their sizes");
        self.doc.rename_instance(id, name);
        self.made.push(id);
        id
    }

    fn component(&mut self, part: InstanceId, name: &str) -> InstanceId {
        self.doc.selection_mut().clear();
        self.doc.selection_mut().extend([Entity::Instance(part)]);
        let id = self.doc.group_selection(DefinitionKind::Component).expect("a part");
        self.doc.rename_instance(id, name);
        id
    }

    /// Copies of `part` turned `degrees` about the vertical through `center`, then moved to
    /// each spot.
    fn copies(&mut self, part: InstanceId, center: DVec3, spots: &[(DVec3, f64)]) -> Vec<InstanceId> {
        let transforms: Vec<DAffine3> = spots
            .iter()
            .map(|&(to, degrees)| DAffine3::from_translation(to) * DAffine3::from_rotation_z(degrees.to_radians()) * DAffine3::from_translation(-center))
            .collect();
        let copies: Vec<InstanceId> = self
            .doc
            .transform_entities(&[Entity::Instance(part)], &transforms, true)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| if let Entity::Instance(i) = e { Some(i) } else { None })
            .collect();
        self.made.extend(&copies);
        copies
    }

    /// Join `part`'s end to `into` the way a frame or case joint is chosen.
    fn frame_joint(&mut self, part: InstanceId, into: InstanceId, joints: FrameJoints, tenon: Option<Tenon>) -> Result<(), JoineryError> {
        let fit = self.doc.joint_fit(part, into)?;
        let (thick, wide) = fit.contact;
        match joints {
            FrameJoints::MortiseAndTenon => self.doc.mortise_and_tenon(part, into, tenon.unwrap_or(fit.tenon)),
            FrameJoints::PocketScrews => self.doc.pocket_screws(part, into, pocket_count(wide), pocket_screw_for(thick)),
            FrameJoints::Dowels => {
                // Short enough that holes from two sides of a leg don't meet.
                let depth = if tenon.is_some() { fit.dowels.depth.min(0.875) } else { fit.dowels.depth };
                self.doc.dowels(part, into, Dowels { depth, ..fit.dowels })
            }
        }
    }

    fn cabinet(&mut self, c: Cabinet) -> Result<(), JoineryError> {
        let t = c.stock;
        let deep = if c.back { c.depth - PANEL } else { c.depth };
        let wood = self.wood;
        let left = self.board("Side", wood, DVec3::new(t, 0.0, 0.0), DVec3::Z, DVec3::Y, [c.height, deep, t]);
        let right = self.board("Side", wood, DVec3::new(c.width, 0.0, 0.0), DVec3::Z, DVec3::Y, [c.height, deep, t]);
        let inside = c.width - 2.0 * t;
        let mut across = vec![
            self.board("Bottom", wood, DVec3::new(t, 0.0, 0.0), DVec3::X, DVec3::Y, [inside, deep, t]),
            self.board("Top", wood, DVec3::new(t, 0.0, c.height - t), DVec3::X, DVec3::Y, [inside, deep, t]),
        ];
        let n = f64::from(c.shelves);
        let gap = (c.height - 2.0 * t - n * t) / (n + 1.0);
        // Adjustable shelves: loose, 1/32" short of each side, each on four shelf pins.
        let play = 1.0 / 32.0;
        let mut loose = Vec::new();
        for i in 0..c.shelves {
            let z = t + gap * f64::from(i + 1) + t * f64::from(i);
            if c.adjustable {
                self.board("Shelf", wood, DVec3::new(t + play, 0.0, z), DVec3::X, DVec3::Y, [inside - 2.0 * play, deep, t]);
                loose.push(z);
            } else {
                across.push(self.board("Shelf", wood, DVec3::new(t, 0.0, z), DVec3::X, DVec3::Y, [inside, deep, t]));
            }
        }
        for &part in &across {
            for side in [left, right] {
                let (thick, wide) = (t, deep);
                match c.joints {
                    CaseJoints::Dados => self.doc.let_in(part, side, t / 3.0).map(|_| ())?,
                    CaseJoints::Biscuits => self.doc.slots(part, side, &Slots::biscuits(Slots::suggested_biscuit(thick, wide), wide))?,
                    CaseJoints::PocketScrews => self.doc.pocket_screws(part, side, pocket_count(wide), pocket_screw_for(thick))?,
                    CaseJoints::Butt => {}
                }
            }
        }
        // The pins: 1/4" studs out of the sides under each loose shelf, 1 1/2" in from front and back.
        for z in loose {
            for x in [t, c.width - t - 0.375] {
                for y in [1.5, deep - 1.5 - 0.25] {
                    self.hardware("Shelf pin", crate::hardware::SHELF_PIN, &[(DVec3::new(x, y, z - 0.25), DVec3::X, DVec3::Y, [0.375, 0.25, 0.25])]);
                }
            }
        }
        if c.back {
            let plywood = Some(self.plywood());
            self.board("Back", plywood, DVec3::new(0.0, deep, c.height), DVec3::X, -DVec3::Z, [c.width, c.height, PANEL]);
        }
        self.openings(&c, deep)?;
        Ok(())
    }

    /// Group `parts` (already made) as one group named `name`, in their place among the made parts.
    fn group(&mut self, parts: &[InstanceId], name: &str) -> InstanceId {
        self.made.retain(|p| !parts.contains(p));
        self.doc.selection_mut().clear();
        self.doc.selection_mut().extend(parts.iter().map(|&p| Entity::Instance(p)));
        let group = self.doc.group_selection(DefinitionKind::Group).expect("parts to group");
        self.doc.rename_instance(group, name);
        self.made.push(group);
        group
    }

    /// A piece of hardware: plain steel boxes `[origin, length_dir, width_dir, [l, w, t]]`, grouped
    /// as `name` and counted in the shopping list as `piece`.
    fn hardware(&mut self, name: &str, piece: &str, boxes: &[(DVec3, DVec3, DVec3, [f64; 3])]) -> InstanceId {
        let steel = Some(self.doc.edit_scene(|scene, _| crate::hardware::steel(scene)));
        let parts: Vec<InstanceId> = boxes.iter().map(|&(origin, l, w, size)| self.board(name, steel, origin, l, w, size)).collect();
        let group = self.group(&parts, name);
        let piece = piece.to_owned();
        self.doc.edit_scene(|scene, path| {
            let definition = scene.container(path).and_then(|c| c.instance(group)).map(|i| i.definition);
            if let Some(d) = definition.and_then(|d| scene.definition_mut(d)) {
                d.hardware = Some(piece);
            }
        });
        group
    }

    /// Mark `group` as one that moves in its case (a drawer or door).
    fn mark_moving(&mut self, group: InstanceId) {
        self.doc.edit_scene(|scene, path| {
            let definition = scene.container(path).and_then(|c| c.instance(group)).map(|i| i.definition);
            if let Some(d) = definition.and_then(|d| scene.definition_mut(d)) {
                d.moves = true;
            }
        });
    }

    /// Fill the cabinet's openings (between the bottom, shelves and top) with drawers and doors.
    fn openings(&mut self, c: &Cabinet, deep: f64) -> Result<(), JoineryError> {
        let t = c.stock;
        let n = c.shelves as usize;
        let gap = (c.height - 2.0 * t - n as f64 * t) / (n as f64 + 1.0);
        for k in 0..=n.min(MAX_OPENINGS - 1) {
            let z0 = t + k as f64 * (gap + t);
            let opening = Opening { x0: t, x1: c.width - t, z0, z1: z0 + gap, deep, first: k == 0, last: k == n, case: t };
            match c.fills[k] {
                Fill::Open => {}
                Fill::Drawer => self.drawer_in(c, &opening)?,
                Fill::Doors => self.doors_in(c, &opening)?,
            }
        }
        Ok(())
    }

    /// A drawer in `o`: a box on side-mount slides (the longest that fit), a front, a pull.
    fn drawer_in(&mut self, c: &Cabinet, o: &Opening) -> Result<(), JoineryError> {
        let start = self.made.len();
        let front = o.front(c.fronts);
        // The box starts behind an inset front, at the cabinet's front edge behind an overlay one.
        let behind = if c.fronts == Fronts::Inset { STOCK } else { 0.0 };
        let length = crate::hardware::slide_for(o.deep - behind - 0.25).ok_or(JoineryError::Size("The cabinet is too shallow for a drawer (10\" slides at least)."))?;
        let width = o.x1 - o.x0 - 2.0 * crate::hardware::SLIDE_SPACE;
        let height = o.z1 - o.z0 - 1.0;
        if height < 2.0 || width < 4.0 {
            return Err(JoineryError::Size("An opening is too small for a drawer."));
        }
        let at = DVec3::new(o.x0 + crate::hardware::SLIDE_SPACE, behind, o.z0 + 0.5);
        self.drawer(Drawer { width, height, depth: length, joints: c.drawer_joints, stock: DRAWER_STOCK }, at)?;
        // The front: on the cabinet, or set in the opening.
        let wood = self.wood;
        self.board("Drawer front", wood, DVec3::new(front.x0, front.y0 + STOCK, front.z0), DVec3::X, DVec3::Z, [front.x1 - front.x0, front.z1 - front.z0, STOCK]);
        // Slides on each side of the box, at its middle.
        let mid = o.z0 + 0.5 + height / 2.0;
        let tall = 1.75_f64.min(height);
        let slide = |x: f64| (DVec3::new(x, behind, mid - tall / 2.0), DVec3::Y, DVec3::Z, [length, tall, crate::hardware::SLIDE_SPACE]);
        self.hardware("Drawer slides", &crate::hardware::slides_name(length), &[slide(o.x0), slide(o.x1 - crate::hardware::SLIDE_SPACE)]);
        // One pull in the middle; two, at the quarters, on a front over 24" wide.
        let (w, cz) = (front.x1 - front.x0, (front.z0 + front.z1) / 2.0);
        let spots: &[f64] = if w > 24.0 { &[0.25, 0.75] } else { &[0.5] };
        for &at in spots {
            self.pull(c.pulls, DVec3::new(front.x0 + w * at, front.y0, cz), true);
        }
        let parts: Vec<InstanceId> = self.made[start..].to_vec();
        let drawer = self.group(&parts, "Drawer");
        self.mark_moving(drawer);
        Ok(())
    }

    /// A door in `o`, or a pair when it's wider than 24": hinged on the cabinet's sides.
    fn doors_in(&mut self, c: &Cabinet, o: &Opening) -> Result<(), JoineryError> {
        let front = o.front(c.fronts);
        let leaves = if front.x1 - front.x0 > 24.0 {
            let middle = (front.x0 + front.x1) / 2.0;
            vec![(Rect { x1: middle - REVEAL / 2.0, ..front }, true), (Rect { x0: middle + REVEAL / 2.0, ..front }, false)]
        } else {
            vec![(front, c.hinge == HingeSide::Left)]
        };
        // Hinge arms reach in from the door's back to the cabinet side.
        let back = front.y0 + STOCK;
        for (leaf, hinged_left) in leaves {
            let start = self.made.len();
            let (w, h) = (leaf.x1 - leaf.x0, leaf.z1 - leaf.z0);
            match c.doors {
                DoorStyle::Slab => {
                    let wood = self.wood;
                    self.board("Door", wood, DVec3::new(leaf.x0, back, leaf.z0), DVec3::X, DVec3::Z, [w, h, STOCK]);
                }
                DoorStyle::FrameAndPanel => self.frame_and_panel(&leaf)?,
            }
            let count = crate::hardware::hinges_for(h);
            let arm = 2.0;
            let x = if hinged_left { o.x0 } else { o.x1 - arm };
            for i in 0..count {
                // 3" in from the top and bottom, the rest evenly between.
                let z = leaf.z0 + 3.0 + (h - 6.0 - 1.5) * i as f64 / (count - 1) as f64;
                self.hardware("Hinge", crate::hardware::HINGE, &[(DVec3::new(x, back, z + 1.5), DVec3::X, -DVec3::Z, [arm, 1.5, 0.5])]);
            }
            // The pull near the edge that opens.
            let x = if hinged_left { leaf.x1 - 1.5 } else { leaf.x0 + 1.5 };
            self.pull(c.pulls, DVec3::new(x, leaf.y0, (leaf.z0 + leaf.z1) / 2.0), false);
            let parts: Vec<InstanceId> = self.made[start..].to_vec();
            let door = self.group(&parts, "Door");
            self.mark_moving(door);
        }
        Ok(())
    }

    /// A frame-and-panel door filling `leaf`: stiles full height, rails between them stub-tenoned
    /// into the stiles, a 1/4" plywood panel let into grooves all round.
    fn frame_and_panel(&mut self, leaf: &Rect) -> Result<(), JoineryError> {
        let (w, h) = (leaf.x1 - leaf.x0, leaf.z1 - leaf.z0);
        let s = 2.25_f64.min(w / 4.0).min(h / 4.0);
        let wood = self.wood;
        let y = leaf.y0;
        let left = self.board("Door stile", wood, DVec3::new(leaf.x0, y, leaf.z0), DVec3::Z, DVec3::X, [h, s, STOCK]);
        let right = self.board("Door stile", wood, DVec3::new(leaf.x1 - s, y, leaf.z0), DVec3::Z, DVec3::X, [h, s, STOCK]);
        let bottom = self.board("Door rail", wood, DVec3::new(leaf.x0 + s, y + STOCK, leaf.z0), DVec3::X, DVec3::Z, [w - 2.0 * s, s, STOCK]);
        let top = self.board("Door rail", wood, DVec3::new(leaf.x0 + s, y + STOCK, leaf.z1 - s), DVec3::X, DVec3::Z, [w - 2.0 * s, s, STOCK]);
        let groove = 0.375;
        for rail in [bottom, top] {
            for stile in [left, right] {
                // Stub tenons, as deep as the panel groove, set in 1/2" from the rail's edges: the
                // groove (which reaches 3/8" past the rail's edge) stays clear of the mortise.
                self.doc.mortise_and_tenon(rail, stile, Tenon { length: groove, thickness: PANEL, shoulder: 0.5, haunch: 0.0, fit: 0.0 })?;
            }
        }
        let plywood = Some(self.plywood());
        let panel = self.board(
            "Door panel",
            plywood,
            DVec3::new(leaf.x0 + s, y + (STOCK + PANEL) / 2.0, leaf.z0 + s),
            DVec3::X,
            DVec3::Z,
            [w - 2.0 * s, h - 2.0 * s, PANEL],
        );
        for into in [bottom, top, left, right] {
            self.doc.let_in(panel, into, groove)?;
        }
        Ok(())
    }

    /// A bar pull (5", along the front for a drawer, upright for a door) or a knob, centered on
    /// `at` on a front whose face is at `at.y`.
    fn pull(&mut self, pulls: Pulls, at: DVec3, across: bool) {
        match pulls {
            Pulls::Bar if across => {
                self.hardware("Pull", crate::hardware::PULL, &[(at + DVec3::new(-2.5, 0.0, -0.25), DVec3::X, DVec3::Z, [5.0, 0.5, 1.0])]);
            }
            Pulls::Bar => {
                self.hardware("Pull", crate::hardware::PULL, &[(at + DVec3::new(0.25, 0.0, -2.5), DVec3::Z, -DVec3::X, [5.0, 0.5, 1.0])]);
            }
            Pulls::Knob => {
                self.hardware("Knob", crate::hardware::KNOB, &[(at + DVec3::new(-0.5, 0.0, -0.5), DVec3::X, DVec3::Z, [1.0, 1.0, 1.0])]);
            }
            Pulls::None => {}
        }
    }

    /// A drawer box with its front-left-bottom corner at `at`.
    fn drawer(&mut self, d: Drawer, at: DVec3) -> Result<(), JoineryError> {
        let t = d.stock;
        let wood = self.wood;
        // The front and back run the full width; the sides butt their insides, and the joints
        // carry them out to the corners.
        let front = self.board("Drawer box front", wood, at + DVec3::new(0.0, t, 0.0), DVec3::X, DVec3::Z, [d.width, d.height, t]);
        let back = self.board("Drawer box back", wood, at + DVec3::new(d.width, d.depth - t, 0.0), -DVec3::X, DVec3::Z, [d.width, d.height, t]);
        let left = self.board("Drawer box side", wood, at + DVec3::new(t, t, d.height), DVec3::Y, -DVec3::Z, [d.depth - 2.0 * t, d.height, t]);
        let right = self.board("Drawer box side", wood, at + DVec3::new(d.width - t, d.depth - t, d.height), -DVec3::Y, -DVec3::Z, [d.depth - 2.0 * t, d.height, t]);
        // The bottom, 1/2" up, between the sides, front and back; let into each 1/4".
        let plywood = Some(self.plywood());
        let up = 0.5;
        let bottom = self.board("Drawer bottom", plywood, at + DVec3::new(t, t, up), DVec3::X, DVec3::Y, [d.width - 2.0 * t, d.depth - 2.0 * t, PANEL]);
        for side in [left, right] {
            for end in [front, back] {
                match d.joints {
                    DrawerJoints::Dovetails => {
                        let tails = Dovetails::suggested(d.height);
                        self.doc.dovetails(side, end, Dovetails::new(tails, false))?;
                    }
                    DrawerJoints::BoxJoints => {
                        let finger = self.doc.joint_fit(side, end)?.finger;
                        self.doc.box_joint(side, end, finger)?;
                    }
                    DrawerJoints::Rabbets => self.doc.let_in(side, end, t / 2.0).map(|_| ())?,
                }
            }
        }
        // The bottom's grooves after the corners: the front and back first (stopping short of
        // the corner joints), then the sides, through their tails.
        for into in [front, back, left, right] {
            self.doc.let_in(bottom, into, PANEL).map(|_| ())?;
        }
        Ok(())
    }

    fn face_frame(&mut self, f: FaceFrame) -> Result<(), JoineryError> {
        let t = f.stock;
        let wood = self.wood;
        let left = self.board("Stile", wood, DVec3::ZERO, DVec3::Z, DVec3::X, [f.height, f.stile, t]);
        let right = self.board("Stile", wood, DVec3::new(f.width - f.stile, 0.0, 0.0), DVec3::Z, DVec3::X, [f.height, f.stile, t]);
        let long = f.width - 2.0 * f.stile;
        let bottom = self.board("Bottom rail", wood, DVec3::new(f.stile, t, 0.0), DVec3::X, DVec3::Z, [long, f.rail, t]);
        let top = self.board("Top rail", wood, DVec3::new(f.stile, t, f.height - f.rail), DVec3::X, DVec3::Z, [long, f.rail, t]);
        let mut rails = vec![bottom, top];
        // Middle rails, evenly spaced between the top and bottom ones.
        let n = f64::from(f.middle_rails);
        let gap = (f.height - (n + 2.0) * f.rail) / (n + 1.0);
        for i in 0..f.middle_rails {
            let z = f.rail + gap * f64::from(i + 1) + f.rail * f64::from(i);
            rails.push(self.board("Middle rail", wood, DVec3::new(f.stile, t, z), DVec3::X, DVec3::Z, [long, f.rail, t]));
        }
        for rail in rails {
            for stile in [left, right] {
                self.frame_joint(rail, stile, f.joints, None)?;
            }
        }
        Ok(())
    }

    fn table(&mut self, t: Table) -> Result<(), JoineryError> {
        let wood = self.wood;
        let (leg, apron, thick, overhang, set_back) = (t.leg, t.apron.min((t.height - t.top) / 3.0), STOCK, 1.0, 0.25);
        let legs_up = t.height - t.top;
        // One leg, made a component and turned to each corner so its inside faces (where the
        // joints go) always face in.
        let first = self.board("Leg", wood, DVec3::new(overhang, overhang, 0.0), DVec3::Z, DVec3::X, [legs_up, leg, leg]);
        self.made.pop();
        let front_left = self.component(first, "Leg");
        self.made.push(front_left);
        let c = overhang + leg / 2.0;
        let others = self.copies(front_left, DVec3::new(c, c, 0.0), &[
            (DVec3::new(t.width - c, c, 0.0), 90.0),
            (DVec3::new(t.width - c, t.depth - c, 0.0), 180.0),
            (DVec3::new(c, t.depth - c, 0.0), 270.0),
        ]);
        let [front_right, back_right, back_left] = others[..] else { return Err(JoineryError::NotParts) };

        // Aprons set back from the legs' outside faces; the back a copy of the front turned
        // around, the right a copy of the left.
        let middle = DVec3::new(t.width / 2.0, t.depth / 2.0, 0.0);
        let inner = overhang + leg;
        let front = self.board("Front apron", wood, DVec3::new(inner, overhang + set_back, legs_up), DVec3::X, -DVec3::Z, [t.width - 2.0 * inner, apron, thick]);
        self.made.pop();
        let front = self.component(front, "Front and back apron");
        self.made.push(front);
        let back = self.copies(front, middle, &[(middle, 180.0)])[0];
        let left = self.board("Side apron", wood, DVec3::new(overhang + set_back + thick, inner, legs_up), DVec3::Y, -DVec3::Z, [t.depth - 2.0 * inner, apron, thick]);
        self.made.pop();
        let left = self.component(left, "Side apron");
        self.made.push(left);
        let right = self.copies(left, middle, &[(middle, 180.0)])[0];

        // Tenons short enough that the two mortises in each leg don't meet (3/4" in a 1 3/4" leg).
        let tenon = Tenon { length: (leg - 1.0).clamp(0.5, 1.5), thickness: 0.25, shoulder: 0.25, haunch: 0.0, fit: 0.0 };
        for (apron, leg) in [
            (front, front_left),
            (front, front_right),
            (back, back_right),
            (back, back_left),
            (left, front_left),
            (left, back_left),
            (right, front_right),
            (right, back_right),
        ] {
            self.frame_joint(apron, leg, t.joints, Some(tenon))?;
        }
        self.board("Table top", wood, DVec3::new(0.0, 0.0, legs_up), DVec3::X, DVec3::Y, [t.width, t.depth, t.top]);
        Ok(())
    }
}

/// The gap between fronts, and around an inset front: 1/16".
const REVEAL: f64 = 1.0 / 16.0;

/// A rectangle on a cabinet's front: across `x0..x1`, up `z0..z1`, its front face at `y0` (a
/// door or drawer front is `y0..y0 + 3/4"`).
#[derive(Clone, Copy, Debug)]
struct Rect {
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    y0: f64,
}

/// An opening between a cabinet's sides and the parts above and below; `first` and `last` are
/// the bottom and top ones (with the cabinet's bottom or top for an edge).
struct Opening {
    x0: f64,
    x1: f64,
    z0: f64,
    z1: f64,
    deep: f64,
    first: bool,
    last: bool,
    /// The case's stock (how much an overlay front covers).
    case: f64,
}

impl Opening {
    /// Where its front goes. Overlay: over the cabinet's sides, top and bottom (to 1/16" of their
    /// outer edges) and halfway over a shelf shared with the next opening; inset: inside it.
    fn front(&self, fronts: Fronts) -> Rect {
        let t = self.case;
        match fronts {
            Fronts::Overlay => {
                let edge = t - REVEAL;
                let shared = t / 2.0 - REVEAL / 2.0;
                Rect {
                    x0: self.x0 - edge,
                    x1: self.x1 + edge,
                    z0: self.z0 - if self.first { edge } else { shared },
                    z1: self.z1 + if self.last { edge } else { shared },
                    y0: -t,
                }
            }
            Fronts::Inset => Rect { x0: self.x0 + REVEAL, x1: self.x1 - REVEAL, z0: self.z0 + REVEAL, z1: self.z1 - REVEAL, y0: 0.0 },
        }
    }
}

/// About one pocket screw every 4", at least two.
fn pocket_count(wide: f64) -> u32 {
    ((wide / 4.0).round() as u32).max(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closed(doc: &Document, part: InstanceId, path: &[InstanceId]) -> bool {
        let container = doc.scene().container(path).unwrap();
        let def = container.instance(part).unwrap().definition;
        let m = &doc.scene().definition(def).unwrap().container.model;
        m.faces().count() > 0 && m.edges().all(|(id, _)| m.edge_faces(id).len() == 2)
    }

    /// Every board in the container at `path`, and in the groups inside it, is a closed solid.
    fn every_board_closed(doc: &Document, path: &[InstanceId]) {
        let container = doc.scene().container(path).unwrap();
        for (id, instance) in container.instances() {
            let definition = doc.scene().definition(instance.definition).unwrap();
            if definition.container.instances().next().is_some() {
                every_board_closed(doc, &[path, &[id]].concat());
            } else {
                assert!(closed(doc, id, path), "{} is a closed solid", definition.name);
            }
        }
    }

    /// Every part inside the template group is a closed solid.
    fn all_closed(doc: &Document, group: InstanceId) -> usize {
        let path = [group];
        let container = doc.scene().container(&path).unwrap();
        let parts: Vec<InstanceId> = container.instances().map(|(id, _)| id).collect();
        for &p in &parts {
            assert!(closed(doc, p, &path), "{:?} is a closed solid", doc.scene().definition(container.instance(p).unwrap().definition).unwrap().name);
        }
        parts.len()
    }

    fn cut_list_names(doc: &Document) -> Vec<(String, usize)> {
        let list = crate::cutlist::cut_list(doc.scene());
        let mut names: Vec<(String, usize)> = list.materials.iter().flat_map(|m| m.parts.iter().map(|p| (p.names.join(", "), p.count))).collect();
        names.sort();
        names
    }

    #[test]
    fn every_template_builds_with_every_joint() {
        let mut templates = Vec::new();
        for joints in CaseJoints::ALL {
            templates.push(Template::Cabinet(Cabinet { joints, ..Default::default() }));
        }
        for joints in DrawerJoints::ALL {
            templates.push(Template::Drawer(Drawer { joints, ..Default::default() }));
        }
        for joints in FrameJoints::ALL {
            templates.push(Template::FaceFrame(FaceFrame { joints, ..Default::default() }));
            templates.push(Template::Table(Table { joints, ..Default::default() }));
        }
        for template in templates {
            let mut doc = Document::default();
            let group = doc.add_template(&template, None).unwrap_or_else(|e| panic!("{template:?}: {e}"));
            all_closed(&doc, group);
            assert!(doc.undo(), "{template:?} is one undo step");
            assert!(doc.container().instances().next().is_none(), "{template:?}: undone, nothing is left");
        }
    }

    #[test]
    fn a_cabinet_lists_its_parts() {
        let mut doc = Document::default();
        let group = doc.add_template(&Template::Cabinet(Cabinet::default()), None).unwrap();
        assert_eq!(all_closed(&doc, group), 7, "two sides, top, bottom, two shelves, back");
        assert_eq!(doc.instance_name(group), Some("Cabinet"));
        let names = cut_list_names(&doc);
        assert!(names.contains(&("Side".into(), 2)), "{names:?}");
        assert!(names.contains(&("Bottom, Top, Shelf".into(), 4)), "same size: {names:?}");
        // Let in 1/4" at each end: 28 1/2" between the sides, 29" long.
        let shelf = doc.scene().container(&[group]).unwrap().instances().find(|(_, i)| doc.scene().definition(i.definition).unwrap().name == "Shelf").unwrap().0;
        doc.enter(group);
        assert_eq!(doc.part_size(shelf).unwrap()[0], 29.0);
        assert!(!doc.joint_list(shelf).is_empty());
    }

    /// A cabinet with a drawer in the bottom opening and doors in the next, each style.
    fn fitted(fronts: Fronts, doors: DoorStyle, pulls: Pulls, width: f64) -> Cabinet {
        let mut fills = [Fill::Open; MAX_OPENINGS];
        fills[0] = Fill::Drawer;
        fills[1] = Fill::Doors;
        Cabinet { width, height: 30.0, depth: 20.0, shelves: 1, fills, fronts, doors, pulls, ..Default::default() }
    }

    #[test]
    fn drawers_and_doors_fill_openings_in_every_style() {
        for fronts in Fronts::ALL {
            for doors in DoorStyle::ALL {
                for (pulls, width) in [(Pulls::Bar, 20.0), (Pulls::Knob, 36.0), (Pulls::None, 30.0)] {
                    let c = fitted(fronts, doors, pulls, width);
                    let mut doc = Document::default();
                    let group = doc.add_template(&Template::Cabinet(c), None).unwrap_or_else(|e| panic!("{fronts:?} {doors:?} {width}: {e}"));
                    // Every board, down through the drawer and door groups, is a closed solid.
                    every_board_closed(&doc, &[group]);
                    let hardware: std::collections::BTreeMap<String, u32> = crate::hardware::placed(doc.scene());
                    assert_eq!(hardware.get("18\" side-mount drawer slides (pair)"), Some(&1), "{hardware:?}");
                    let leaves = if width > 24.0 { 2 } else { 1 };
                    assert_eq!(hardware.get(crate::hardware::HINGE), Some(&(2 * leaves)), "{width}: {hardware:?}");
                    let pulls_count = match pulls { Pulls::Bar => hardware.get(crate::hardware::PULL), Pulls::Knob => hardware.get(crate::hardware::KNOB), Pulls::None => None };
                    // Two pulls on a drawer front over 24" wide, one on each door.
                    let drawer_pulls = if width - 1.0 / 8.0 > 24.0 { 2 } else { 1 };
                    assert_eq!(pulls_count.copied(), if pulls == Pulls::None { None } else { Some(drawer_pulls + leaves) });
                    // Hardware isn't a part to cut.
                    let names = cut_list_names(&doc);
                    assert!(!names.iter().any(|(n, _)| n.contains("slide") || n.contains("Hinge") || n.contains("Pull")), "{names:?}");
                }
            }
        }
    }

    #[test]
    fn adjustable_shelves_sit_loose_on_pins() {
        let mut doc = Document::default();
        let c = Cabinet { shelves: 2, adjustable: true, ..Default::default() };
        doc.add_template(&Template::Cabinet(c), None).unwrap();
        assert_eq!(crate::hardware::placed(doc.scene()).get(crate::hardware::SHELF_PIN), Some(&8), "four pins under each of two shelves");
        let list = crate::cutlist::cut_list(doc.scene());
        let shelf = list.materials.iter().flat_map(|m| &m.parts).find(|p| p.names.iter().any(|n| n == "Shelf")).unwrap();
        // 30" less two 3/4" sides, less 1/32" play each side; not let in.
        assert_eq!(shelf.length, 28.5 - 1.0 / 16.0);
        assert!(doc.scene().joints.iter().all(|j| doc.scene().definition(j.cut).unwrap().name != "Shelf"), "loose shelves aren't joined");
    }

    #[test]
    fn templates_take_their_stock_and_sizes() {
        let mut doc = Document::default();
        let c = Cabinet { stock: 1.0, ..Default::default() };
        doc.add_template(&Template::Cabinet(c), None).unwrap();
        let thick = |doc: &Document, name: &str| {
            let list = crate::cutlist::cut_list(doc.scene());
            list.materials.iter().flat_map(|m| m.parts.clone()).find(|p| p.names.iter().any(|n| n == name)).map(|p| p.thickness)
        };
        assert_eq!(thick(&doc, "Side"), Some(1.0));
        let mut doc = Document::default();
        doc.add_template(&Template::Table(Table { leg: 2.5, apron: 5.0, top: 1.0, ..Default::default() }), None).unwrap();
        assert_eq!(thick(&doc, "Table top"), Some(1.0));
        let (lo, hi) = doc.bounds().unwrap();
        assert!((hi.z - lo.z - 30.0).abs() < 1e-9, "still 30\" to the top");
        let mut doc = Document::default();
        doc.add_template(&Template::FaceFrame(FaceFrame { middle_rails: 2, ..Default::default() }), None).unwrap();
        let names = cut_list_names(&doc);
        assert!(names.iter().any(|(n, c)| n.contains("Middle rail") && *c >= 2) || names.iter().any(|(n, _)| n.contains("Middle rail")), "{names:?}");
    }

    #[test]
    fn a_drawer_is_sized_from_its_opening() {
        let mut doc = Document::default();
        doc.add_template(&Template::Cabinet(fitted(Fronts::Overlay, DoorStyle::Slab, Pulls::Bar, 20.0)), None).unwrap();
        let names = cut_list_names(&doc);
        // The opening is 18 1/2" wide (20 less two 3/4" sides): the box is 1" narrower for the slides.
        let list = crate::cutlist::cut_list(doc.scene());
        let box_front = list.materials.iter().flat_map(|m| &m.parts).find(|p| p.names.iter().any(|n| n == "Drawer box front")).unwrap_or_else(|| panic!("{names:?}"));
        assert_eq!(box_front.length, 17.5);
        // The overlay front covers the sides to 1/16" of their edges: 20" less 1/8".
        let front = list.materials.iter().flat_map(|m| &m.parts).find(|p| p.names.iter().any(|n| n == "Drawer front")).unwrap();
        assert_eq!(front.length, 19.875);
    }

    #[test]
    fn a_drawer_keeps_its_outside_size() {
        let mut doc = Document::default();
        let group = doc.add_template(&Template::Drawer(Drawer::default()), None).unwrap();
        let (lo, hi) = doc.bounds().unwrap();
        assert!((hi - lo - DVec3::new(16.0, 14.0, 5.0)).abs().max_element() < 1e-9, "{:?}", hi - lo);
        assert_eq!(all_closed(&doc, group), 5);
    }

    #[test]
    fn a_table_shares_its_leg() {
        let mut doc = Document::default();
        doc.add_template(&Template::Table(Table::default()), None).unwrap();
        let names = cut_list_names(&doc);
        assert!(names.contains(&("Leg".into(), 4)), "{names:?}");
        assert!(names.contains(&("Front and back apron".into(), 2)), "{names:?}");
        let (lo, hi) = doc.bounds().unwrap();
        assert!((hi - lo - DVec3::new(36.0, 20.0, 30.0)).abs().max_element() < 1e-9, "{:?}", hi - lo);
    }

    #[test]
    fn templates_go_beside_the_drawing_and_change_size_in_place() {
        let mut doc = Document::default();
        let first = doc.add_template(&Template::FaceFrame(FaceFrame::default()), None).unwrap();
        let second = doc.add_template(&Template::Cabinet(Cabinet::default()), None).unwrap();
        let x = |doc: &Document, id| doc.container().instance(id).unwrap().transform.translation.x;
        assert_eq!(x(&doc, second), 42.0, "a foot to the right of the face frame");
        // Moved, then made taller: it stays where it was moved.
        doc.edit_scene(|scene, path| scene.container_mut(path).unwrap().instance_mut(first).unwrap().transform.translation.y = 5.0);
        let Some(Made { template: Template::FaceFrame(f), .. }) = doc.template_of(first) else { panic!("a face frame") };
        let taller = doc.rebuild_template(first, &Template::FaceFrame(FaceFrame { height: 40.0, ..f }), None).unwrap();
        assert_eq!(doc.container().instance(taller).unwrap().transform.translation.y, 5.0);
        assert!(doc.container().instance(first).is_none());
        let Some(Made { template: Template::FaceFrame(f), .. }) = doc.template_of(taller) else { panic!("still a face frame") };
        assert_eq!(f.height, 40.0);
        assert!(doc.undo());
        assert!(doc.container().instance(first).is_some(), "one undo step brings the old one back");
    }

    #[test]
    fn sizes_that_cant_be_built_say_why() {
        let mut doc = Document::default();
        let err = doc.add_template(&Template::Cabinet(Cabinet { height: 6.0, shelves: 5, ..Default::default() }), None).unwrap_err();
        assert!(err.contains("shelves"), "{err}");
        assert!(doc.container().instances().next().is_none());
    }
}#[cfg(test)]
mod probe {
    use super::*;
    fn closed(doc: &Document, part: InstanceId) -> bool {
        let def = doc.container().instance(part).unwrap().definition;
        let m = &doc.scene().definition(def).unwrap().container.model;
        m.edges().all(|(id, _)| m.edge_faces(id).len() == 2)
    }
    #[test]
    fn probe_frame_and_panel() {
        for (tenons_first, shoulder) in [(true, 0.5), (false, 0.5)] {
            let mut doc = Document::default();
            let mut p = Parts { doc: &mut doc, wood: None, made: Vec::new() };
            let (x0, x1, z0, z1, y) = (0.0, 15.0, 0.0, 20.0, -0.75);
            let (w, h, s) = (x1 - x0, z1 - z0, 2.25);
            let left = p.board("S", None, DVec3::new(x0, y, z0), DVec3::Z, DVec3::X, [h, s, STOCK]);
            let right = p.board("S", None, DVec3::new(x1 - s, y, z0), DVec3::Z, DVec3::X, [h, s, STOCK]);
            let bottom = p.board("R", None, DVec3::new(x0 + s, y + STOCK, z0), DVec3::X, DVec3::Z, [w - 2.0 * s, s, STOCK]);
            let top = p.board("R", None, DVec3::new(x0 + s, y + STOCK, z1 - s), DVec3::X, DVec3::Z, [w - 2.0 * s, s, STOCK]);
            let panel = p.board("P", None, DVec3::new(x0 + s, y + 0.5, z0 + s), DVec3::X, DVec3::Z, [w - 2.0 * s, h - 2.0 * s, PANEL]);
            let mut log = Vec::new();
            let tenons = |doc: &mut Document, log: &mut Vec<String>| for rail in [bottom, top] { for stile in [left, right] {
                let r = doc.mortise_and_tenon(rail, stile, Tenon { length: 0.375, thickness: PANEL, shoulder, haunch: 0.0, fit: 0.0 });
                log.push(format!("mt {:?} L{}", r.err(), closed(doc, left)));
            }};
            let grooves = |doc: &mut Document, log: &mut Vec<String>| for into in [bottom, top, left, right] {
                let r = doc.let_in(panel, into, 0.375);
                log.push(format!("groove {:?} L{}", r.err(), closed(doc, left)));
            };
            if tenons_first { tenons(&mut doc, &mut log); grooves(&mut doc, &mut log); } else { grooves(&mut doc, &mut log); tenons(&mut doc, &mut log); }
            println!("tenons_first={tenons_first} shoulder={shoulder}: {log:?}");
        }
    }
}
