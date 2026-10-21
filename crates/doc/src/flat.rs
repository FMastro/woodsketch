//! Parts flattened for CNC: each cut-list part seen straight down through its thickness, as the
//! 2D drawing a router cuts from (DXF or SVG). Every face square to the thickness is projected
//! onto the part's face; where they overlap is worked out in a planar arrangement (the same one
//! the Line tool splits faces with), and each region of it is the part's full thickness, a pocket
//! from the face (as deep as the highest face over it is below the top), a pocket from the back,
//! or a through hole. Regions alike are merged into outlines:
//! - the outline to cut out, and through holes;
//! - pockets by depth (dados, grooves, rabbets, mortises in the face, tenon cheeks);
//! - pockets from the back, for after the part is turned over, drawn where they are seen through
//!   it.
//!
//! What a router can't cut from either face (sloped or curved surfaces, like a pocket screw's
//! angled bore or a dovetail's pins, and cuts in the edges or ends, like a mortise in a leg's
//! side) is left out, and each part says so.

use std::collections::BTreeMap;
use std::fmt::Write;

use glam::{DMat3, DVec2, DVec3};

use crate::export::ExportUnits;
use crate::model::{Location, locate};
use crate::units::format_length;
use crate::{Model, Scene, VertexId};

/// Pocket depths closer than this are the same depth.
const DEPTH: f64 = 1e-4;

/// One part, flattened: its 2D outlines in its own length (x) and width (y), seen from its face.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlatPart {
    /// The cut list's letter.
    pub label: String,
    pub name: String,
    pub count: usize,
    /// How many of `count` are mirror images (cut from the drawing turned over).
    pub mirrored: usize,
    pub material: String,
    /// Length, width and thickness.
    pub size: [f64; 3],
    /// Cut all the way round: each outside outline, counter-clockwise.
    pub outline: Vec<Vec<DVec2>>,
    /// Cut all the way through, inside the outline: each hole, clockwise.
    pub holes: Vec<Vec<DVec2>>,
    /// Cut partway from the face, by depth: each pocket's loops (its outline, and islands left
    /// standing in it).
    pub pockets: Vec<(f64, Vec<Vec<DVec2>>)>,
    /// Cut partway from the back, by depth.
    pub back_pockets: Vec<(f64, Vec<Vec<DVec2>>)>,
    /// What a router can't cut from either face, and isn't drawn.
    pub left_out: Vec<&'static str>,
}

/// Every part in the cut list, flattened, in cut-list order.
pub fn flat_parts(scene: &Scene) -> Vec<FlatPart> {
    let list = crate::cutlist::cut_list(scene);
    let mut out = Vec::new();
    for (label, sheet, row) in crate::buildsheet::sheets(scene, &list) {
        let Some(def) = scene.definition(sheet.definition) else { continue };
        let Some(extents) = crate::part::extents(def, DMat3::from_diagonal(sheet.scale)) else { continue };
        let [l, w, t] = crate::part::axes(extents, &def.part);
        // A square part (a leg) lies either way: on the side that shows the most.
        let flats = |axes| flatten(&def.container.model, sheet.scale, axes);
        let mut part = flats([l, w, t]);
        if (extents[w] - extents[t]).abs() < 1e-6 && !part.left_out.is_empty() {
            let turned = flats([l, t, w]);
            if turned.left_out.len() < part.left_out.len() {
                part = turned;
            }
        }
        part.label = label;
        part.name = def.name.clone();
        part.count = sheet.count;
        part.mirrored = sheet.mirrored;
        part.material = sheet.material.and_then(|m| scene.material(m)).map_or_else(|| "No material".to_owned(), |m| m.name.clone());
        if let Some(row) = row {
            part.size = [row.length, row.width, row.thickness];
        }
        out.push(part);
    }
    out
}

/// A face square to the thickness, seen from above: facing up (the top of the part there) or down.
struct Level {
    up: bool,
    height: f64,
    loops: Vec<Vec<DVec2>>,
    /// A point inside it.
    inside: DVec2,
}

impl Level {
    fn covers(&self, q: DVec2) -> bool {
        locate(q, &self.loops[0]) == Location::Inside && self.loops[1..].iter().all(|h| locate(q, h) != Location::Inside)
    }
}

