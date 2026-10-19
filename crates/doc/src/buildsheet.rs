//! Build sheets: a printable web page with one sheet per part, for the shop. Each has the part's
//! name, cut-list letter and count, material and stock, finished size and grain; three
//! dimensioned views (face, edge, end) drawn to scale, showing its joints and profiles; and the
//! steps in shop order (mill to size, joinery, profiles, sand). It says what to make, not how
//! to set up a machine.

use std::collections::BTreeMap;
use std::fmt::Write;

use glam::{DMat3, DVec2, DVec3};

use crate::cutlist::{CutList, Part};
use crate::cutplan::match_stock;
use crate::joinery::JointKind;
use crate::units::format_length;
use crate::{Container, DefinitionId, Model, Scene};

/// Escape text for HTML element content.
pub(crate) fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// A part to make: one definition with faces, however many copies (and other definitions shaped
/// the same, or mirror images of it, as a drawer's left and right sides).
struct Sheet {
    definition: DefinitionId,
    count: usize,
    /// How many of `count` are mirror images.
    mirrored: usize,
    material: Option<crate::MaterialId>,
    /// How much each of its own axes is stretched (copies' scale).
    scale: DVec3,
}

fn collect(scene: &Scene, container: &Container, inherited: Option<crate::MaterialId>, linear: DMat3, found: &mut BTreeMap<DefinitionId, Sheet>) {
    for (_, instance) in container.instances() {
        if !scene.is_visible(instance) {
            continue;
        }
        let Some(def) = scene.definition(instance.definition) else { continue };
        let material = instance.material.or(inherited);
        let linear = linear * instance.transform.matrix3;
        if def.container.model.faces().next().is_some() {
            let material = material.or_else(|| def.container.model.faces().find_map(|(_, f)| f.material));
            let scale = DVec3::from_array([0, 1, 2].map(|i| (linear * DVec3::AXES[i]).length()));
            let sheet = found.entry(instance.definition).or_insert(Sheet { definition: instance.definition, count: 0, mirrored: 0, material, scale });
            sheet.count += 1;
            // A copy placed mirrored (scaled by -1) is the other hand.
            if linear.determinant() < 0.0 {
                sheet.mirrored += 1;
            }
        }
        collect(scene, &def.container, material, linear, found);
    }
}

