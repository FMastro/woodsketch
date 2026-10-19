//! Assembly instructions: for each assembly in the drawing (a group of parts, like a table, or a
//! cabinet's drawers and doors), the order to put it together, and an exploded drawing.
//!
//! The order comes from the joints. It starts with the part joined to the most others, then
//! adds, one at a time, the part joined most to what's already together, saying which joints
//! hold it. Parts that only touch (a top resting on aprons, a back) are fastened after the glue-up;
//! drawers are fitted and doors hung last. Sub-assemblies (a drawer inside a cabinet) come first,
//! each on its own.
//!
//! Hardware is said where it's used: hinges and pulls on a door as it's hung, slides mounted on a
//! drawer box's sides once the glue's dry, a pull after its front is on.
//!
//! The exploded drawing pulls each part away from the assembly's middle and draws it from the
//! front-left, above (like the Iso view), its faces filled and sorted back to front, each part
//! labeled with its cut-list letter (a drawer or door with its name) in a column beside the
//! drawing, with a leader line to a spot on it you can see.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use glam::{DAffine3, DVec3};

use crate::cutlist::CutList;
use crate::{Container, DefinitionId, MaterialId, Scene};

/// One assembly: its name, its steps in order, and an exploded drawing (SVG).
#[derive(Clone, Debug)]
pub struct Assembly {
    pub name: String,
    pub steps: Vec<String>,
    pub svg: String,
}

/// A part or sub-assembly in an assembly.
struct Member {
    /// What to call it in a step: `B (Front apron)`.
    name: String,
    /// Every definition in it (one for a part; a sub-assembly's parts).
    definitions: BTreeSet<DefinitionId>,
    /// Each geometry in it and where it's placed in the world (to find a joint's cuts).
    places: Vec<(DefinitionId, DAffine3)>,
    /// Its box in the world.
    lo: DVec3,
    hi: DVec3,
    /// A drawer or door: fitted or hung, not glued.
    moves: bool,
    /// A board of solid wood wider than 6": fastened so it can move.
    wide_solid: bool,
    /// Faces to draw: each a loop of world points and the face's side, with a color.
    faces: Vec<(Vec<DVec3>, DVec3, [u8; 3])>,
    label: Option<String>,
    /// Hardware mounted on it: what touches it in the assembly, and, for a sub-assembly without
    /// a page of its own (a slab door), what's in it.
    hardware: BTreeMap<String, u32>,
    /// Hardware mounted on it and on other members too (a pair of slides on both box sides): the
    /// others' names, by piece.
    also_on: BTreeMap<String, Vec<String>>,
}

impl Member {
    fn new(name: String, moves: bool) -> Self {
        Self {
            name,
            definitions: BTreeSet::new(),
            places: Vec::new(),
            lo: DVec3::splat(f64::INFINITY),
            hi: DVec3::splat(f64::NEG_INFINITY),
            moves,
            wide_solid: false,
            faces: Vec::new(),
            label: None,
            hardware: BTreeMap::new(),
            also_on: BTreeMap::new(),
        }
    }
}

/// Every assembly in the drawing, sub-assemblies before the ones they're in.
pub fn assemblies(scene: &Scene, list: &CutList) -> Vec<Assembly> {
    let mut out = Vec::new();
    let mut paged = BTreeSet::new();
    walk(scene, list, &scene.root, "The whole piece", DAffine3::IDENTITY, None, (&mut out, &mut paged), 0);
    out
}

/// Adds the container's assembly page (after its sub-assemblies'), returning whether it got one.
/// `paged` collects the definitions that got pages.
#[allow(clippy::too_many_arguments)]
fn walk(scene: &Scene, list: &CutList, container: &Container, name: &str, world: DAffine3, paint: Option<MaterialId>, (out, paged): (&mut Vec<Assembly>, &mut BTreeSet<DefinitionId>), depth: usize) -> bool {
    if depth > 32 {
        return false;
    }
    // Sub-assemblies first.
    for (_, instance) in container.instances() {
        let Some(def) = scene.definition(instance.definition) else { continue };
        if scene.is_visible(instance)
            && def.hardware.is_none()
            && def.container.instances().count() >= 1
            && !paged.contains(&instance.definition)
            && walk(scene, list, &def.container, &def.name, world * instance.transform, instance.material.or(paint), (out, paged), depth + 1)
        {
            paged.insert(instance.definition);
        }
    }
    let mut members: Vec<Member> = container
        .instances()
        .filter(|(_, i)| scene.is_visible(i))
        .filter_map(|(_, i)| member(scene, list, i, world, paint, paged))
        .collect();
    // An assembly is parts that meet: separate pieces side by side aren't one.
    let meet = (0..members.len()).any(|i| (0..members.len()).any(|j| i != j && touching(&members[i], &members[j])));
    if members.len() < 2 || !meet {
        return false;
    }
    // Each piece of hardware goes with what it's mounted on: a drawer or door it touches, else the
    // part it touches most.
    let mut loose = BTreeMap::new();
    for (_, instance) in container.instances().filter(|(_, i)| scene.is_visible(i)) {
        let Some(piece) = scene.definition(instance.definition).and_then(|d| d.hardware.clone()) else { continue };
        let mut placed = Member::new(String::new(), false);
        gather(scene, instance.definition, world * instance.transform, None, &mut placed, 0);
        let touching: Vec<(usize, bool, f64)> =
            members.iter().enumerate().map(|(i, m)| (i, m.moves, against(m, &placed))).filter(|(_, _, area)| *area > 0.0).collect();
        let on = touching.iter().copied().max_by(|a, b| a.1.cmp(&b.1).then(a.2.total_cmp(&b.2)));
        let Some((i, moves, area)) = on else {
            *loose.entry(piece).or_default() += 1;
            continue;
        };
        // Mounted against others about as much (each slide of a pair on its own box side).
        let others: Vec<String> = touching.iter().filter(|&&(j, m, a)| j != i && m == moves && a >= area * 0.5).map(|&(j, _, _)| members[j].name.clone()).collect();
        if !others.is_empty() {
            members[i].also_on.insert(piece.clone(), others);
        }
        *members[i].hardware.entry(piece).or_default() += 1;
    }
    out.push(Assembly { name: name.to_owned(), steps: steps(scene, &members, &loose), svg: exploded(&members) });
    true
}