/// `model` (stretched by `scale`) flattened along its axes `[length, width, thickness]`.
fn flatten(model: &Model, scale: DVec3, [l, w, t]: [usize; 3]) -> FlatPart {
    let mut part = FlatPart::default();
    let Some((lo, hi)) = model.bounds() else { return part };
    let size = (hi - lo) * scale;
    part.size = [size[l], size[w], size[t]];
    let thick = size[t];
    let flat = |p: DVec3| {
        let q = (p - lo) * scale;
        (DVec2::new(q[l], q[w]), q[t])
    };

    let mut levels: Vec<Level> = Vec::new();
    let mut sloped = false;
    for (_, face) in model.faces() {
        let up = face.normal[t];
        if up.abs() < 1e-6 {
            continue;
        }
        if up.abs() < 1.0 - 1e-6 {
            sloped = true;
            continue;
        }
        let loops: Vec<Vec<DVec2>> = face.loops().map(|l| model.loop_points(l).into_iter().map(|p| flat(p).0).collect()).collect();
        let height = flat(model.vertex(face.outer[0])).1;
        // A point inside: the middle of its biggest triangle.
        let Some(inside) = model
            .triangulate(face)
            .into_iter()
            .map(|tri| tri.map(|p| flat(p).0))
            .max_by(|a, b| area(a).total_cmp(&area(b)))
            .map(|[a, b, c]| (a + b + c) / 3.0)
        else {
            continue;
        };
        levels.push(Level { up: up > 0.0, height, loops, inside });
    }
    // A face under another facing the same way is inside the part (a mortise in its edge, a
    // tenon's shoulder in its end): not seen from either face.
    let hidden = levels.iter().any(|a| {
        levels.iter().any(|b| a.up == b.up && b.covers(a.inside) && if a.up { b.height > a.height + DEPTH } else { b.height < a.height - DEPTH })
    });
    if sloped {
        part.left_out.push("angled or curved cuts");
    }
    if hidden {
        part.left_out.push("cuts in its edges or ends");
    }

    // Every face's outlines on one plane: the regions they make.
    let mut plane = Model::default();
    for level in &levels {
        for points in &level.loops {
            for i in 0..points.len() {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                plane.add_line(a.extend(0.0), b.extend(0.0));
            }
        }
    }
    // Each region: wood all the way through, a pocket from the face or the back, or a hole.
    let mut material: Vec<Region> = Vec::new();
    let mut pockets: BTreeMap<i64, Vec<Region>> = BTreeMap::new();
    let mut back: BTreeMap<i64, Vec<Region>> = BTreeMap::new();
    let key = |d: f64| (d / DEPTH).round() as i64;
    for (_, face) in plane.faces() {
        let Some(q) = plane.triangulate(face).into_iter().map(|tri| tri.map(|p| p.truncate())).max_by(|a, b| area(a).total_cmp(&area(b))).map(|[a, b, c]| (a + b + c) / 3.0)
        else {
            continue;
        };
        let top = levels.iter().filter(|lv| lv.up && lv.covers(q)).map(|lv| lv.height).reduce(f64::max);
        let bottom = levels.iter().filter(|lv| !lv.up && lv.covers(q)).map(|lv| lv.height).reduce(f64::min);
        if top.is_none() && bottom.is_none() {
            continue;
        }
        let region: Region = (face.outer.clone(), face.holes.clone());
        material.push(region.clone());
        if let Some(top) = top
            && thick - top > DEPTH
        {
            pockets.entry(key(thick - top)).or_default().push(region.clone());
        }
        if let Some(bottom) = bottom
            && bottom > DEPTH
        {
            back.entry(key(bottom)).or_default().push(region);
        }
    }
    let at = |v: VertexId| plane.vertex(v).truncate();
    for l in merge(&material, at) {
        if signed_area(&l) > 0.0 {
            part.outline.push(l);
        } else {
            part.holes.push(l);
        }
    }
    part.pockets = pockets.into_iter().map(|(d, loops)| (d as f64 * DEPTH, merge(&loops, at))).collect();
    part.back_pockets = back.into_iter().map(|(d, loops)| (d as f64 * DEPTH, merge(&loops, at))).collect();
    part
}

fn area([a, b, c]: &[DVec2; 3]) -> f64 {
    ((b - a).perp_dot(c - a) / 2.0).abs()
}