/// The build sheets for the drawing, one per part, in cut-list order.
pub fn to_html(scene: &Scene, list: &CutList, title: &str) -> String {
    let mut found = BTreeMap::new();
    collect(scene, &scene.root, None, DMat3::IDENTITY, &mut found);

    let mut html = String::new();
    let _ = write!(
        html,
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>{title}</title>
<style>
body {{ font: 13px/1.45 system-ui, sans-serif; color: #222; margin: 24px; }}
h1 {{ font-size: 20px; margin: 0 0 4px; }}
h2 {{ font-size: 18px; margin: 0 0 2px; }}
.label {{ display: inline-block; min-width: 22px; padding: 0 6px; margin-right: 6px; border: 2px solid #222; border-radius: 4px; text-align: center; }}
.meta {{ color: #555; margin-bottom: 10px; }}
.sheet {{ margin-bottom: 36px; }}
.facts td {{ padding: 1px 12px 1px 0; vertical-align: top; }}
.facts td:first-child {{ color: #666; }}
ol {{ margin: 6px 0 0 20px; padding: 0; }}
li {{ margin: 2px 0; }}
svg {{ display: block; margin: 10px 0; }}
svg text {{ font: 11px system-ui, sans-serif; fill: #333; }}
@media print {{ .sheet {{ page-break-after: always; }} .noprint {{ display: none; }} body {{ margin: 0; }} }}
</style></head><body>
<h1>{title}</h1>
<p class="meta noprint"><button onclick="window.print()">Print</button> One part per page, then how it goes together (the letters are the cut list's). Sizes are finished sizes.</p>
"#,
        title = esc(title)
    );

    // Cut-list order: by the letter of the row each part falls in. Only what's made in the shop:
    // not flat things (no row) or parts of metal, glass and such (material type Other).
    let not_wood = |sheet: &Sheet| {
        sheet.material.and_then(|m| scene.material(m)).is_some_and(|m| matches!(m.kind, crate::MaterialKind::Other | crate::MaterialKind::Hardware))
    };
    let mut sheets: Vec<(String, Sheet, Option<&Part>)> = Vec::new();
    for sheet in found.into_values().filter(|sheet| !not_wood(sheet)) {
        let Some(row) = row_for(scene, list, &sheet) else { continue };
        // Another design with the same name, row and shape (or its mirror image) is the same
        // part: one sheet, counted together.
        let name = scene.definition(sheet.definition).map(|d| d.name.as_str());
        let same = sheets.iter_mut().find_map(|(label, kept, _)| {
            if *label != row.label || scene.definition(kept.definition).map(|d| d.name.as_str()) != name {
                return None;
            }
            same_shape(scene, kept, &sheet).map(|mirrored| (kept, mirrored))
        });
        match same {
            Some((kept, mirrored)) => {
                kept.count += sheet.count;
                kept.mirrored += if mirrored { sheet.count - sheet.mirrored } else { sheet.mirrored };
            }
            None => sheets.push((row.label.clone(), sheet, Some(row))),
        }
    }
    sheets.sort_by(|a, b| (a.0.len(), &a.0).cmp(&(b.0.len(), &b.0)));
    if sheets.is_empty() {
        html.push_str("<p>No parts yet. Each group or component is a part.</p>");
    }
    for (label, sheet, row) in &sheets {
        write_sheet(&mut html, scene, sheet, label, *row);
    }
    // Then putting it together: each assembly, sub-assemblies first.
    for assembly in crate::assembly::assemblies(scene, list) {
        let _ = write!(html, "<section class=\"sheet assembly\"><h2>Assemble: {}</h2>", esc(&assembly.name));
        html.push_str(&assembly.svg);
        html.push_str("<ol>");
        for step in &assembly.steps {
            let _ = write!(html, "<li>{}</li>", esc(step));
        }
        html.push_str("</ol></section>
");
    }
    html.push_str("</body></html>\n");
    html
}

/// Whether two parts are the same shape: `Some(false)` alike, turned any way; `Some(true)` mirror
/// images (a left and a right); `None` different. Compared along each one's length, width and
/// thickness from its corner, every way it can be turned end for end or flipped.
fn same_shape(scene: &Scene, a: &Sheet, b: &Sheet) -> Option<bool> {
    // Each part's faces as triangles in its own axes (length, width, thickness) from its corner,
    // and its size.
    let triangles = |sheet: &Sheet| -> Option<(Vec<[DVec3; 3]>, DVec3)> {
        let def = scene.definition(sheet.definition)?;
        let model = &def.container.model;
        let extents = crate::part::extents(def, DMat3::from_diagonal(sheet.scale))?;
        let axes = crate::part::axes(extents, &def.part);
        let (lo, _) = model.bounds()?;
        let local = |p: DVec3| {
            let p = (p - lo) * sheet.scale;
            DVec3::new(p[axes[0]], p[axes[1]], p[axes[2]])
        };
        let triangles = model.faces().flat_map(|(_, f)| model.triangulate(f)).map(|t| t.map(local)).collect();
        Some((triangles, DVec3::new(extents[axes[0]], extents[axes[1]], extents[axes[2]])))
    };
    let (ta, size) = triangles(a)?;
    let (tb, size_b) = triangles(b)?;
    if (size - size_b).abs().max_element() > 1e-3 {
        return None;
    }
    let want = surface(&ta);
    // Flip any of the three directions (end for end, edge for edge, face for face): an even number
    // of flips is the part turned over, an odd number its mirror image.
    let mut found = None;
    for flips in 0..8u32 {
        let flip = |p: DVec3| DVec3::from_array([0, 1, 2].map(|i| if flips & (1 << i) != 0 { size[i] - p[i] } else { p[i] }));
        let have = surface(&tb.iter().map(|t| t.map(flip)).collect::<Vec<_>>());
        let alike = have.len() == want.len() && have.iter().all(|(k, area)| want.get(k).is_some_and(|w| (w - area).abs() < 1e-3 * w.max(1.0)));
        if alike {
            let mirrored = flips.count_ones() % 2 == 1;
            if !mirrored {
                return Some(false);
            }
            found = Some(true);
        }
    }
    found
}

/// How much surface lies on each plane (its normal either way, and where it is): the same for
/// two parts of one shape however their faces are split up (an extra seam, a leftover point on an
/// edge).
fn surface(triangles: &[[DVec3; 3]]) -> BTreeMap<[i64; 4], f64> {
    let mut planes = BTreeMap::new();
    for [a, b, c] in triangles {
        let cross = (*b - *a).cross(*c - *a);
        let area = cross.length() / 2.0;
        let Some(mut n) = cross.try_normalize().filter(|_| area > 1e-9) else { continue };
        // One key per plane, whichever way it faces (a mirror image reverses the winding).
        if n.x < -1e-9 || (n.x.abs() <= 1e-9 && (n.y < -1e-9 || (n.y.abs() <= 1e-9 && n.z < 0.0))) {
            n = -n;
        }
        let key = [n.x, n.y, n.z, n.dot(*a)].map(|v| (v * 1000.0).round() as i64);
        *planes.entry(key).or_insert(0.0) += area;
    }
    planes
}

/// The cut-list row a part falls in (same material and size).
fn row_for<'a>(scene: &Scene, list: &'a CutList, sheet: &Sheet) -> Option<&'a Part> {
    let def = scene.definition(sheet.definition)?;
    let [length, width, thickness] = crate::part::size(crate::part::extents(def, DMat3::from_diagonal(sheet.scale))?, &def.part);
    let close = |a: f64, b: f64| (a - b).abs() < 1.0 / 64.0;
    list.materials
        .iter()
        .filter(|m| m.material == sheet.material)
        .flat_map(|m| &m.parts)
        .find(|p| close(p.length, length) && close(p.width, width) && close(p.thickness, thickness))
}

fn write_sheet(html: &mut String, scene: &Scene, sheet: &Sheet, label: &str, row: Option<&Part>) {
    let Some(def) = scene.definition(sheet.definition) else { return };
    let Some(extents) = crate::part::extents(def, DMat3::from_diagonal(sheet.scale)) else { return };
    let axes = crate::part::axes(extents, &def.part);
    let [length, width, thickness] = axes.map(|i| extents[i]);
    let material = sheet.material.and_then(|m| scene.material(m));

    let _ = write!(
        html,
        r#"<section class="sheet"><h2><span class="label">{}</span>{}</h2>
<div class="meta">Make {}</div>
<table class="facts">
"#,
        esc(label),
        esc(&def.name),
        match (sheet.count, sheet.mirrored) {
            (1, _) => "1".to_owned(),
            (n, 0) => format!("{n}, all the same"),
            (2, 1) => "2, a mirrored pair (a left and a right)".to_owned(),
            (n, m) => format!("{n}: {} one way, {m} mirrored", n - m),
        }
    );
    let size = format!("{} x {} x {}", format_length(length), format_length(width), format_length(thickness));
    let _ = writeln!(html, "<tr><td>Finished size</td><td>{} <span class=\"meta\">(length x width x thickness)</span></td></tr>", esc(&size));
    let stock = material.map_or_else(
        || "No material yet".to_owned(),
        |m| {
            let part = Part { count: 1, length, width, thickness, ..Part::default() };
            match match_stock(&part, m.kind, &m.standard) {
                Ok(stock) => format!("{}, {}", m.name, stock.name),
                Err(_) => m.name.clone(),
            }
        },
    );
    let _ = writeln!(html, "<tr><td>Material</td><td>{}</td></tr>", esc(&stock));
    let cut = match def.part.cut {
        crate::part::Cut::Unknown => String::new(),
        c => format!(", {}", c.name().to_lowercase()),
    };
    let _ = writeln!(html, "<tr><td>Grain</td><td>Along the length{}</td></tr>", esc(&cut));
    if let Some(row) = row
        && row.count > sheet.count
    {
        let _ = writeln!(html, "<tr><td>Cut list</td><td>Row {}: {} parts this size in all</td></tr>", esc(&row.label), row.count);
    }
    html.push_str("</table>\n");

    // A solid-wood glue-up: its boards come first, and are drawn on the views.
    let glue = material.filter(|m| m.kind == crate::MaterialKind::SolidWood).and_then(|m| {
        let part = Part { count: 1, length, width, thickness, glue_up: def.part.glue_up, ..Part::default() };
        let n = crate::cutplan::glue_up(&part, &m.standard)?;
        let (l, w) = crate::cutplan::glue_up_board(&part, n);
        let boards = format!("{n} boards {} x {} x {}", format_length(thickness), format_length(w), format_length(l));
        // Reinforced seams: what goes in each.
        let seams = def.part.edge_joint.seam(thickness, length).map(|(piece, per_seam)| {
            let each = if per_seam == 1 { format!("a {piece}") } else { format!("{per_seam} {piece}s") };
            format!("{each} along each of the {} seams", n - 1)
        });
        Some((n, boards, seams))
    });
    // Where each cut is: dimensioned on the views and listed, from the reference face and edge.
    let model = &def.container.model;
    let placed: Vec<(String, (DVec3, DVec3))> = cuts(scene, sheet.definition)
        .into_iter()
        .filter_map(|cut| Some((cut.what, cut_box(model, sheet.scale, &cut.ops)?)))
        .collect();
    let profiles = scene
        .profiles
        .iter()
        .filter(|p| p.part == sheet.definition)
        .map(|p| (format!("{} {}", format_length(p.size), p.profile.name().to_lowercase()), profile_edges(model, &p.edges)))
        .collect();
    let marks = Marks { face: def.part.reference_face, edge: def.part.reference_edge, end: def.part.reference_end, cuts: placed.iter().map(|(_, b)| *b).collect(), profiles };
    html.push_str(&views(model, sheet.scale, axes, [length, width, thickness], glue.as_ref().map(|(n, _, _)| (*n as usize, label)), &marks));
    if !placed.is_empty() {
        let from = match (marks.face, marks.edge, marks.end) {
            (Some(_), Some(_), Some(_)) => "from the reference face, edge and end (marked in blue)",
            (Some(_), Some(_), None) => "from the reference face and edge (marked in blue) and the left end as drawn",
            _ => "from the bottom face and edge and the left end as drawn (pick the reference face and edge in Entity Info to measure from them)",
        };
        let _ = writeln!(html, "<b>Where the cuts are</b> <span class=\"meta\">{}</span><ol>", esc(from));
        for (what, b) in &placed {
            let _ = writeln!(html, "<li>{}: {}.</li>", esc(what), esc(&where_is(*b, axes, [length, width, thickness], &marks)));
        }
        html.push_str("</ol>\n");
    }

    html.push_str("<b>Steps</b><ol>\n");
    for step in steps(scene, sheet.definition, &stock, &size, glue.as_ref().map(|(_, b, s)| (b.as_str(), s.as_deref()))) {
        let _ = writeln!(html, "<li>{}</li>", esc(&step));
    }
    html.push_str("</ol></section>\n");
}

/// What to do to the part, in shop order.
fn steps(scene: &Scene, def: DefinitionId, stock: &str, size: &str, glue: Option<(&str, Option<&str>)>) -> Vec<String> {
    let mut steps = match glue {
        Some((boards, seams)) => vec![
            format!("Mill {boards} from {stock}."),
            seams.map_or_else(String::new, |s| format!("Cut for {s}.")),
            format!("Glue them edge to edge, alternating the end grain (rings up, then down); when dry, flatten and cut to {size}, grain along the length."),
        ],
        None => vec![format!("Mill to {size} from {stock}, grain along the length.")],
    };
    steps.retain(|s| !s.is_empty());
    let mut said: Vec<(String, usize, bool)> = Vec::new(); // the step, how many, at a part's end
    for cut in cuts(scene, def) {
        match said.iter_mut().find(|(s, _, _)| *s == cut.step) {
            Some((_, count, _)) => *count += 1,
            None => said.push((cut.step, 1, cut.at_end)),
        }
    }
    for (step, count, at_end) in said {
        steps.push(match count {
            1 => step,
            // A tenon or tails at each end.
            2 if at_end => format!("{step} At both ends."),
            n => format!("{step} In {n} places."),
        });
    }
    for profile in scene.profiles.iter().filter(|p| p.part == def) {
        steps.push(format!("Rout the edges: {}.", profile.describe()));
    }
    steps.push("Sand.".to_owned());
    steps
}

/// Where a cut is on the part, as a box in the views' coordinates (the part's own axes, scaled,
/// from its corner): everything its pushes and pulls reach.
fn cut_box(model: &Model, scale: DVec3, ops: &[crate::joinery::JointOp]) -> Option<(DVec3, DVec3)> {
    let lo = model.bounds()?.0;
    let points = ops.iter().flat_map(|op| op.points.iter().flat_map(move |p| [*p, *p + op.normal * op.distance]));
    let points: Vec<DVec3> = points.map(|p| (p - lo) * scale).collect();
    let first = *points.first()?;
    Some(points.iter().fold((first, first), |(lo, hi), p| (lo.min(*p), hi.max(*p))))
}

/// Where a cut is, said from the reference face and edge (or, before they're picked, the bottom
/// face and edge as the views draw them) and the left end as drawn.
fn where_is((lo, hi): (DVec3, DVec3), [l, w, t]: [usize; 3], [length, width, thickness]: [f64; 3], marks: &Marks) -> String {
    const CLOSE: f64 = 1.0 / 64.0;
    let near = |a: f64, b: f64| (a - b).abs() < CLOSE;
    let (face, edge) = (marks.face, marks.edge);
    let mut said = Vec::new();
    // Along the length, from the reference end (or the left end as drawn).
    let (end_far, end_name, other_end) = match marks.end.filter(|s| s.axis.index() == l) {
        Some(side) => (side.positive, "the reference end", "the other end"),
        None => (false, "the left end", "the right end"),
    };
    let (from, to) = if end_far { (length - hi[l], length - lo[l]) } else { (lo[l], hi[l]) };
    if near(from, 0.0) && near(to, length) {
        said.push("the full length".to_owned());
    } else if near(from, 0.0) {
        said.push(format!("at {end_name}, {} long", format_length(to - from)));
    } else if near(to, length) {
        said.push(format!("at {other_end}, {} long", format_length(to - from)));
    } else {
        said.push(format!("{} from {end_name}, {} long", format_length(from), format_length(to - from)));
    }
    // Across the width, from the reference edge.
    let (edge_far, edge_name) = match edge.filter(|s| s.axis.index() == w) {
        Some(side) => (side.positive, "the reference edge"),
        None => (false, "the bottom edge"),
    };
    let (from, to) = if edge_far { (width - hi[w], width - lo[w]) } else { (lo[w], hi[w]) };
    if near(from, 0.0) && near(to, width) {
        said.push("across the full width".to_owned());
    } else if near(from, 0.0) {
        said.push(format!("{} in from {edge_name}", format_length(to)));
    } else if near(to, width) {
        said.push(format!("{} in from the other edge", format_length(to - from)));
    } else {
        said.push(format!("{} from {edge_name}, {} wide", format_length(from), format_length(to - from)));
    }
    // Into the thickness, from the reference face.
    let (face_far, face_name) = match face.filter(|s| s.axis.index() == t) {
        Some(side) => (side.positive, "the reference face"),
        None => (false, "the bottom face"),
    };
    let (from, to) = if face_far { (thickness - hi[t], thickness - lo[t]) } else { (lo[t], hi[t]) };
    if near(from, 0.0) && near(to, thickness) {
        said.push("through".to_owned());
    } else if near(from, 0.0) {
        said.push(format!("{} deep from {face_name}", format_length(to)));
    } else if near(to, thickness) {
        said.push(format!("{} deep from the other face", format_length(to - from)));
    } else {
        said.push(format!("{} to {} from {face_name}", format_length(from), format_length(to)));
    }
    said.join("; ")
}

/// What's measured from on a part's views: its reference face and edge, and its cuts' boxes.
struct Marks {
    face: Option<crate::part::Side>,
    edge: Option<crate::part::Side>,
    end: Option<crate::part::Side>,
    cuts: Vec<(DVec3, DVec3)>,
    /// Edge profiles: what each is, and the box edges it runs along (the part's own coordinates).
    profiles: Vec<(String, Vec<[DVec3; 2]>)>,
}

/// The box edges a profile runs along, in the part's own coordinates (see [`crate::profile::Edges`]).
fn profile_edges(model: &Model, edges: &crate::profile::Edges) -> Vec<[DVec3; 2]> {
    let Some((lo, hi)) = model.bounds() else { return Vec::new() };
    let corner = |coords: [(usize, f64); 3]| {
        let mut p = DVec3::ZERO;
        for (axis, v) in coords {
            p[axis] = v;
        }
        p
    };
    match *edges {
        crate::profile::Edges::AroundFace { axis, positive, sides } => {
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            let z = if positive { hi[axis] } else { lo[axis] };
            // `[u-, u+, v-, v+]`: the edges along v at each end of u, then along u at each end of v.
            let along_v = |at: f64| [corner([(axis, z), (u, at), (v, lo[v])]), corner([(axis, z), (u, at), (v, hi[v])])];
            let along_u = |at: f64| [corner([(axis, z), (v, at), (u, lo[u])]), corner([(axis, z), (v, at), (u, hi[u])])];
            let all = [along_v(lo[u]), along_v(hi[u]), along_u(lo[v]), along_u(hi[v])];
            all.into_iter().zip(sides).filter(|(_, on)| *on).map(|(e, _)| e).collect()
        }
        crate::profile::Edges::Along { axis, corners } => {
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            let at = [(lo[u], lo[v]), (hi[u], lo[v]), (hi[u], hi[v]), (lo[u], hi[v])];
            at.into_iter()
                .zip(corners)
                .filter(|(_, on)| *on)
                .map(|((cu, cv), _)| [corner([(u, cu), (v, cv), (axis, lo[axis])]), corner([(u, cu), (v, cv), (axis, hi[axis])])])
                .collect()
        }
    }
}

/// One cut (or addition, like a tenon) a joint made on a part's design.
struct Cut {
    /// Its step: `Cut the mortise for Side apron (...).`
    step: String,
    /// What it is, short: `Mortise and tenon, Side apron`.
    what: String,
    /// Its pushes and pulls on the part's geometry (none for joints made before they were kept).
    ops: Vec<crate::joinery::JointOp>,
    /// A tenon or tails: at the part's end.
    at_end: bool,
}

/// The cuts joints made on a part's design, one each.
fn cuts(scene: &Scene, def: DefinitionId) -> Vec<Cut> {
    let name = |d: DefinitionId| scene.definition(d).map_or_else(|| "another part".to_owned(), |d| d.name.clone());
    // Joints are kept per pair of parts placed, so a design made as copies (four legs from one
    // component, turned to each corner) has one for every copy's meeting. Each cut on this
    // design is one, whichever parts it takes (a leg's mortise holds the front apron on one
    // corner and a side apron on the next).
    let mut cuts: Vec<(Vec<crate::joinery::JointOp>, Vec<&crate::joinery::Joint>)> = Vec::new();
    for joint in scene.joints.iter().filter(|j| j.cut == def || j.into == def) {
        let here: Vec<crate::joinery::JointOp> = joint.ops.iter().filter(|op| op.part == def).cloned().collect();
        // Joints without their cuts kept (or none on this part): one per kind, part and size.
        let same = |(ops, joints): &&mut (Vec<crate::joinery::JointOp>, Vec<&crate::joinery::Joint>)| {
            if here.is_empty() {
                ops.is_empty() && joints.iter().any(|j| j.kind == joint.kind && j.cut == joint.cut && j.into == joint.into && j.detail == joint.detail)
            } else {
                *ops == here
            }
        };
        match cuts.iter_mut().find(|c| same(c)) {
            Some((_, joints)) => joints.push(joint),
            None => cuts.push((here, vec![joint])),
        }
    }
    let mut out = Vec::new();
    for (ops, joints) in cuts {
        let joint = joints[0];
        // Which parts it takes, and on how many of the copies (one leg's mortise holds a front
        // apron on two legs and a side apron on the other two).
        let mut others: Vec<(String, usize)> = Vec::new();
        for j in joints.iter().filter(|j| j.kind == joint.kind && (j.into == def) == (joint.into == def) && j.detail == joint.detail) {
            let other = name(if j.cut == def { j.into } else { j.cut });
            match others.iter_mut().find(|(o, _)| *o == other) {
                Some((_, n)) => *n += 1,
                None => others.push((other, 1)),
            }
        }
        let other = others.iter().map(|(o, _)| o.as_str()).collect::<Vec<_>>().join(" or ");
        // The list of where the cuts are says which copies take which.
        let counted = match others.as_slice() {
            [(one, _)] => one.clone(),
            many => many.iter().map(|(o, n)| format!("{o} on {n}")).collect::<Vec<_>>().join(", "),
        };
        let kind = joint.kind.name().to_lowercase();
        let step = match (joint.kind, joint.into == def) {
            (JointKind::MortiseAndTenon, true) => format!("Cut the mortise for {other} ({}).", joint.detail),
            (JointKind::MortiseAndTenon, false) => format!("Cut the tenon that goes into {other} ({}).", joint.detail),
            (JointKind::Dowels, _) => format!("Drill for dowels to join {other} ({}).", joint.detail),
            (JointKind::Biscuits, _) => format!("Cut biscuit slots to join {other} ({}).", joint.detail),
            (JointKind::Dovetails, true) => format!("Cut the pins for {other}'s dovetails ({}).", joint.detail),
            (JointKind::Dovetails, false) => format!("Cut the dovetails that go into {other} ({}); its length includes them.", joint.detail),
            (JointKind::Splines, _) => format!("Cut the spline groove to join {other} ({}).", joint.detail),
            (JointKind::LooseTenons, _) => format!("Cut loose tenon mortises to join {other} ({}).", joint.detail),
            (JointKind::PocketScrews, false) => format!("Drill pocket holes to screw into {other} ({}).", joint.detail),
            (JointKind::PocketScrews, true) => format!("{other} is pocket-screwed to it ({}).", joint.detail),
            (JointKind::BoxJoint | JointKind::HalfLap, _) => format!("Cut the {kind} with {other} ({}).", joint.detail),
            (_, true) => format!("Cut the {kind} for {other} ({}).", joint.detail),
            // The part that fits into another's dado or rabbet: already sized for it.
            (_, false) => format!("Fits into the {kind} in {other} ({}); its size includes that.", joint.detail),
        };
        let what = format!("{}, {counted}", joint.kind.name());
        // A part let into another (a shelf in a dado) has no cut of its own: its size includes it.
        let ops = if step.starts_with("Fits into") { Vec::new() } else { ops };
        out.push(Cut { step, what, ops, at_end: joint.cut == def && matches!(joint.kind, JointKind::MortiseAndTenon | JointKind::Dovetails) });
    }
    out
}

/// The three views, to scale: the face (length across, width up), the edge below it (length
/// across, thickness up) and the end beside that (width across, thickness up), each with its
/// overall dimensions; a glue-up's boards (`(how many, the part's letter)`) are drawn on them. The
/// reference face and edge are marked and the cuts numbered, measured from the edge.
fn views(model: &Model, scale: DVec3, [l, w, t]: [usize; 3], [length, width, thickness]: [f64; 3], boards: Option<(usize, &str)>, marks: &Marks) -> String {
    const ROOM: f64 = 640.0;
    const GAP: f64 = 60.0; // between views, and room for dimensions
    const ROW: f64 = 16.0; // between rows of a cut's dimensions
    let lo = model.bounds().map_or(DVec3::ZERO, |b| b.0);
    // Points in the part's own scaled axes, from its corner.
    let at = |p: DVec3| (p - lo) * scale;
    let s = (ROOM / (length + width)).min(260.0 / (width + thickness)).min(40.0);
    let segments = drawn_edges(model);

    let face_h = width * s;
    let edge_h = thickness * s;
    // Above the face view, a row of dimensions for each cut that isn't the full length.
    let near = |a: f64, b: f64| (a - b).abs() < 1.0 / 64.0;
    // Cuts at the same place along it share a row (a leg's two mortises at one corner).
    let mut rows: Vec<Vec<usize>> = Vec::new();
    for (n, (lo, hi)) in marks.cuts.iter().enumerate() {
        if near(lo[l], 0.0) && near(hi[l], length) {
            continue;
        }
        match rows.iter_mut().find(|r| near(marks.cuts[r[0]].0[l], lo[l]) && near(marks.cuts[r[0]].1[l], hi[l])) {
            Some(row) => row.push(n),
            None => rows.push(vec![n]),
        }
    }
    let (x0, y_face) = (GAP, GAP + ROW * rows.len() as f64);
    let y_edge = y_face + face_h + GAP;
    let x_end = x0 + length * s + GAP;
    let total_w = x_end + width * s + GAP;
    let total_h = y_edge + edge_h + GAP * 0.8;

    let triangles: Vec<[DVec3; 3]> = model.faces().flat_map(|(_, f)| model.triangulate(f)).map(|t| t.map(at)).collect();

    let mut svg = format!(r#"<svg width="{total_w:.0}" height="{total_h:.0}" viewBox="0 0 {total_w:.1} {total_h:.1}" xmlns="http://www.w3.org/2000/svg">"#);
    // Each view: which axes go across and up, which one it looks along and from which side, and
    // where its corner is (SVG y grows down). Edges behind the part's faces are dashed.
    let view = |svg: &mut String, across: usize, up: usize, (depth, toward): (usize, f64), ox: f64, oy: f64, height: f64| {
        let flat = |p: DVec3| DVec2::new(ox + p[across] * s, oy + height - p[up] * s);
        let mut hidden_lines = String::new();
        let mut seen_lines = String::new();
        for [a, b] in &segments {
            let (pa, pb) = (at(*a), at(*b));
            let (fa, fb) = (flat(pa), flat(pb));
            if fa.distance(fb) < 0.3 {
                continue;
            }
            for (from, to, hidden) in runs(pa, pb, fa.distance(fb), |p| behind(&triangles, p, across, up, depth, toward)) {
                let (fa, fb) = (flat(pa.lerp(pb, from)), flat(pa.lerp(pb, to)));
                let out = if hidden { &mut hidden_lines } else { &mut seen_lines };
                let _ = write!(out, r#"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}"/>"#, fa.x, fa.y, fb.x, fb.y);
            }
        }
        // Hidden first, so a seen edge in front of a hidden one is drawn over it.
        let _ = write!(svg, r##"<g class="hidden" stroke="#888" stroke-width="0.8" stroke-dasharray="4 3">{hidden_lines}</g>"##);
        let _ = write!(svg, r##"<g stroke="#222" stroke-width="1">{seen_lines}</g>"##);
    };
    // Third-angle: the face from above, the edge below it from the front, the end from the right.
    view(&mut svg, l, w, (t, 1.0), x0, y_face, face_h);
    view(&mut svg, l, t, (w, -1.0), x0, y_edge, edge_h);
    view(&mut svg, w, t, (l, 1.0), x_end, y_edge, edge_h);
    // A glue-up: the boards it's made from, as the cutting diagrams name them (B-1, B-2...), their
    // seams along the face and across the end, and in the end, each board's growth rings
    // alternating (up, then down).
    if let Some((n, label)) = boards.filter(|(n, _)| *n > 1) {
        let strip = width / n as f64;
        let mut glue = String::new();
        for k in 1..n {
            let y = y_face + face_h - k as f64 * strip * s;
            let x = x_end + k as f64 * strip * s;
            let _ = write!(glue, r#"<line x1="{x0:.2}" y1="{y:.2}" x2="{:.2}" y2="{y:.2}"/><line x1="{x:.2}" y1="{y_edge:.2}" x2="{x:.2}" y2="{:.2}"/>"#, x0 + length * s, y_edge + edge_h);
        }
        let _ = write!(svg, r##"<g class="glue-up" stroke="#b0703a" stroke-width="1">{glue}</g>"##);
        for k in 0..n {
            let y = y_face + face_h - (k as f64 + 0.5) * strip * s;
            let _ = write!(svg, r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" fill="#b0703a">{}-{}</text>"##, x0 + length * s / 2.0, y + 4.0, esc(label), k + 1);
            // Growth rings: an arc across the board's end, bowed up or down in turn.
            let (left, right) = (x_end + k as f64 * strip * s, x_end + (k + 1) as f64 * strip * s);
            let middle = y_edge + edge_h / 2.0;
            let bow = if k % 2 == 0 { -edge_h * 0.35 } else { edge_h * 0.35 };
            let _ = write!(
                svg,
                r##"<path class="rings" d="M{:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2}" fill="none" stroke="#b0703a" stroke-width="0.8"/>"##,
                left + strip * s * 0.15,
                middle - bow / 2.0,
                (left + right) / 2.0,
                middle + bow,
                right - strip * s * 0.15,
                middle - bow / 2.0
            );
        }
    }
    // The reference edge (on the face view) and face (on the edge view), in blue, and each cut
    // numbered as in the list below the views, with how far it is from the reference edge.
    let blue = "#2a5ac8";
    let edge_far = marks.edge.filter(|e| e.axis.index() == w).is_some_and(|e| e.positive);
    let face_far = marks.face.filter(|f| f.axis.index() == t).is_some_and(|f| f.positive);
    let edge_y = if edge_far { y_face } else { y_face + face_h };
    let face_y = if face_far { y_edge } else { y_edge + edge_h };
    if marks.edge.is_some() {
        let _ = write!(svg, r##"<line class="reference" x1="{x0:.1}" y1="{edge_y:.1}" x2="{:.1}" y2="{edge_y:.1}" stroke="{blue}" stroke-width="3"/>"##, x0 + length * s);
        let _ = write!(svg, r##"<text x="{:.1}" y="{:.1}" fill="{blue}">ref. edge</text>"##, x0 + length * s + 4.0, edge_y + 4.0);
    }
    if marks.face.is_some() {
        let _ = write!(svg, r##"<line class="reference" x1="{x0:.1}" y1="{face_y:.1}" x2="{:.1}" y2="{face_y:.1}" stroke="{blue}" stroke-width="3"/>"##, x0 + length * s);
        let _ = write!(svg, r##"<text x="{:.1}" y="{:.1}" fill="{blue}">ref. face</text>"##, x0 + length * s + 4.0, face_y + 4.0);
    }
    let mut across_used: Vec<f64> = Vec::new();
    for (n, (lo, hi)) in marks.cuts.iter().enumerate() {
        let (cx, cy) = (x0 + (lo[l] + hi[l]) / 2.0 * s, y_face + face_h - (lo[w] + hi[w]) / 2.0 * s);
        let _ = write!(
            svg,
            r##"<g class="cut-number"><circle cx="{cx:.1}" cy="{cy:.1}" r="7" fill="white" stroke="{blue}"/><text x="{cx:.1}" y="{:.1}" text-anchor="middle" fill="{blue}">{}</text></g>"##,
            cy + 4.0,
            n + 1
        );
        // From the reference edge (or the bottom edge) to the cut's near side, just left of it.
        let near = if edge_far { y_face + face_h - hi[w] * s } else { y_face + face_h - lo[w] * s };
        if (near - edge_y).abs() > 2.0 {
            // Stepped aside from another cut's at the same place.
            let mut x = (x0 + lo[l] * s - 8.0).max(x0 + 4.0);
            while across_used.iter().any(|u: &f64| (u - x).abs() < 1.0) {
                x -= 26.0;
            }
            across_used.push(x);
            dimension(&mut svg, DVec2::new(x, edge_y), DVec2::new(x, near), &format_length((near - edge_y).abs() / s));
        }
    }
    // The woodworker's marks: the face mark's loop on the face view, its tail to the reference
    // edge; the edge mark's caret on the edge view, pointing to the reference face.
    let polyline = |svg: &mut String, points: Vec<DVec2>, class: &str| {
        let d: Vec<String> = points.iter().map(|p| format!("{:.1},{:.1}", p.x, p.y)).collect();
        let _ = write!(svg, r##"<polyline class="{class}" points="{}" fill="none" stroke="{blue}" stroke-width="1.5"/>"##, d.join(" "));
    };
    if marks.face.is_some() {
        let tail = if edge_far { width * 0.45 } else { -width * 0.45 };
        let mark = crate::part::face_mark(length.min(width) * 0.12, tail);
        polyline(&mut svg, mark.iter().map(|p| DVec2::new(x0 + (length / 2.0 + p.x) * s, y_face + face_h - (width / 2.0 + p.y) * s)).collect(), "face-mark");
    }
    if marks.edge.is_some() {
        let mark = crate::part::edge_mark(thickness * 0.35, if face_far { 1.0 } else { -1.0 });
        polyline(&mut svg, mark.iter().map(|p| DVec2::new(x0 + (length / 2.0 + p.x) * s, y_edge + edge_h - (thickness / 2.0 + p.y) * s)).collect(), "edge-mark");
    }
    // Edge profiles: their edges in orange, where a view sees them from the front, each named
    // once beside its first edge.
    let orange = "#c8641e";
    let extent = |axis: usize| if axis == l { length } else if axis == w { width } else { thickness };
    let views_on: [(usize, usize, usize, f64, f64, f64, f64); 3] =
        [(l, w, t, 1.0, x0, y_face, face_h), (l, t, w, -1.0, x0, y_edge, edge_h), (w, t, l, 1.0, x_end, y_edge, edge_h)];
    for (label, edges) in &marks.profiles {
        let mut named = false;
        for &(across, up, depth, toward, ox, oy, height) in &views_on {
            let flat = |p: DVec3| DVec2::new(ox + p[across] * s, oy + height - p[up] * s);
            let front = if toward > 0.0 { extent(depth) } else { 0.0 };
            for [a, b] in edges {
                let (pa, pb) = (at(*a), at(*b));
                let (fa, fb) = (flat(pa), flat(pb));
                if fa.distance(fb) < 2.0 || !(near(pa[depth], front) && near(pb[depth], front)) {
                    continue;
                }
                let _ = write!(svg, r##"<line class="profile" x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{orange}" stroke-width="3" opacity="0.8"/>"##, fa.x, fa.y, fb.x, fb.y);
                if !named {
                    named = true;
                    let mid = (fa + fb) / 2.0;
                    let _ = write!(svg, r##"<text x="{:.1}" y="{:.1}" fill="{orange}" text-anchor="middle">{}</text>"##, mid.x, mid.y + 14.0, esc(label));
                }
            }
        }
    }
    // The reference end (or the left end as drawn), on the face view.
    let end_far = marks.end.filter(|e| e.axis.index() == l).is_some_and(|e| e.positive);
    let end_x = if end_far { x0 + length * s } else { x0 };
    if marks.end.is_some() {
        let _ = write!(svg, r##"<line class="reference" x1="{end_x:.1}" y1="{y_face:.1}" x2="{end_x:.1}" y2="{:.1}" stroke="{blue}" stroke-width="3"/>"##, y_face + face_h);
        let _ = write!(svg, r##"<text x="{end_x:.1}" y="{:.1}" text-anchor="middle" fill="{blue}">ref. end</text>"##, y_face + face_h + 26.0);
    }
    // Each cut's place along the length, a row apiece above the face (numbered as on the face):
    // from the end to its near side, then how long it is; and on the edge view, how deep it goes
    // from the reference face (unless it goes through).
    for (row, cuts) in rows.iter().enumerate() {
        let (lo, hi) = marks.cuts[cuts[0]];
        let y = y_face - 16.0 - ROW * (row + 1) as f64;
        let (from, to) = (x0 + lo[l] * s, x0 + hi[l] * s);
        let close_side = if end_far { to } else { from };
        let _ = write!(svg, r##"<text x="{:.1}" y="{:.1}" text-anchor="end" fill="{blue}">{}</text>"##, x0 - 22.0, y + 4.0, cuts.iter().map(|n| (n + 1).to_string()).collect::<Vec<_>>().join(", "));
        if (close_side - end_x).abs() > 2.0 {
            dimension(&mut svg, DVec2::new(end_x.min(close_side), y), DVec2::new(end_x.max(close_side), y), &format_length((close_side - end_x).abs() / s));
        }
        dimension(&mut svg, DVec2::new(from, y), DVec2::new(to, y), &format_length(hi[l] - lo[l]));
    }
    // Depths, from the face each cut opens on (a cut through, or inside, like a mortise in an
    // edge, has none here), beside the cut; each depth once per place.
    let mut drawn: Vec<(f64, f64, f64)> = Vec::new();
    for (lo, hi) in &marks.cuts {
        let y = |v: f64| y_edge + edge_h - v * s;
        let (open, far) = match (near(lo[t], 0.0), near(hi[t], thickness)) {
            (true, false) => (y(0.0), y(hi[t])),
            (false, true) => (y(thickness), y(lo[t])),
            _ => continue,
        };
        let x = if x0 + lo[l] * s - 8.0 > x0 + 4.0 { x0 + lo[l] * s - 6.0 } else { x0 + hi[l] * s + 6.0 };
        if (far - open).abs() <= 2.0 || drawn.iter().any(|&(dx, a, b)| (dx - x).abs() < 1.0 && (a - open).abs() < 0.5 && (b - far).abs() < 0.5) {
            continue;
        }
        drawn.push((x, open, far));
        dimension(&mut svg, DVec2::new(x, open.max(far)), DVec2::new(x, open.min(far)), &format_length((far - open).abs() / s));
    }
    // Overall dimensions.
    dimension(&mut svg, DVec2::new(x0, y_face - 16.0), DVec2::new(x0 + length * s, y_face - 16.0), &format_length(length));
    dimension(&mut svg, DVec2::new(x0 - 16.0, y_face + face_h), DVec2::new(x0 - 16.0, y_face), &format_length(width));
    dimension(&mut svg, DVec2::new(x0 - 16.0, y_edge + edge_h), DVec2::new(x0 - 16.0, y_edge), &format_length(thickness));
    dimension(&mut svg, DVec2::new(x_end, y_edge + edge_h + 16.0), DVec2::new(x_end + width * s, y_edge + edge_h + 16.0), &format_length(width));
    for (text, x, y) in [("Face", x0, y_face + face_h + 14.0), ("Edge", x0, y_edge + edge_h + 30.0), ("End", x_end, y_edge - 8.0)] {
        let _ = write!(svg, r##"<text x="{x:.1}" y="{y:.1}" fill="#777">{text}</text>"##);
    }
    svg.push_str("</svg>\n");
    svg
}

/// A segment split where it goes in and out of sight: `(from, to, hidden)` as fractions along it,
/// tested every pixel or so of its `length` on the page.
fn runs(a: DVec3, b: DVec3, length: f64, hidden: impl Fn(DVec3) -> bool) -> Vec<(f64, f64, bool)> {
    let steps = (length / 1.5).ceil().clamp(1.0, 400.0) as usize;
    let mut out: Vec<(f64, f64, bool)> = Vec::new();
    for i in 0..steps {
        let (from, to) = (i as f64 / steps as f64, (i + 1) as f64 / steps as f64);
        let h = hidden(a.lerp(b, (from + to) / 2.0));
        match out.last_mut() {
            Some(last) if last.2 == h => last.1 = to,
            _ => out.push((from, to, h)),
        }
    }
    out
}

/// Whether a face of the part is between `p` and the eye, looking along axis `depth` from its
/// `toward` (+1 or -1) side: a triangle covering `p` across the view (not just at its edge) and
/// nearer the eye.
fn behind(triangles: &[[DVec3; 3]], p: DVec3, across: usize, up: usize, depth: usize, toward: f64) -> bool {
    const EPS: f64 = 1e-4;
    let q = DVec2::new(p[across], p[up]);
    triangles.iter().any(|t| {
        let [a, b, c] = t.map(|v| DVec2::new(v[across], v[up]));
        let area = (b - a).perp_dot(c - a);
        if area.abs() < 1e-12 {
            return false;
        }
        // Barycentric weights, all inside (margins scaled to the triangle's size).
        let (wa, wb) = ((b - q).perp_dot(c - q) / area, (c - q).perp_dot(a - q) / area);
        let wc = 1.0 - wa - wb;
        let margin = 1e-4;
        if wa <= margin || wb <= margin || wc <= margin {
            return false;
        }
        let d = wa * t[0][depth] + wb * t[1][depth] + wc * t[2][depth];
        (d - p[depth]) * toward > EPS
    })
}

/// A dimension line from `a` to `b` with end ticks and its text.
fn dimension(svg: &mut String, a: DVec2, b: DVec2, text: &str) {
    let dir = (b - a).normalize_or_zero();
    let tick = DVec2::new(-dir.y, dir.x) * 4.0;
    let _ = write!(svg, r##"<g stroke="#888" stroke-width="0.8"><line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}"/>"##, a.x, a.y, b.x, b.y);
    for p in [a, b] {
        let _ = write!(svg, r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}"/>"#, p.x - tick.x, p.y - tick.y, p.x + tick.x, p.y + tick.y);
    }
    svg.push_str("</g>");
    let mid = (a + b) / 2.0;
    if dir.x.abs() >= dir.y.abs() {
        let _ = write!(svg, r#"<text x="{:.1}" y="{:.1}" text-anchor="middle">{}</text>"#, mid.x, mid.y - 4.0, esc(text));
    } else if a.distance(b) < 32.0 {
        // Too short to write along: level, beside it.
        let _ = write!(svg, r#"<text x="{:.1}" y="{:.1}" text-anchor="end">{}</text>"#, mid.x - 6.0, mid.y + 4.0, esc(text));
    } else {
        let _ = write!(
            svg,
            r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" transform="rotate(-90 {:.1} {:.1})">{}</text>"#,
            mid.x - 5.0,
            mid.y,
            mid.x - 5.0,
            mid.y,
            esc(text)
        );
    }
}

/// The model's edges worth drawing: not the soft ones inside a curve (faces meeting under 30°,
/// or softened in SketchUp) or hidden ones.
fn drawn_edges(model: &Model) -> Vec<[DVec3; 2]> {
    let soft = 30f64.to_radians().cos();
    model
        .edges()
        .filter(|(_, e)| !e.hidden)
        .filter(|(id, _)| {
            let faces = model.edge_faces(*id);
            match faces[..] {
                [a, b] => match (model.face(a), model.face(b)) {
                    (Some(fa), Some(fb)) => fa.normal.dot(fb.normal) < soft,
                    _ => true,
                },
                _ => true,
            }
        })
        .map(|(_, e)| model.edge_points(e))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::library::LIBRARY;
    use crate::{Document, InstanceId};

    fn board(doc: &mut Document, origin: DVec3, length_dir: DVec3, width_dir: DVec3, [l, w, t]: [f64; 3]) -> InstanceId {
        let oak = doc.add_preset(LIBRARY.iter().find(|p| p.name == "Red Oak").unwrap());
        doc.add_board(&Board { origin, length_dir, width_dir, length: l, width: w, thickness: t }, Some(oak)).unwrap()
    }

    #[test]
    fn a_sheet_per_part_with_size_views_and_steps_in_order() {
        let mut doc = Document::default();
        let side = board(&mut doc, DVec3::ZERO, DVec3::Z, DVec3::Y, [30.0, 11.25, 0.75]);
        let shelf = board(&mut doc, DVec3::new(0.0, 0.0, 12.0), DVec3::X, DVec3::Y, [20.0, 11.25, 0.75]);
        doc.rename_instance(side, "Side");
        doc.rename_instance(shelf, "Shelf");
        doc.let_in(shelf, side, 0.25).unwrap();
        let list = crate::cutlist::cut_list(doc.scene());
        let html = to_html(doc.scene(), &list, "Bookcase");
        assert_eq!(html.matches("<section class=\"sheet\">").count(), 2, "one sheet per part");
        assert!(html.contains("Side") && html.contains("Shelf"));
        assert!(html.contains("30\" x 11 1/4\" x 3/4\""), "the side's finished size");
        assert!(html.contains("20 1/4\" x 11 1/4\" x 3/4\""), "the shelf includes its let-in");
        let dashed = "stroke-dasharray=\"4 3\"><line";
        assert!(html.contains(dashed), "the dado's far edges, out of sight in a view, are dashed");
        let plain = {
            let mut doc = Document::default();
            board(&mut doc, DVec3::ZERO, DVec3::X, DVec3::Y, [20.0, 11.25, 0.75]);
            to_html(doc.scene(), &crate::cutlist::cut_list(doc.scene()), "Plain")
        };
        assert!(!plain.contains(dashed), "a plain board hides nothing");
        assert!(html.contains("Cut the dado for Shelf (1/4\" deep)."), "the side's step");
        assert!(html.contains("Fits into the dado in Side"), "the shelf's note");
        assert_eq!(html.matches("<svg").count(), 3, "drawings for each part, and the exploded assembly");
        assert!(html.contains("Assemble: The whole piece") && html.contains("Glue"), "how it goes together");
        let mill = html.find("Mill to").unwrap();
        let dado = html.find("Cut the dado").unwrap();
        let sand = html.rfind("Sand.").unwrap();
        assert!(mill < dado && dado < sand, "shop order");
    }

    #[test]
    fn parts_that_arent_wood_get_no_sheet() {
        let mut doc = Document::default();
        let shelf = board(&mut doc, DVec3::ZERO, DVec3::X, DVec3::Y, [20.0, 11.25, 0.75]);
        let glass = board(&mut doc, DVec3::new(0.0, 20.0, 0.0), DVec3::X, DVec3::Y, [20.0, 11.25, 0.25]);
        let other = doc.add_material(crate::MaterialKind::Other);
        doc.paint(&[crate::Entity::Instance(glass)], Some(other));
        let _ = shelf;
        let list = crate::cutlist::cut_list(doc.scene());
        let html = to_html(doc.scene(), &list, "Cabinet");
        assert_eq!(html.matches("<section class=\"sheet\">").count(), 1, "only the shelf");
    }

    #[test]
    fn copies_make_one_sheet_with_a_count_and_profiles_are_steps() {
        let mut doc = Document::default();
        let leg = board(&mut doc, DVec3::ZERO, DVec3::Z, DVec3::X, [28.0, 1.5, 1.5]);
        doc.profile_edges(leg, crate::profile::Profile::Chamfer, 0.125, crate::profile::Edges::Along { axis: 0, corners: [true; 4] }).unwrap();
        doc.selection_mut().extend([crate::Entity::Instance(leg)]);
        doc.group_selection(crate::DefinitionKind::Component);
        let copies: Vec<glam::DAffine3> = (1..4).map(|i| glam::DAffine3::from_translation(DVec3::X * 10.0 * f64::from(i))).collect();
        doc.transform_entities(&[crate::Entity::Instance(leg)], &copies, true).unwrap();
        let list = crate::cutlist::cut_list(doc.scene());
        let html = to_html(doc.scene(), &list, "Table");
        assert_eq!(html.matches("<section class=\"sheet\">").count(), 1);
        assert!(html.contains("Make 4, all the same"));
        assert!(html.contains("Rout the edges: 1/8\" Chamfer, 4 edges."));
        // The chamfered edges drawn on the views, and named.
        assert!(html.contains("class=\"profile\"") && html.contains(">1/8\" chamfer<"), "{html}");
    }

    #[test]
    fn a_wide_top_says_to_glue_it_up_first() {
        let mut doc = Document::default();
        board(&mut doc, DVec3::ZERO, DVec3::X, DVec3::Y, [36.0, 20.0, 0.75]);
        let list = crate::cutlist::cut_list(doc.scene());
        let html = to_html(doc.scene(), &list, "Table");
        assert!(html.contains("Mill 3 boards 3/4\" x ~ 6 15/16\" x 37\" from Red Oak"), "{html}");
        let glue = html.find("Glue them edge to edge").unwrap();
        assert!(glue < html.find("Sand.").unwrap());
        // The boards drawn: two seams on the face and the end, each board named as in the diagrams.
        assert_eq!(html.matches("class=\"rings\"").count(), 3);
        assert!(html.contains(">A-1<") && html.contains(">A-3<") && html.contains("class=\"glue-up\""), "{html}");
    }

    #[test]
    fn cuts_are_placed_from_the_reference_face_and_edge() {
        use crate::part::{PartAxis, Side};
        let mut doc = Document::default();
        let side = board(&mut doc, DVec3::ZERO, DVec3::Z, DVec3::Y, [30.0, 11.25, 0.75]);
        let shelf = board(&mut doc, DVec3::new(0.0, 0.0, 12.0), DVec3::X, DVec3::Y, [20.0, 11.25, 0.75]);
        doc.rename_instance(shelf, "Shelf");
        doc.let_in(shelf, side, 0.25).unwrap();
        let html = |doc: &Document| to_html(doc.scene(), &crate::cutlist::cut_list(doc.scene()), "Case");
        let before = html(&doc);
        assert!(before.contains("Where the cuts are") && before.contains("pick the reference face and edge in Entity Info"), "measured from the drawing until picked");
        doc.edit_part(side, |p| {
            p.reference_face = Some(Side { axis: PartAxis::Z, positive: false });
            p.reference_edge = Some(Side { axis: PartAxis::Y, positive: false });
        });
        let after = html(&doc);
        assert!(after.contains("Dado, Shelf: 12\" from the left end, 3/4\" long; across the full width; 1/4\" deep from the reference face."), "{after}");
        assert_eq!(after.matches("class=\"reference\"").count(), 2, "the face and edge marked");
        assert!(after.contains("class=\"face-mark\"") && after.contains("class=\"edge-mark\""), "the woodworker's marks");
        assert!(after.contains("class=\"cut-number\""));
        // The dado's place along the side and its depth, dimensioned on the views.
        let svg = &after[after.find("Side</h2>").unwrap_or(0)..];
        let svg = &svg[..svg.find("</svg>").unwrap()];
        assert!(svg.contains(">12\"<") && svg.contains(">3/4\"<") && svg.contains(">1/4\"<"), "12\" to the dado, 3/4\" wide, 1/4\" deep: {svg}");
        // Measured from the other end once it's the reference end.
        doc.edit_part(side, |p| p.reference_end = Some(Side { axis: PartAxis::X, positive: true }));
        let from_end = html(&doc);
        assert!(from_end.contains("Dado, Shelf: 17 1/4\" from the reference end, 3/4\" long"), "{from_end}");
        assert_eq!(from_end.matches("class=\"reference\"").count(), 3, "face, edge and end marked");
        // The shelf, let in, has no cut of its own to place.
        let shelf_sheet = &after[after.find("Shelf</h2>").unwrap()..];
        assert!(!shelf_sheet[..shelf_sheet.find("</section>").unwrap()].contains("Where the cuts are"));
    }

    #[test]
    fn parts_alike_but_drawn_a_little_differently_share_a_sheet() {
        let mut doc = Document::default();
        let a = board(&mut doc, DVec3::ZERO, DVec3::X, DVec3::Y, [20.0, 5.0, 0.75]);
        let b = board(&mut doc, DVec3::new(0.0, 10.0, 0.0), DVec3::X, DVec3::Y, [20.0, 5.0, 0.75]);
        doc.rename_instance(a, "Slat");
        doc.rename_instance(b, "Slat");
        // A seam drawn across one's top: its top is two faces now, same shape.
        doc.enter(b);
        doc.edit(|m| m.add_line(DVec3::new(8.0, 0.0, 0.75), DVec3::new(8.0, 5.0, 0.75)));
        assert_eq!(doc.model().faces().count(), 7, "the seam split its top");
        doc.exit();
        let html = to_html(doc.scene(), &crate::cutlist::cut_list(doc.scene()), "Rack");
        assert_eq!(html.matches("Slat</h2>").count(), 1, "one sheet: {html}");
        assert!(html.contains("Make 2, all the same"));
    }

    #[test]
    fn a_left_and_a_right_side_are_one_sheet_a_mirrored_pair() {
        let mut doc = Document::default();
        // Two sides (each its own board), a shelf dadoed into both and a back rabbeted into both.
        let left = board(&mut doc, DVec3::new(0.75, 0.0, 0.0), DVec3::Z, DVec3::Y, [30.0, 11.25, 0.75]);
        let right = board(&mut doc, DVec3::new(21.5, 0.0, 0.0), DVec3::Z, DVec3::Y, [30.0, 11.25, 0.75]);
        let shelf = board(&mut doc, DVec3::new(0.75, 0.0, 12.0), DVec3::X, DVec3::Y, [20.0, 10.75, 0.75]);
        let back = board(&mut doc, DVec3::new(0.75, 11.25, 0.0), DVec3::X, DVec3::Z, [20.0, 30.0, 0.5]);
        for (part, name) in [(left, "Side"), (right, "Side"), (shelf, "Shelf"), (back, "Back")] {
            doc.rename_instance(part, name);
        }
        for side in [left, right] {
            doc.let_in(shelf, side, 0.25).unwrap();
            doc.let_in(back, side, 0.25).unwrap();
        }
        let list = crate::cutlist::cut_list(doc.scene());
        let html = to_html(doc.scene(), &list, "Case");
        assert_eq!(html.matches("Side</h2>").count(), 1, "one sheet for both sides");
        assert!(html.contains("Make 2, a mirrored pair (a left and a right)"), "{html}");
        let side = &html[html.find("Side</h2>").unwrap()..];
        let side = &side[..side.find("</section>").unwrap()];
        assert!(side.matches("for Shelf").count() == 1 && side.matches("for Back").count() == 1, "each side's steps once: {side}");
    }

    #[test]
    fn copies_of_one_leg_turned_to_each_corner_list_each_cut_once() {
        let mut doc = Document::default();
        let cherry = doc.add_preset(LIBRARY.iter().find(|p| p.name == "Cherry").unwrap());
        doc.add_template(&crate::template::Template::Table(crate::template::Table::default()), Some(cherry)).unwrap();
        let list = crate::cutlist::cut_list(doc.scene());
        let html = to_html(doc.scene(), &list, "Table");
        let leg = &html[html.find("Leg</h2>").unwrap()..];
        let leg = &leg[..leg.find("</section>").unwrap()];
        assert_eq!(leg.matches("Cut the mortise").count(), 1, "one step for the leg's two mortises, not eight: {leg}");
        assert!(leg.contains("In 2 places."), "{leg}");
        assert!(leg.contains("Side apron on ") && leg.contains("Front and back apron on "), "which legs take which: {leg}");
        assert!(html.contains("Cut the tenon that goes into Leg") && html.contains("At both ends."));
    }
}