/// A part or sub-assembly of an assembly (hardware isn't one).
fn member(scene: &Scene, list: &CutList, instance: &crate::Instance, world: DAffine3, paint: Option<MaterialId>, paged: &BTreeSet<DefinitionId>) -> Option<Member> {
    let def = scene.definition(instance.definition)?;
    if def.hardware.is_some() {
        return None;
    }
    let place = world * instance.transform;
    let paint = instance.material.or(paint);
    let mut m = Member::new(def.name.clone(), def.moves);
    // A sub-assembly without its own page: its hardware is mounted when it goes on.
    if !paged.contains(&instance.definition) {
        m.hardware = hardware_in(scene, &def.container);
    }
    gather(scene, instance.definition, place, paint, &mut m, 0);
    if m.faces.is_empty() {
        return None;
    }
    // A single part: its cut-list letter, and whether it's a wide solid-wood board.
    if def.container.instances().next().is_none() {
        let linear = place.matrix3;
        let material = paint.or_else(|| def.container.model.faces().find_map(|(_, f)| f.material));
        if let Some(extents) = crate::part::extents(def, linear) {
            let [length, width, thickness] = crate::part::size(extents, &def.part);
            m.label = label(list, material, [length, width, thickness]);
            m.wide_solid = width > 6.0 && material.and_then(|id| scene.material(id)).is_some_and(|mat| mat.kind == crate::MaterialKind::SolidWood);
        }
    }
    if let Some(label) = &m.label {
        m.name = format!("{label} ({})", def.name);
    }
    Some(m)
}

/// The cut-list letter of a part this size in this material.
fn label(list: &CutList, material: Option<MaterialId>, [length, width, thickness]: [f64; 3]) -> Option<String> {
    let close = |a: f64, b: f64| (a - b).abs() < 1.0 / 64.0;
    list.materials
        .iter()
        .filter(|m| m.material == material)
        .flat_map(|m| &m.parts)
        .find(|p| close(p.length, length) && close(p.width, width) && close(p.thickness, thickness))
        .map(|p| p.label.clone())
}

/// A definition's faces (and its nested parts'), placed in the world.
fn gather(scene: &Scene, definition: DefinitionId, place: DAffine3, paint: Option<MaterialId>, m: &mut Member, depth: usize) {
    let Some(def) = scene.definition(definition) else { return };
    if depth > 32 {
        return;
    }
    m.definitions.insert(definition);
    m.places.push((definition, place));
    let model = &def.container.model;
    for (_, face) in model.faces() {
        let points: Vec<DVec3> = face.outer.iter().map(|&v| place.transform_point3(model.vertex(v))).collect();
        for p in &points {
            m.lo = m.lo.min(*p);
            m.hi = m.hi.max(*p);
        }
        let color = face.material.or(paint).and_then(|id| scene.material(id)).map_or([222, 214, 196], |mat| mat.color);
        m.faces.push((points, place.transform_vector3(face.normal).normalize(), color));
    }
    for (_, instance) in def.container.instances() {
        if scene.is_visible(instance) {
            gather(scene, instance.definition, place * instance.transform, instance.material.or(paint), m, depth + 1);
        }
    }
}

/// Hardware placed directly in an assembly: `name -> count`.
fn hardware_in(scene: &Scene, container: &Container) -> BTreeMap<String, u32> {
    let mut total = BTreeMap::new();
    for (_, instance) in container.instances() {
        if let Some(name) = scene.definition(instance.definition).and_then(|d| d.hardware.clone()) {
            *total.entry(name).or_default() += 1;
        }
    }
    total
}