fn signed_area(points: &[DVec2]) -> f64 {
    (0..points.len()).map(|i| points[i].perp_dot(points[(i + 1) % points.len()])).sum::<f64>() / 2.0
}

/// A region of the arrangement: its outline and holes.
type Region = (Vec<VertexId>, Vec<Vec<VertexId>>);

/// The outline of `regions` together: outer loops counter-clockwise, holes clockwise, edges the
/// regions share taken out, and corners on straight lines dropped.
fn merge(regions: &[Region], at: impl Fn(VertexId) -> DVec2) -> Vec<Vec<DVec2>> {
    let wound = |l: &[VertexId], ccw: bool| -> Vec<VertexId> {
        let points: Vec<DVec2> = l.iter().map(|&v| at(v)).collect();
        if (signed_area(&points) > 0.0) == ccw { l.to_vec() } else { l.iter().rev().copied().collect() }
    };
    // Shared edges run opposite ways in the two regions, and cancel.
    let mut edges: BTreeMap<(VertexId, VertexId), i32> = BTreeMap::new();
    for (outer, holes) in regions {
        for ids in std::iter::once(wound(outer, true)).chain(holes.iter().map(|h| wound(h, false))) {
            for i in 0..ids.len() {
                let (a, b) = (ids[i], ids[(i + 1) % ids.len()]);
                match edges.get_mut(&(b, a)) {
                    Some(n) if *n > 0 => *n -= 1,
                    _ => *edges.entry((a, b)).or_default() += 1,
                }
            }
        }
    }
    let mut out: BTreeMap<VertexId, Vec<VertexId>> = BTreeMap::new();
    for (&(a, b), &n) in &edges {
        for _ in 0..n.max(0) {
            out.entry(a).or_default().push(b);
        }
    }
    let mut result = Vec::new();
    while let Some((&start, _)) = out.iter().find(|(_, to)| !to.is_empty()) {
        let mut l = vec![start];
        let (mut here, mut from) = (start, None::<VertexId>);
        while let Some(choices) = out.get_mut(&here) {
            if choices.is_empty() {
                break;
            }
            // Where outlines touch at a corner, keep to this one: the sharpest left turn.
            let pick = match from {
                Some(f) if choices.len() > 1 => {
                    let d = at(here) - at(f);
                    let turn = |c: VertexId| {
                        let e = at(c) - at(here);
                        d.perp_dot(e).atan2(d.dot(e))
                    };
                    (0..choices.len()).max_by(|&x, &y| turn(choices[x]).total_cmp(&turn(choices[y]))).unwrap_or(0)
                }
                _ => 0,
            };
            let next = choices.remove(pick);
            if next == start {
                break;
            }
            from = Some(here);
            here = next;
            l.push(here);
        }
        let points: Vec<DVec2> = l.iter().map(|&v| at(v)).collect();
        let points = straightened(&points);
        if points.len() >= 3 {
            result.push(points);
        }
    }
    result
}

/// `points` without corners that lie on a straight line.
fn straightened(points: &[DVec2]) -> Vec<DVec2> {
    let n = points.len();
    (0..n)
        .filter(|&i| {
            let (p, c, q) = (points[(i + n - 1) % n], points[i], points[(i + 1) % n]);
            let (a, b) = (c - p, q - c);
            a.perp_dot(b).abs() > 1e-9 * a.length() * b.length() || a.dot(b) < 0.0
        })
        .map(|i| points[i])
        .collect()
}

/// Where each part goes on the sheet: one above the other, each under its label, `gap` apart.
/// Returns each part's lower-left corner and the whole size.
fn layout(parts: &[FlatPart], gap: f64, label: f64) -> (Vec<DVec2>, DVec2) {
    let mut at = Vec::new();
    let mut y = 0.0;
    let mut width: f64 = 0.0;
    // From the top down: the first part highest.
    for part in parts.iter().rev() {
        at.push(DVec2::new(0.0, y));
        y += part.size[1] + label + gap;
        width = width.max(part.size[0]);
    }
    at.reverse();
    (at, DVec2::new(width, (y - gap).max(0.0)))
}

/// Above each part, in inches: its label's baseline and size, then the note's under it.
const LABEL: (f64, f64) = (0.4, 0.3);
const NOTE: (f64, f64) = (0.12, 0.18);

/// The words above a part: its label, and what isn't drawn (if anything).
fn texts(part: &FlatPart) -> Vec<String> {
    let mut words = vec![label(part)];
    if !part.left_out.is_empty() {
        words.push(format!("Not drawn: {}", part.left_out.join(", ")));
    }
    words
}

/// A part's label: `A  Side  x2 (1 mirrored), Plywood 3/4"`.
fn label(part: &FlatPart) -> String {
    let count = match (part.count, part.mirrored) {
        (1, _) => String::new(),
        (n, 0) => format!("  x{n}"),
        (n, m) => format!("  x{n} ({m} mirrored)"),
    };
    format!("{}  {}{count}, {} {}", part.label, part.name, part.material, format_length(part.size[2]))
}

fn depth_text(depth: f64, units: ExportUnits) -> String {
    match units {
        ExportUnits::Inches => format!("{depth:.3}in"),
        ExportUnits::Millimeters => format!("{:.2}mm", depth * 25.4),
    }
}

/// The parts as an SVG drawing, one above the other, at full size in `units`. Cuts are told
/// apart by fill, as shape-cutting programs read them: the outline filled black, through holes
/// white with a black edge, pockets grey (each with its depth in `data-depth`), back pockets as a
/// dashed blue guide, and labels in blue.
pub fn to_svg(parts: &[FlatPart], units: ExportUnits) -> String {
    let s = units.scale();
    let (gap, text) = (1.0, 0.6);
    let (at, size) = layout(parts, gap, text);
    let margin = 0.5;
    let (width, height) = ((size.x + 2.0 * margin) * s, (size.y + 2.0 * margin) * s);
    let unit = match units {
        ExportUnits::Inches => "in",
        ExportUnits::Millimeters => "mm",
    };
    // Y runs down in SVG: flip, so each part reads as seen from its face.
    let point = |corner: DVec2, p: DVec2| DVec2::new((margin + corner.x + p.x) * s, height - (margin + corner.y + p.y) * s);
    let path = |corner: DVec2, loops: &[Vec<DVec2>]| {
        let mut d = String::new();
        for l in loops {
            for (i, p) in l.iter().enumerate() {
                let q = point(corner, *p);
                let _ = write!(d, "{}{:.4},{:.4} ", if i == 0 { "M" } else { "L" }, q.x, q.y);
            }
            d.push_str("Z ");
        }
        d.trim_end().to_owned()
    };
    let stroke = 0.01 * s;
    let mut svg = String::new();
    let _ = writeln!(svg, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.4}{unit}" height="{height:.4}{unit}" viewBox="0 0 {width:.4} {height:.4}">"#
    );
    let _ = writeln!(svg, "<title>Parts for CNC, seen from the face, in {}</title>", units.name());
    for (part, corner) in parts.iter().zip(&at) {
        let _ = writeln!(svg, r#"<g id="part-{}">"#, xml(&part.label));
        for (words, (above, size)) in texts(part).iter().zip([LABEL, NOTE]) {
            let at = point(*corner, DVec2::new(0.0, part.size[1] + above));
            let _ = writeln!(svg, r##"<text x="{:.4}" y="{:.4}" font-family="sans-serif" font-size="{:.4}" fill="#0068ff">{}</text>"##, at.x, at.y, size * s, xml(words));
        }
        let _ = writeln!(svg, r##"<path class="outline" d="{}" fill="#000000" stroke="#000000" stroke-width="{stroke:.4}"/>"##, path(*corner, &part.outline));
        for hole in &part.holes {
            let _ = writeln!(svg, r##"<path class="hole" d="{}" fill="#ffffff" stroke="#000000" stroke-width="{stroke:.4}"/>"##, path(*corner, std::slice::from_ref(hole)));
        }
        for (depth, loops) in &part.pockets {
            let _ = writeln!(
                svg,
                r##"<path class="pocket" data-depth="{}" d="{}" fill="#7f7f7f" fill-rule="evenodd"/>"##,
                depth_text(*depth, units),
                path(*corner, loops)
            );
        }
        for (depth, loops) in &part.back_pockets {
            let _ = writeln!(
                svg,
                r##"<path class="back-pocket" data-depth="{}" d="{}" fill="none" stroke="#0068ff" stroke-width="{stroke:.4}" stroke-dasharray="{:.4}"/>"##,
                depth_text(*depth, units),
                path(*corner, loops),
                0.1 * s
            );
        }
        svg.push_str("</g>\n");
    }
    svg.push_str("</svg>\n");
    svg
}

fn xml(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The parts as a DXF (R12, which CAM programs read): one above the other at full size in
/// `units`, closed polylines on layers by what's cut: `OUTLINE`, `HOLES`, `POCKET_<depth>`,
/// `BACK_POCKET_<depth>` (the depth with `_` for its point, as `POCKET_0_250IN`), and `LABELS`.
pub fn to_dxf(parts: &[FlatPart], units: ExportUnits) -> String {
    let s = units.scale();
    let (gap, text) = (1.0, 0.6);
    let (at, _) = layout(parts, gap, text);
    let layer = |what: &str, depth: f64| format!("{what}_{}", depth_text(depth, units).replace('.', "_").to_uppercase());
    let mut layers: Vec<(String, u8)> = vec![("OUTLINE".into(), 7), ("HOLES".into(), 1), ("LABELS".into(), 5)];
    for part in parts {
        for (depth, _) in &part.pockets {
            let name = layer("POCKET", *depth);
            if !layers.iter().any(|(n, _)| *n == name) {
                layers.push((name, 3));
            }
        }
        for (depth, _) in &part.back_pockets {
            let name = layer("BACK_POCKET", *depth);
            if !layers.iter().any(|(n, _)| *n == name) {
                layers.push((name, 4));
            }
        }
    }
    let mut dxf = String::new();
    let mut pair = |code: i32, value: &str| {
        let _ = write!(dxf, "{code}\n{value}\n");
    };
    pair(0, "SECTION");
    pair(2, "HEADER");
    pair(9, "$ACADVER");
    pair(1, "AC1009");
    // 1 inches, 4 millimeters.
    pair(9, "$INSUNITS");
    pair(70, if units == ExportUnits::Inches { "1" } else { "4" });
    pair(0, "ENDSEC");
    pair(0, "SECTION");
    pair(2, "TABLES");
    // The solid line type the layers use, defined for strict readers.
    pair(0, "TABLE");
    pair(2, "LTYPE");
    pair(70, "1");
    pair(0, "LTYPE");
    pair(2, "CONTINUOUS");
    pair(70, "0");
    pair(3, "Solid line");
    pair(72, "65");
    pair(73, "0");
    pair(40, "0.0");
    pair(0, "ENDTAB");
    pair(0, "TABLE");
    pair(2, "LAYER");
    pair(70, &layers.len().to_string());
    for (name, color) in &layers {
        pair(0, "LAYER");
        pair(2, name);
        pair(70, "0");
        pair(62, &color.to_string());
        pair(6, "CONTINUOUS");
    }
    pair(0, "ENDTAB");
    pair(0, "ENDSEC");
    pair(0, "SECTION");
    pair(2, "ENTITIES");
    let num = |v: f64| format!("{:.6}", v * s);
    for (part, corner) in parts.iter().zip(&at) {
        let mut polyline = |name: &str, points: &[DVec2]| {
            pair(0, "POLYLINE");
            pair(8, name);
            pair(66, "1");
            pair(70, "1");
            pair(10, "0.0");
            pair(20, "0.0");
            pair(30, "0.0");
            for p in points {
                pair(0, "VERTEX");
                pair(8, name);
                pair(10, &num(corner.x + p.x));
                pair(20, &num(corner.y + p.y));
                pair(30, "0.0");
            }
            pair(0, "SEQEND");
            pair(8, name);
        };
        for l in &part.outline {
            polyline("OUTLINE", l);
        }
        for l in &part.holes {
            polyline("HOLES", l);
        }
        for (depth, loops) in &part.pockets {
            for l in loops {
                polyline(&layer("POCKET", *depth), l);
            }
        }
        for (depth, loops) in &part.back_pockets {
            for l in loops {
                polyline(&layer("BACK_POCKET", *depth), l);
            }
        }
        for (words, (above, size)) in texts(part).iter().zip([LABEL, NOTE]) {
            pair(0, "TEXT");
            pair(8, "LABELS");
            pair(10, &num(corner.x));
            pair(20, &num(corner.y + part.size[1] + above));
            pair(30, "0.0");
            pair(40, &num(size));
            pair(1, words);
        }
    }
    pair(0, "ENDSEC");
    pair(0, "EOF");
    dxf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    fn part<'a>(parts: &'a [FlatPart], name: &str) -> &'a FlatPart {
        parts.iter().find(|p| p.name == name).unwrap_or_else(|| panic!("no {name}"))
    }

    fn box_of(points: &[DVec2]) -> (DVec2, DVec2) {
        points.iter().fold((DVec2::splat(f64::MAX), DVec2::splat(f64::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)))
    }

    fn built(recipe: &str) -> Document {
        crate::recipe::build(&crate::recipe::parse(recipe).unwrap()).unwrap_or_else(|e| panic!("{e:?}"))
    }

    #[test]
    fn a_side_with_dados_is_its_outline_and_two_pockets() {
        let doc = built(
            r#"{ "materials": [{ "id": "ply", "library": "Plywood" }],
            "parts": [
                { "name": "Side", "material": "ply", "size": [30, 12, 0.75], "at": [0, 0, 0], "length": "z", "width": "y" },
                { "name": "Shelf", "material": "ply", "size": [20, 12, 0.75], "at": [0.75, 0, 10], "length": "x", "width": "y" },
                { "name": "Shelf 2", "material": "ply", "size": [20, 12, 0.75], "at": [0.75, 0, 20], "length": "x", "width": "y" }
            ],
            "joints": [
                { "kind": "let_in", "part": "Shelf", "into": "Side", "options": { "depth": 0.25 } },
                { "kind": "let_in", "part": "Shelf 2", "into": "Side", "options": { "depth": 0.25 } }
            ] }"#,
        );
        let parts = flat_parts(doc.scene());
        let side = part(&parts, "Side");
        assert_eq!(side.size, [30.0, 12.0, 0.75]);
        assert_eq!(side.outline.len(), 1);
        let (lo, hi) = box_of(&side.outline[0]);
        assert!(lo.abs_diff_eq(DVec2::ZERO, 1e-9) && hi.abs_diff_eq(DVec2::new(30.0, 12.0), 1e-9), "{lo} {hi}");
        assert_eq!(side.outline[0].len(), 4, "a plain rectangle: {:?}", side.outline[0]);
        assert!(side.holes.is_empty());
        // Both dados: 1/4" deep, from one face or the other.
        let dados: Vec<&(f64, Vec<Vec<DVec2>>)> = side.pockets.iter().chain(&side.back_pockets).collect();
        assert_eq!(dados.len(), 1, "{:?} {:?}", side.pockets, side.back_pockets);
        assert!((dados[0].0 - 0.25).abs() < 1e-9);
        assert_eq!(dados[0].1.len(), 2, "two dados");
        for d in &dados[0].1 {
            let (lo, hi) = box_of(d);
            assert!((hi.x - lo.x - 0.75).abs() < 1e-9 && (hi.y - lo.y - 12.0).abs() < 1e-9, "3/4\" wide, right across: {lo} {hi}");
        }
        assert!(side.left_out.is_empty());
        let shelf = part(&parts, "Shelf");
        assert!(shelf.pockets.is_empty() && shelf.back_pockets.is_empty() && shelf.holes.is_empty());
    }

    #[test]
    fn a_tenoned_rail_has_its_tenons_and_cheeks_from_both_faces() {
        let doc = built(
            r#"{ "materials": [{ "id": "oak", "library": "White Oak" }],
            "parts": [
                { "name": "Leg", "material": "oak", "size": [14, 1.75, 1.75], "at": [0, 0, 0], "length": "z", "width": "x" },
                { "name": "Rail", "material": "oak", "size": [8, 4, 0.75], "at": [1.75, 0.5, 9], "length": "x", "width": "z" }
            ],
            "joints": [{ "kind": "mortise_and_tenon", "part": "Rail", "into": "Leg" }] }"#,
        );
        let parts = flat_parts(doc.scene());
        let rail = part(&parts, "Rail");
        // The outline runs out to the tenon's end; its cheeks are pockets from both faces.
        let (lo, hi) = box_of(&rail.outline[0]);
        assert!((hi.x - lo.x - rail.size[0]).abs() < 1e-9 && rail.size[0] > 8.0, "{lo} {hi} {:?}", rail.size);
        assert!(rail.outline[0].len() > 4, "the tenon stands out of the end");
        assert_eq!((rail.pockets.len(), rail.back_pockets.len()), (1, 1), "a cheek each face");
        // The leg's mortise is in its side: in the flat drawing only if seen from its face.
        let leg = part(&parts, "Leg");
        assert!(!leg.pockets.is_empty() || !leg.back_pockets.is_empty() || leg.left_out.contains(&"cuts in its edges or ends"), "{leg:?}");
    }

    #[test]
    fn a_hole_through_and_angled_cuts() {
        let mut doc = Document::default();
        let board = crate::board::Board { origin: DVec3::ZERO, length_dir: DVec3::X, width_dir: DVec3::Y, length: 20.0, width: 10.0, thickness: 0.75 };
        let id = doc.add_board(&board, None).unwrap();
        // A round hole right through.
        let def = doc.container().instance(id).unwrap().definition;
        doc.edit_scene(|scene, _| {
            let model = &mut scene.definition_mut(def).unwrap().container.model;
            let rod = crate::solid::Rod { from: DVec3::new(5.0, 5.0, -1.0), to: DVec3::new(5.0, 5.0, 2.0), radius: 0.5, sides: 16 };
            *model = crate::solid::drill(model, &[rod]).unwrap();
        });
        let parts = flat_parts(doc.scene());
        let p = &parts[0];
        assert_eq!((p.outline.len(), p.holes.len()), (1, 1), "{p:?}");
        assert_eq!(p.holes[0].len(), 16);
        assert!(signed_area(&p.holes[0]) < 0.0, "holes run clockwise");
        assert!(p.left_out.is_empty());
        // And an angled bore: left out, said so.
        doc.edit_scene(|scene, _| {
            let model = &mut scene.definition_mut(def).unwrap().container.model;
            let dir = DVec3::new(1.0, 0.0, -0.3).normalize();
            let rod = crate::solid::Rod { from: DVec3::new(12.0, 5.0, 1.0), to: DVec3::new(12.0, 5.0, 1.0) + dir * 2.0, radius: 0.2, sides: 12 };
            *model = crate::solid::drill(model, &[rod]).unwrap();
        });
        let parts = flat_parts(doc.scene());
        assert!(parts[0].left_out.contains(&"angled or curved cuts"), "{:?}", parts[0].left_out);
    }

    #[test]
    fn svg_and_dxf_carry_the_layers() {
        let doc = built(
            r#"{ "materials": [{ "id": "ply", "library": "Plywood" }],
            "parts": [
                { "name": "Side", "material": "ply", "size": [30, 12, 0.75], "at": [0, 0, 0], "length": "z", "width": "y" },
                { "name": "Shelf", "material": "ply", "size": [20, 12, 0.75], "at": [0.75, 0, 10], "length": "x", "width": "y" }
            ],
            "joints": [{ "kind": "let_in", "part": "Shelf", "into": "Side", "options": { "depth": 0.25 } }] }"#,
        );
        let parts = flat_parts(doc.scene());
        let svg = to_svg(&parts, ExportUnits::Millimeters);
        assert!(svg.contains(r#"width="#) && svg.contains("mm\"") && svg.contains(r#"class="outline""#) && svg.contains(r#"data-depth="6.35mm""#), "{svg}");
        assert!(svg.contains("Side") && svg.contains("Shelf"));
        let dxf = to_dxf(&parts, ExportUnits::Inches);
        assert!(dxf.starts_with("0\nSECTION\n2\nHEADER\n") && dxf.ends_with("0\nEOF\n"));
        let pockets: Vec<&str> = dxf.lines().filter(|l| l.starts_with("POCKET_") || l.starts_with("BACK_POCKET_")).collect();
        assert!(pockets.iter().any(|l| l.ends_with("0_250IN")), "{pockets:?}");
        assert!(dxf.contains("\nOUTLINE\n") && dxf.contains("\nLABELS\n"));
        // Every polyline is closed and ends.
        assert_eq!(dxf.matches("\nPOLYLINE\n").count(), dxf.matches("\nSEQEND\n").count());
    }
}