/// How much two touching boxes share: the area of their meeting (the two largest sides of their
/// overlap).
fn contact(a: &Member, b: &Member) -> f64 {
    let overlap = (a.hi.min(b.hi) - a.lo.max(b.lo)).max(DVec3::ZERO);
    let mut sides = [overlap.x, overlap.y, overlap.z];
    sides.sort_by(f64::total_cmp);
    sides[1] * sides[2]
}

/// How much of `piece`'s faces lie against `m`'s box (each flat face's area within it): a pair
/// of slides lies along a drawer box's sides, not its front, though its box spans the front.
fn against(m: &Member, piece: &Member) -> f64 {
    const TOL: f64 = 1e-3;
    piece
        .faces
        .iter()
        .map(|(points, _, _)| {
            let (lo, hi) = points.iter().fold((DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            if (hi - lo).min_element() > TOL || (m.lo - hi).max_element() > TOL || (lo - m.hi).max_element() > TOL {
                return 0.0;
            }
            let overlap = (hi.min(m.hi) - lo.max(m.lo)).max(DVec3::ZERO);
            let mut sides = [overlap.x, overlap.y, overlap.z];
            sides.sort_by(f64::total_cmp);
            sides[1] * sides[2]
        })
        .sum()
}

/// Whether two boxes touch or overlap.
fn touching(a: &Member, b: &Member) -> bool {
    const TOL: f64 = 1e-3;
    (a.lo - b.hi).max_element() <= TOL && (b.lo - a.hi).max_element() <= TOL
}

/// The joints between two members, by name (from the drawing's joints).
fn joints_between(scene: &Scene, a: &Member, b: &Member) -> Vec<&'static str> {
    // Joints are kept per design, so copies share them: only those with a cut where these two
    // meet count (a drawer of copies dovetailed at one corner isn't joined at the next).
    const TOL: f64 = 1e-3;
    let meeting = (a.lo.max(b.lo) - DVec3::splat(TOL), a.hi.min(b.hi) + DVec3::splat(TOL));
    let here = |j: &crate::joinery::Joint| {
        j.ops.is_empty()
            || j.ops.iter().any(|op| {
                let (lo, hi) = (meeting.0 - DVec3::splat(op.distance.abs()), meeting.1 + DVec3::splat(op.distance.abs()));
                a.places.iter().chain(&b.places).filter(|(d, _)| *d == op.part).any(|(_, place)| {
                    op.points.iter().map(|p| place.transform_point3(*p)).any(|p| p.cmpge(lo).all() && p.cmple(hi).all())
                })
            })
    };
    let mut kinds: Vec<&'static str> = scene
        .joints
        .iter()
        .filter(|j| (a.definitions.contains(&j.cut) && b.definitions.contains(&j.into)) || (b.definitions.contains(&j.cut) && a.definitions.contains(&j.into)))
        .filter(|j| here(j))
        .map(|j| j.kind.name())
        .collect();
    kinds.sort_unstable();
    kinds.dedup();
    kinds
}

fn steps(scene: &Scene, members: &[Member], loose: &BTreeMap<String, u32>) -> Vec<String> {
    let n = members.len();
    let joined: Vec<Vec<(usize, Vec<&'static str>)>> = (0..n)
        .map(|i| {
            (0..n)
                .filter(|&j| j != i && !members[i].moves && !members[j].moves && touching(&members[i], &members[j]))
                .filter_map(|j| {
                    let kinds = joints_between(scene, &members[i], &members[j]);
                    (!kinds.is_empty()).then_some((j, kinds))
                })
                .collect()
        })
        .collect();
    // Panels held in grooves (a drawer bottom, a door panel): slid in without glue as the last part
    // around them goes on, not glued up.
    let panel = |i: usize| joined[i].len() >= 2 && joined[i].iter().all(|(_, kinds)| kinds.iter().all(|k| *k == "Groove"));
    let panels: Vec<(usize, Vec<usize>)> = (0..n).filter(|&i| panel(i)).map(|i| (i, joined[i].iter().map(|(j, _)| *j).collect())).collect();
    let joined: Vec<Vec<(usize, Vec<&'static str>)>> = (0..n)
        .map(|i| if panel(i) { Vec::new() } else { joined[i].iter().filter(|(j, _)| !panel(*j)).cloned().collect() })
        .collect();
    let mut steps = Vec::new();
    let mut done = vec![false; n];
    // Frames first, as a table is glued up: rails that each join two posts (not joined to each
    // other), every post once; the longest such rails, each glued up with its two posts.
    if let Some(rails) = frames(members, &joined) {
        let first = rails[0];
        let same = rails.iter().all(|&r| members[r].name == members[first].name)
            && rails.iter().flat_map(|&r| joined[r].iter().map(|(p, _)| members[*p].name.clone())).collect::<BTreeSet<_>>().len() == 1;
        for (k, &r) in rails.iter().enumerate() {
            if same && k > 0 {
                break;
            }
            let posts: Vec<String> = joined[r].iter().map(|(p, kinds)| format!("{} ({})", members[*p].name, kinds.join(", ").to_lowercase())).collect();
            let make = if same && rails.len() > 1 { format!(" Make {}.", rails.len()) } else { String::new() };
            steps.push(format!("Glue up a frame: {} between {}.{make}", members[r].name, both(&posts)));
        }
        for &r in &rails {
            done[r] = true;
            for (p, _) in &joined[r] {
                done[*p] = true;
            }
        }
        if (0..n).any(|i| !done[i] && joined[i].iter().any(|(j, _)| done[*j])) {
            steps.push("Join the frames:".to_owned());
        }
    }
    // Then grow from the most-joined part, adding the part joined most to what's together.
    loop {
        if !done.iter().any(|d| *d) || !(0..n).any(|i| !done[i] && joined[i].iter().any(|(j, _)| done[*j])) {
            let Some(start) = (0..n).filter(|&i| !done[i] && !joined[i].is_empty()).max_by_key(|&i| joined[i].len()) else { break };
            done[start] = true;
            let word = if steps.is_empty() { "Start with" } else { "Separately, start with" };
            steps.push(format!("{word} {}.", members[start].name));
        }
        loop {
            // Next, a part joined to what's together: the simplest first (fewest joins in all: shelves
            // into one side before the other side goes on), then the one joined most to what's together.
            let next = (0..n)
                .filter(|&i| !done[i])
                .map(|i| {
                    let to_done = joined[i].iter().filter(|(j, _)| done[*j]).count() as i64;
                    let to_rest = joined[i].len() as i64 - to_done;
                    (i, to_done, to_done - to_rest, -(joined[i].len() as i64))
                })
                .filter(|(_, to_done, _, _)| *to_done > 0)
                .max_by_key(|(i, _, score, fewer)| (*fewer, *score, -(*i as i64)));
            let Some((i, _, _, _)) = next else { break };
            // A panel goes in before the last part around it.
            for (p, around) in &panels {
                if !done[*p] && around.contains(&i) && around.iter().filter(|j| !done[**j]).count() == 1 {
                    done[*p] = true;
                    steps.push(format!("Slide {} into its grooves, without glue (it floats).", members[*p].name));
                }
            }
            done[i] = true;
            let to: Vec<String> = joined[i].iter().filter(|(j, _)| done[*j] && *j != i).map(|(j, kinds)| format!("{} ({})", members[*j].name, kinds.join(", ").to_lowercase())).collect();
            steps.push(format!("Glue {} to {}.", members[i].name, both(&to)));
        }
        if !(0..n).any(|i| !done[i] && !joined[i].is_empty()) {
            break;
        }
    }
    // The same step twice in a row (both side aprons): once, said twice.
    let mut merged: Vec<(String, usize)> = Vec::new();
    for step in steps.drain(..) {
        match merged.last_mut() {
            Some((last, count)) if *last == step && !step.ends_with(':') => *count += 1,
            _ => merged.push((step, 1)),
        }
    }
    steps = merged.into_iter().map(|(step, count)| if count > 1 { format!("{step} Do this {count} times.") } else { step }).collect();
    if steps.is_empty() && n > 0 {
        steps.push("Nothing here is joined yet: make its joints in the Joinery panel to get a glue-up order.".to_owned());
    }
    // Once the glue's dry: hardware on the glued-up parts (slides on a drawer box's sides).
    for (i, m) in members.iter().enumerate() {
        if done[i] && !m.moves && !m.hardware.is_empty() {
            steps.push(mount(m));
        }
    }
    // Parts that only touch: fastened (a solid top so it can move), then drawers and doors.
    for (i, m) in members.iter().enumerate() {
        if done[i] || m.moves {
            continue;
        }
        // Fastened to what it bears on: the parts it touches over the most area (a top to its
        // aprons, not the tops of its legs; a drawer front to the box's front, not its sides' ends).
        let touches: Vec<(usize, f64)> = (0..n).filter(|&j| j != i && done[j] && touching(m, &members[j])).map(|j| (j, contact(m, &members[j]))).collect();
        let most = touches.iter().map(|(_, a)| *a).fold(0.0, f64::max);
        let bears: Vec<usize> = touches.iter().filter(|(_, a)| *a >= most * 0.5).map(|(j, _)| *j).collect();
        let against = counted(&bears.iter().map(|&j| members[j].name.clone()).collect::<Vec<_>>());
        let on_top = bears.iter().all(|&j| m.lo.z >= members[j].hi.z - 1e-3);
        if against.is_empty() {
            continue;
        }
        done[i] = true;
        let how = if m.wide_solid && on_top { " with fasteners that let it move across its grain (tabletop buttons or figure-8s, or screws in slotted holes)" } else { "" };
        steps.push(format!("Fasten {} to {}{how}.", m.name, list(&against)));
        if !m.hardware.is_empty() {
            steps.push(mount(m));
        }
    }
    for m in members.iter().filter(|m| m.moves) {
        let verb = if m.name.to_lowercase().contains("door") { "Hang" } else { "Fit" };
        let on = if verb == "Hang" { "its hinges" } else { "its slides" };
        let with = if m.hardware.is_empty() { String::new() } else { format!(": {}", pieces(&m.hardware)) };
        steps.push(format!("{verb} {} on {on}{with}.", m.name));
    }
    // The same step twice in a row (both doors hung): once, said twice.
    let mut merged: Vec<(String, usize)> = Vec::new();
    for step in steps.drain(..) {
        match merged.last_mut() {
            Some((last, count)) if *last == step => *count += 1,
            _ => merged.push((step, 1)),
        }
    }
    steps = merged.into_iter().map(|(step, count)| if count > 1 { format!("{step} Do this {count} times.") } else { step }).collect();
    for (i, m) in members.iter().enumerate() {
        if !done[i] && !m.moves {
            steps.push(format!("Place {} (it doesn't touch the rest).", m.name));
            if !m.hardware.is_empty() {
                steps.push(mount(m));
            }
        }
    }
    // Hardware that touches nothing here.
    if !loose.is_empty() {
        steps.push(format!("Install the hardware: {}.", pieces(loose)));
    }
    steps
}

/// `Mount 2 × Bar pull, 5" on E (Drawer front).`; hardware on several parts names them all: `Mount
/// 1 × 18" side-mount drawer slides (pair) on both J (Drawer box side).`
fn mount(m: &Member) -> String {
    let mut by_place: Vec<(String, BTreeMap<String, u32>)> = Vec::new();
    for (piece, count) in &m.hardware {
        let mut on = vec![m.name.clone()];
        on.extend(m.also_on.get(piece).cloned().unwrap_or_default());
        let on = if on.len() == 2 { both(&on) } else { list(&counted(&on)) };
        match by_place.iter_mut().find(|(o, _)| *o == on) {
            Some((_, pieces)) => {
                pieces.insert(piece.clone(), *count);
            }
            None => by_place.push((on, BTreeMap::from([(piece.clone(), *count)]))),
        }
    }
    by_place.iter().map(|(on, hardware)| format!("Mount {} on {on}.", pieces(hardware))).collect::<Vec<_>>().join(" ")
}

/// `2 × Concealed hinge (35 mm cup) and 1 × Knob`.
fn pieces(hardware: &BTreeMap<String, u32>) -> String {
    list(&hardware.iter().map(|(name, count)| format!("{count} × {name}")).collect::<Vec<_>>())
}

/// Rails for frames: members joined to exactly two others that aren't joined to each other, all
/// alike (one name), between them using every post once, at least two posts; the longest such.
fn frames(members: &[Member], joined: &[Vec<(usize, Vec<&'static str>)>]) -> Option<Vec<usize>> {
    let n = members.len();
    let length = |i: usize| (members[i].hi - members[i].lo).max_element();
    let is_rail = |i: usize| joined[i].len() == 2 && {
        let (a, b) = (joined[i][0].0, joined[i][1].0);
        !joined[a].iter().any(|(j, _)| *j == b)
    };
    let mut names: Vec<String> = (0..n).filter(|&i| is_rail(i)).map(|i| members[i].name.clone()).collect();
    names.sort_by(|a, b| {
        let len = |name: &String| (0..n).filter(|&i| &members[i].name == name).map(length).fold(0.0, f64::max);
        len(b).total_cmp(&len(a))
    });
    names.dedup();
    for name in names {
        let rails: Vec<usize> = (0..n).filter(|&i| is_rail(i) && members[i].name == name).collect();
        let posts: Vec<usize> = rails.iter().flat_map(|&r| joined[r].iter().map(|(p, _)| *p)).collect();
        let distinct: BTreeSet<usize> = posts.iter().copied().collect();
        if rails.len() >= 2 && distinct.len() == posts.len() && !distinct.iter().any(|p| rails.contains(p)) {
            return Some(rails);
        }
    }
    None
}

/// Names with repeats counted: `A (Leg) ×2`.
fn counted(names: &[String]) -> Vec<String> {
    let mut seen: Vec<(String, usize)> = Vec::new();
    for name in names {
        match seen.iter_mut().find(|(n, _)| n == name) {
            Some((_, count)) => *count += 1,
            None => seen.push((name.clone(), 1)),
        }
    }
    seen.into_iter().map(|(name, count)| if count > 1 { format!("{count} × {name}") } else { name }).collect()
}

/// Like [`list`], with two of the same said as `both`: `both A (Leg) (mortise and tenon)`.
fn both(items: &[String]) -> String {
    match items {
        [a, b] if a == b => format!("both {a}"),
        _ => list(&counted(items)),
    }
}

/// `a`, `a and b`, `a, b and c`.
fn list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// A face seen in the drawing: its area on the page, its depth (toward the eye) and its outline.
type Seen = (f64, f64, Vec<(f64, f64)>);

/// A label placed beside the drawing: its center, where its leader points, and its text.
type Label<'a> = ((f64, f64), (f64, f64), &'a str);

/// A face to draw: its depth (toward the eye), its points on the page, color and how squarely it faces you.
type Polygon = (f64, Vec<(f64, f64)>, [u8; 3], f64);

/// The assembly drawn from the front-left, above, each member pulled away from the middle.
fn exploded(members: &[Member]) -> String {
    // Looking from front-left, above (the Iso view's direction).
    let toward_eye = DVec3::new(-0.433, -0.75, 0.5).normalize();
    let right = DVec3::Z.cross(toward_eye).normalize();
    let up = toward_eye.cross(right);
    let (lo, hi) = members.iter().fold((DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY)), |(lo, hi), m| (lo.min(m.lo), hi.max(m.hi)));
    let middle = (lo + hi) / 2.0;
    let mut polygons: Vec<Polygon> = Vec::new();
    // Each member's label and where its leader points: the middle of its biggest face toward you.
    let mut labels: Vec<((f64, f64), String, bool)> = Vec::new();
    let mut candidates = Vec::new();
    // Every face drawn, by the member it's on, for leaders to keep clear of.
    let mut outlines: Vec<(usize, Vec<(f64, f64)>)> = Vec::new();
    let project = |p: DVec3| (p.dot(right), -p.dot(up));
    for m in members {
        let center = (m.lo + m.hi) / 2.0;
        let mut shift = (center - middle) * 0.6;
        // Something resting on top with nothing above it (a table top) lifts well clear.
        let resting = members.iter().any(|o| !std::ptr::eq(o, m) && touching(o, m) && m.lo.z >= o.hi.z - 1e-3);
        let covered = members.iter().any(|o| !std::ptr::eq(o, m) && o.lo.z >= m.hi.z - 1e-3 && touching(o, m));
        if resting && !covered && m.lo.z > middle.z {
            shift.z += 0.4 * (hi.z - lo.z);
        }
        let mut faces: Vec<Seen> = Vec::new();
        for (points, normal, color) in &m.faces {
            let facing = normal.dot(toward_eye);
            if facing <= 1e-6 {
                continue;
            }
            let moved: Vec<DVec3> = points.iter().map(|p| *p + shift).collect();
            let depth = moved.iter().map(|p| p.dot(toward_eye)).sum::<f64>() / moved.len() as f64;
            let flat: Vec<(f64, f64)> = moved.iter().map(|p| project(*p)).collect();
            faces.push((area(&flat), depth, flat.clone()));
            outlines.push((candidates.len(), flat.clone()));
            polygons.push((depth, flat, *color, facing));
        }
        faces.sort_by(|a, b| b.0.total_cmp(&a.0));
        // A part by its cut-list letter; a drawer or door by its name.
        let tag = m.label.clone().unwrap_or_else(|| m.name.clone());
        candidates.push((faces, tag, project(center + shift)));
    }
    polygons.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (mut x0, mut y0, mut x1, mut y1) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for (_, points, _, _) in &polygons {
        for &(x, y) in points {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    if !x0.is_finite() {
        return String::new();
    }
    let span = (x1 - x0).max(y1 - y0);
    let stroke = span / 600.0;
    let size = span / 40.0;
    // Labels in a column either side of the drawing, in the order of what they point at, with a
    // leader line to it.
    let gap = 2.0 * size;
    let middle_x = (x0 + x1) / 2.0;
    let width = |tag: &str| if tag.chars().count() <= 2 { 1.8 * size } else { 0.6 * size * tag.chars().count() as f64 + size };
    // Each leader points at a spot you can see on its member (the middle of a face, or partway
    // toward a corner, not covered there by something nearer), from the column on one side; the
    // spot and side whose leader crosses the fewest other members, the biggest face first.
    let crossings = |owner: usize, from: (f64, f64), to: (f64, f64)| -> usize {
        let mut hit: Vec<usize> = Vec::new();
        for k in 1..24 {
            let t = f64::from(k) / 24.0;
            let p = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            for (o, outline) in &outlines {
                if *o != owner && !hit.contains(o) && inside(outline, p) {
                    hit.push(*o);
                }
            }
        }
        hit.len()
    };
    for (owner, (faces, tag, fallback)) in candidates.into_iter().enumerate() {
        let spots: Vec<(f64, f64)> = faces
            .iter()
            .flat_map(|(_, depth, outline)| {
                let n = outline.len() as f64;
                let middle = (outline.iter().map(|p| p.0).sum::<f64>() / n, outline.iter().map(|p| p.1).sum::<f64>() / n);
                std::iter::once(middle)
                    .chain(outline.iter().map(move |c| (middle.0 + (c.0 - middle.0) * 0.6, middle.1 + (c.1 - middle.1) * 0.6)))
                    .filter(|&p| inside(outline, p) && !polygons.iter().any(|(d, other, _, _)| *d > depth + 1e-6 && inside(other, p)))
                    .collect::<Vec<_>>()
            })
            .collect();
        let best = spots
            .iter()
            .enumerate()
            .flat_map(|(rank, &spot)| [true, false].map(|left| (rank, spot, left)))
            .min_by_key(|&(rank, spot, left)| {
                let edge = if left { x0 } else { x1 };
                // Fewest crossings; then the nearer side; then the biggest face.
                (crossings(owner, (edge, spot.1), spot), (spot.0 < middle_x) != left, rank)
            });
        let (spot, left) = best.map_or((fallback, fallback.0 < middle_x), |(_, spot, left)| (spot, left));
        labels.push((spot, tag, left));
    }
    let mut placed: Vec<Label> = Vec::new();
    let (mut bx0, mut by0, mut bx1, mut by1) = (x0, y0, x1, y1);
    for left in [true, false] {
        let mut side: Vec<&((f64, f64), String, bool)> = labels.iter().filter(|(_, _, l)| *l == left).collect();
        side.sort_by(|a, b| a.0.1.total_cmp(&b.0.1));
        let mut next = f64::NEG_INFINITY;
        for (anchor, tag, _) in side {
            let y = anchor.1.max(next);
            next = y + 2.2 * size;
            let w = width(tag);
            let x = if left { x0 - gap - w / 2.0 } else { x1 + gap + w / 2.0 };
            bx0 = bx0.min(x - w / 2.0);
            bx1 = bx1.max(x + w / 2.0);
            by0 = by0.min(y - size);
            by1 = by1.max(y + size);
            placed.push(((x, y), *anchor, tag));
        }
    }
    let pad = 0.03 * span;
    let (w, h) = (bx1 - bx0 + 2.0 * pad, by1 - by0 + 2.0 * pad);
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{:.3} {:.3} {w:.3} {h:.3}" width="640" style="max-height:640px">"#,
        bx0 - pad,
        by0 - pad
    );
    for (_, points, [r, g, b], facing) in &polygons {
        // Lit from the eye: faces turned away are darker.
        let shade = 0.72 + 0.28 * facing;
        let c = |v: u8| ((f64::from(v) * 0.8 + 255.0 * 0.2) * shade).clamp(0.0, 255.0) as u8;
        let path: Vec<String> = points.iter().map(|(x, y)| format!("{x:.3},{y:.3}")).collect();
        let _ = write!(svg, r##"<polygon points="{}" fill="rgb({},{},{})" stroke="#333" stroke-width="{stroke:.4}" stroke-linejoin="round"/>"##, path.join(" "), c(*r), c(*g), c(*b));
    }
    for ((x, y), (ax, ay), tag) in placed {
        let w = width(tag);
        let from = if x < middle_x { x + w / 2.0 } else { x - w / 2.0 };
        let _ = write!(
            svg,
            r##"<g class="leader"><line x1="{from:.3}" y1="{y:.3}" x2="{ax:.3}" y2="{ay:.3}" stroke="#222" stroke-width="{:.4}"/><circle cx="{ax:.3}" cy="{ay:.3}" r="{:.3}" fill="#222"/></g>"##,
            stroke * 1.2,
            size * 0.18
        );
        if tag.chars().count() <= 2 {
            let _ = write!(svg, r##"<circle cx="{x:.3}" cy="{y:.3}" r="{:.3}" fill="white" stroke="#222" stroke-width="{stroke:.4}"/>"##, size * 0.9);
        } else {
            let _ = write!(
                svg,
                r##"<rect x="{:.3}" y="{:.3}" width="{w:.3}" height="{:.3}" rx="{:.3}" fill="white" stroke="#222" stroke-width="{stroke:.4}"/>"##,
                x - w / 2.0,
                y - 0.9 * size,
                1.8 * size,
                0.4 * size
            );
        }
        let _ = write!(
            svg,
            r#"<text x="{x:.3}" y="{:.3}" style="font: bold {size:.3}px sans-serif" text-anchor="middle">{}</text>"#,
            y + size * 0.35,
            crate::buildsheet::esc(tag)
        );
    }
    svg.push_str("</svg>");
    svg
}

/// Whether `p` is inside a flat polygon on the page (even-odd).
fn inside(polygon: &[(f64, f64)], p: (f64, f64)) -> bool {
    let n = polygon.len();
    let mut inside = false;
    for i in 0..n {
        let (a, b) = (polygon[i], polygon[(i + 1) % n]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) / (b.1 - a.1) * (b.0 - a.0) {
            inside = !inside;
        }
    }
    inside
}

/// The area of a flat polygon on the page.
fn area(points: &[(f64, f64)]) -> f64 {
    let n = points.len();
    (0..n).map(|i| points[i].0 * points[(i + 1) % n].1 - points[(i + 1) % n].0 * points[i].1).sum::<f64>().abs() / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::template::{Cabinet, Fill, Table, Template};

    #[test]
    fn a_table_glues_up_legs_and_aprons_then_fastens_its_top() {
        let mut doc = Document::default();
        let cherry = doc.add_preset(crate::library::LIBRARY.iter().find(|p| p.name == "Cherry").unwrap());
        doc.add_template(&Template::Table(Table::default()), Some(cherry)).unwrap();
        let list = crate::cutlist::cut_list(doc.scene());
        let all = assemblies(doc.scene(), &list);
        let table = all.iter().find(|a| a.name == "Table").expect("the table");
        let text = table.steps.join("\n");
        assert!(table.steps[0].starts_with("Glue up a frame") && table.steps[0].contains("Make 2"), "two frames of a long apron between two legs first\n{text}");
        assert!(text.contains("Join the frames") && text.contains("Side apron") && text.contains("Do this 2 times"), "{text}");
        assert!(text.contains("mortise and tenon"), "{text}");
        assert!(text.contains("Fasten") && text.contains("Table top") && text.contains("move across its grain"), "the top goes on last, free to move\n{text}");
        assert!(table.svg.starts_with("<svg") && table.svg.contains("<polygon"), "an exploded drawing");
    }

    /// `WOODSKETCH_SHEET_OUT=<file.html>`: build sheets for a table and a cabinet, to look at.
    #[test]
    fn write_sample_sheets() {
        let Ok(path) = std::env::var("WOODSKETCH_SHEET_OUT") else { return };
        let mut doc = Document::default();
        let cherry = doc.add_preset(crate::library::LIBRARY.iter().find(|p| p.name == "Cherry").unwrap());
        doc.add_template(&Template::Table(Table::default()), Some(cherry)).unwrap();
        let mut c = Cabinet { shelves: 2, depth: 18.0, height: 34.0, ..Default::default() };
        c.fills[2] = Fill::Drawer;
        c.fills[0] = Fill::Doors;
        doc.add_template(&Template::Cabinet(c), Some(cherry)).unwrap();
        // Reference faces and edges picked on every part: the far side of its thickness and width.
        doc.edit_scene(|scene, _| {
            let ids: Vec<DefinitionId> = scene.definitions().map(|(id, _)| id).collect();
            for id in ids {
                let Some(def) = scene.definition(id) else { continue };
                let Some(extents) = crate::part::extents(def, glam::DMat3::IDENTITY) else { continue };
                let [_, w, t] = crate::part::axes(extents, &def.part);
                let def = scene.definition_mut(id).unwrap();
                def.part.reference_face = Some(crate::part::Side { axis: crate::part::PartAxis::ALL[t], positive: true });
                def.part.reference_edge = Some(crate::part::Side { axis: crate::part::PartAxis::ALL[w], positive: false });
            }
        });
        let list = crate::cutlist::cut_list(doc.scene());
        std::fs::write(path, crate::buildsheet::to_html(doc.scene(), &list, "Samples")).unwrap();
    }

    #[test]
    fn a_cabinet_does_its_drawers_first_and_fits_them_last() {
        let mut doc = Document::default();
        let mut c = Cabinet { shelves: 1, depth: 20.0, ..Default::default() };
        c.fills[0] = Fill::Drawer;
        c.fills[1] = Fill::Doors;
        doc.add_template(&Template::Cabinet(c), None).unwrap();
        let list = crate::cutlist::cut_list(doc.scene());
        let all = assemblies(doc.scene(), &list);
        let names: Vec<&str> = all.iter().map(|a| a.name.as_str()).collect();
        let cabinet = names.iter().position(|n| *n == "Cabinet").unwrap();
        assert!(names.iter().position(|n| *n == "Drawer").unwrap() < cabinet, "the drawer is built before the cabinet: {names:?}");
        let text = all[cabinet].steps.join("\n");
        assert!(text.contains("Fit Drawer on its slides") && text.contains("Hang Door on its hinges"), "{text}");
        assert!(text.contains("Fasten") && text.contains("Back"), "the back is fastened: {text}");
        // Hardware at the step it's used: hinges as a door is hung, slides on the box's sides.
        assert!(text.contains("Hang Door on its hinges: ") && text.contains("Concealed hinge"), "{text}");
        let drawer = all.iter().find(|a| a.name == "Drawer").unwrap().steps.join("\n");
        assert!(drawer.contains("slides (pair) on both ") && drawer.lines().any(|l| l.contains("slides") && l.ends_with("(Drawer box side).")), "on both box sides, not its front: {drawer}");
        assert!(drawer.lines().any(|l| l.contains("Bar pull") && l.ends_with("(Drawer front).")), "{drawer}");
        assert!(!text.contains("Install the hardware") && !drawer.contains("Install the hardware"), "nothing left over");
        // Each part or sub-assembly labeled, with a leader line to it.
        let svg = &all[cabinet].svg;
        assert_eq!(svg.matches("class=\"leader\"").count(), svg.matches("<text").count(), "a leader for every label");
        assert!(svg.contains(">Drawer</text>") && svg.contains(">Door</text>"), "sub-assemblies by name");
    }
}
