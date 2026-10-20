//! Solid subtraction: cut one closed solid out of another (a pocket hole's angled bore, which a
//! straight push/pull can't make). Both solids' faces go into BSP trees and each is clipped by
//! the other (the classic CSG method), then the pieces left are merged back into faces, one per
//! plane and paint, with holes.

use std::collections::{BTreeMap, BTreeSet};

use glam::{DVec2, DVec3};
use serde::{Deserialize, Serialize};

use crate::model::{Location, locate, newell_normal, projector};
use crate::{MaterialId, Model, VertexId};

/// Thicker than this from a plane is in front of or behind it.
const PLANE: f64 = 1e-7;
/// Points closer than this are the same point.
const WELD: f64 = 1e-6;

/// A round rod from `from` to `to` (flat ends square to it): a drill's path.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rod {
    pub from: DVec3,
    pub to: DVec3,
    pub radius: f64,
    pub sides: u32,
}

impl Rod {
    /// Its faces: the two ends and the sides, wound to face out.
    fn faces(&self) -> Vec<Vec<DVec3>> {
        let axis = (self.to - self.from).normalize();
        let u = axis.any_orthonormal_vector();
        let v = axis.cross(u);
        let n = self.sides.max(3) as usize;
        let ring = |center: DVec3| -> Vec<DVec3> {
            (0..n)
                .map(|i| {
                    let a = std::f64::consts::TAU * i as f64 / n as f64;
                    center + (u * a.cos() + v * a.sin()) * self.radius
                })
                .collect()
        };
        let (bottom, top) = (ring(self.from), ring(self.to));
        // Counter-clockwise about `axis`: the top faces along it, the bottom against it.
        let mut faces = vec![top.clone(), bottom.iter().rev().copied().collect()];
        for i in 0..n {
            let j = (i + 1) % n;
            faces.push(vec![bottom[i], bottom[j], top[j], top[i]]);
        }
        faces
    }
}

/// `model` with `rods` cut out of it, or `None` if it isn't a solid that can be cut (no faces
/// left, or the result came apart).
pub fn drill(model: &Model, rods: &[Rod]) -> Option<Model> {
    let mut polys = polygons(model);
    for rod in rods {
        let tool: Vec<Poly> = rod.faces().into_iter().filter_map(|points| Poly::new(points, None)).collect();
        polys = subtract(polys, tool);
    }
    if polys.is_empty() {
        return None;
    }
    Some(rebuild(&polys, model))
}

#[derive(Clone, Copy, Debug)]
struct Plane {
    normal: DVec3,
    w: f64,
}

impl Plane {
    fn flip(&mut self) {
        self.normal = -self.normal;
        self.w = -self.w;
    }
}

#[derive(Clone, Debug)]
struct Poly {
    points: Vec<DVec3>,
    plane: Plane,
    material: Option<MaterialId>,
}

impl Poly {
    fn new(points: Vec<DVec3>, material: Option<MaterialId>) -> Option<Poly> {
        let n = newell_normal(&points);
        if n.length() < 1e-12 {
            return None;
        }
        let normal = n.normalize();
        Some(Poly { plane: Plane { normal, w: normal.dot(points[0]) }, points, material })
    }

    fn flip(&mut self) {
        self.points.reverse();
        self.plane.flip();
    }
}

/// The model's faces as convex pieces (triangles), each on its face's plane.
fn polygons(model: &Model) -> Vec<Poly> {
    let mut out = Vec::new();
    for (_, face) in model.faces() {
        let normal = face.normal.normalize();
        let w = normal.dot(model.vertex(face.outer[0]));
        for [a, b, c] in model.triangulate(face) {
            let points = if (b - a).cross(c - a).dot(normal) >= 0.0 { vec![a, b, c] } else { vec![a, c, b] };
            out.push(Poly { points, plane: Plane { normal, w }, material: face.material });
        }
    }
    out
}

const COPLANAR: u8 = 0;
const FRONT: u8 = 1;
const BACK: u8 = 2;
const SPANNING: u8 = 3;

/// Sort `poly` by `plane`: into the coplanar lists (by which way it faces) or the front and back
/// lists, split in two if it crosses.
fn split(plane: Plane, poly: Poly, coplanar_front: &mut Vec<Poly>, coplanar_back: &mut Vec<Poly>, front: &mut Vec<Poly>, back: &mut Vec<Poly>) {
    let kinds: Vec<u8> = poly
        .points
        .iter()
        .map(|p| {
            let t = plane.normal.dot(*p) - plane.w;
            if t < -PLANE {
                BACK
            } else if t > PLANE {
                FRONT
            } else {
                COPLANAR
            }
        })
        .collect();
    match kinds.iter().fold(COPLANAR, |all, k| all | k) {
        COPLANAR => {
            if plane.normal.dot(poly.plane.normal) > 0.0 {
                coplanar_front.push(poly);
            } else {
                coplanar_back.push(poly);
            }
        }
        FRONT => front.push(poly),
        BACK => back.push(poly),
        _ => {
            let (mut f, mut b) = (Vec::new(), Vec::new());
            let n = poly.points.len();
            for i in 0..n {
                let j = (i + 1) % n;
                let (ki, kj) = (kinds[i], kinds[j]);
                let (vi, vj) = (poly.points[i], poly.points[j]);
                if ki != BACK {
                    f.push(vi);
                }
                if ki != FRONT {
                    b.push(vi);
                }
                if ki | kj == SPANNING {
                    let t = (plane.w - plane.normal.dot(vi)) / plane.normal.dot(vj - vi);
                    let v = vi.lerp(vj, t);
                    f.push(v);
                    b.push(v);
                }
            }
            if f.len() >= 3 {
                front.push(Poly { points: f, ..poly.clone() });
            }
            if b.len() >= 3 {
                back.push(Poly { points: b, ..poly });
            }
        }
    }
}

#[derive(Default)]
struct Node {
    plane: Option<Plane>,
    front: Option<usize>,
    back: Option<usize>,
    polys: Vec<Poly>,
}

/// A BSP tree over a solid's faces, its nodes in a list (no recursion, so big parts don't run
/// out of stack).
struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    fn new(polys: Vec<Poly>) -> Tree {
        let mut tree = Tree { nodes: vec![Node::default()] };
        tree.build(polys);
        tree
    }

    /// Inside out: what was solid is empty and the other way round.
    fn invert(&mut self) {
        for node in &mut self.nodes {
            for p in &mut node.polys {
                p.flip();
            }
            if let Some(plane) = &mut node.plane {
                plane.flip();
            }
            std::mem::swap(&mut node.front, &mut node.back);
        }
    }

    /// The parts of `polys` outside this solid.
    fn clip(&self, polys: Vec<Poly>) -> Vec<Poly> {
        let mut out = Vec::new();
        let mut work = vec![(0, polys)];
        while let Some((i, polys)) = work.pop() {
            let node = &self.nodes[i];
            let Some(plane) = node.plane else {
                out.extend(polys);
                continue;
            };
            let (mut front, mut back) = (Vec::new(), Vec::new());
            for p in polys {
                let (mut cf, mut cb) = (Vec::new(), Vec::new());
                split(plane, p, &mut cf, &mut cb, &mut front, &mut back);
                front.extend(cf);
                back.extend(cb);
            }
            match node.front {
                Some(j) => work.push((j, front)),
                None => out.extend(front),
            }
            // Behind a leaf is inside: dropped.
            if let Some(j) = node.back {
                work.push((j, back));
            }
        }
        out
    }

    /// Clip every face of this tree by `other`.
    fn clip_to(&mut self, other: &Tree) {
        for i in 0..self.nodes.len() {
            let polys = std::mem::take(&mut self.nodes[i].polys);
            self.nodes[i].polys = other.clip(polys);
        }
    }

    fn all(&self) -> Vec<Poly> {
        self.nodes.iter().flat_map(|n| n.polys.iter().cloned()).collect()
    }

    fn build(&mut self, polys: Vec<Poly>) {
        let mut work = vec![(0, polys)];
        while let Some((i, polys)) = work.pop() {
            if polys.is_empty() {
                continue;
            }
            let plane = *self.nodes[i].plane.get_or_insert(polys[0].plane);
            let (mut front, mut back, mut same, mut opposite) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for p in polys {
                split(plane, p, &mut same, &mut opposite, &mut front, &mut back);
            }
            self.nodes[i].polys.extend(same);
            self.nodes[i].polys.extend(opposite);
            for (side, list) in [(true, front), (false, back)] {
                if list.is_empty() {
                    continue;
                }
                let child = if side { self.nodes[i].front } else { self.nodes[i].back };
                let j = child.unwrap_or_else(|| {
                    self.nodes.push(Node::default());
                    let j = self.nodes.len() - 1;
                    if side {
                        self.nodes[i].front = Some(j);
                    } else {
                        self.nodes[i].back = Some(j);
                    }
                    j
                });
                work.push((j, list));
            }
        }
    }
}

/// `a` with `b` taken out of it.
fn subtract(a: Vec<Poly>, b: Vec<Poly>) -> Vec<Poly> {
    if a.is_empty() || b.is_empty() {
        return a;
    }
    let (mut a, mut b) = (Tree::new(a), Tree::new(b));
    a.invert();
    a.clip_to(&b);
    b.clip_to(&a);
    b.invert();
    b.clip_to(&a);
    b.invert();
    a.build(b.all());
    a.invert();
    a.all()
}

/// Points welded within [`WELD`] of each other, found through a grid.
#[derive(Default)]
struct Points {
    list: Vec<DVec3>,
    grid: BTreeMap<(i64, i64, i64), Vec<usize>>,
}

impl Points {
    const CELL: f64 = 1e-4;

    fn cell(p: DVec3) -> (i64, i64, i64) {
        let c = (p / Self::CELL).floor();
        (c.x as i64, c.y as i64, c.z as i64)
    }

    fn id(&mut self, p: DVec3) -> usize {
        let (x, y, z) = Self::cell(p);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(ids) = self.grid.get(&(x + dx, y + dy, z + dz))
                        && let Some(&i) = ids.iter().find(|&&i| self.list[i].distance_squared(p) <= WELD * WELD)
                    {
                        return i;
                    }
                }
            }
        }
        self.list.push(p);
        let i = self.list.len() - 1;
        self.grid.entry((x, y, z)).or_default().push(i);
        i
    }
}

/// One plane's faces of one paint.
struct Group {
    plane: Plane,
    material: Option<MaterialId>,
    loops: Vec<Vec<usize>>,
}

/// The points of `among` strictly inside the segment `a`-`b`, in order from `a`.
fn on_segment(points: &[DVec3], a: usize, b: usize, among: &BTreeSet<usize>) -> Vec<usize> {
    let (pa, pb) = (points[a], points[b]);
    let ab = pb - pa;
    let len2 = ab.length_squared();
    let mut found: Vec<(f64, usize)> = among
        .iter()
        .filter(|&&i| i != a && i != b)
        .filter_map(|&i| {
            let t = (points[i] - pa).dot(ab) / len2;
            (t > 0.0 && t < 1.0 && (pa + ab * t).distance_squared(points[i]) <= WELD * WELD).then_some((t, i))
        })
        .collect();
    found.sort_by(|x, y| x.0.total_cmp(&y.0));
    found.into_iter().map(|(_, i)| i).collect()
}

/// `l` with every point of `among` that lies on one of its edges put into it.
fn with_points_on_edges(points: &[DVec3], l: &[usize], among: &BTreeSet<usize>) -> Vec<usize> {
    let mut out = Vec::with_capacity(l.len());
    for k in 0..l.len() {
        let (a, b) = (l[k], l[(k + 1) % l.len()]);
        out.push(a);
        out.extend(on_segment(points, a, b, among));
    }
    out
}

/// Faces from the pieces: per plane and paint, the pieces' shared edges cancel, what's left is
/// chained into outlines and holes, straight-through corners go, and every edge is split where
/// another face's corner sits on it (so neighbors share edges). Edges that were hidden in
/// `original` stay hidden.
fn rebuild(polys: &[Poly], original: &Model) -> Model {
    let mut points = Points::default();
    let mut groups: Vec<Group> = Vec::new();
    for p in polys {
        let mut l: Vec<usize> = p.points.iter().map(|&q| points.id(q)).collect();
        l.dedup();
        while l.len() > 1 && l.first() == l.last() {
            l.pop();
        }
        if l.len() < 3 {
            continue;
        }
        let at = groups.iter().position(|g| {
            g.material == p.material && g.plane.normal.dot(p.plane.normal) > 1.0 - 1e-9 && (g.plane.w - p.plane.w).abs() <= WELD
        });
        match at {
            Some(i) => groups[i].loops.push(l),
            None => groups.push(Group { plane: p.plane, material: p.material, loops: vec![l] }),
        }
    }
    let points = points.list;

    // Each group's outline: edges that pieces share both ways cancel.
    let mut faces: Vec<(Plane, Option<MaterialId>, Vec<Vec<usize>>)> = Vec::new();
    for g in &groups {
        let used: BTreeSet<usize> = g.loops.iter().flatten().copied().collect();
        let mut edges: BTreeMap<(usize, usize), i32> = BTreeMap::new();
        for l in &g.loops {
            let l = with_points_on_edges(&points, l, &used);
            for k in 0..l.len() {
                let (a, b) = (l[k], l[(k + 1) % l.len()]);
                if a == b {
                    continue;
                }
                match edges.get_mut(&(b, a)) {
                    Some(n) if *n > 0 => *n -= 1,
                    _ => *edges.entry((a, b)).or_default() += 1,
                }
            }
        }
        let mut out: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (&(a, b), &n) in &edges {
            for _ in 0..n.max(0) {
                out.entry(a).or_default().push(b);
            }
        }
        let normal = g.plane.normal;
        let u = normal.any_orthonormal_vector();
        let v = normal.cross(u);
        let flat = |i: usize| DVec2::new(points[i].dot(u), points[i].dot(v));
        let mut loops: Vec<Vec<usize>> = Vec::new();
        while let Some((&start, _)) = out.iter().find(|(_, to)| !to.is_empty()) {
            let mut l = vec![start];
            let mut at = start;
            let mut from: Option<usize> = None;
            while let Some(choices) = out.get_mut(&at) {
                if choices.is_empty() {
                    break;
                }
                // Where outlines touch at a corner, keep to this one: the sharpest left turn.
                let pick = match from {
                    Some(f) if choices.len() > 1 => {
                        let d = flat(at) - flat(f);
                        (0..choices.len())
                            .max_by(|&x, &y| {
                                let turn = |c: usize| {
                                    let e = flat(choices[c]) - flat(at);
                                    d.perp_dot(e).atan2(d.dot(e))
                                };
                                turn(x).total_cmp(&turn(y))
                            })
                            .unwrap_or(0)
                    }
                    _ => 0,
                };
                let next = choices.remove(pick);
                if next == start {
                    break;
                }
                from = Some(at);
                at = next;
                l.push(at);
                if l.len() > edges.len() + 1 {
                    break;
                }
            }
            if l.len() >= 3 {
                loops.push(l);
            }
        }
        // Outlines wind counter-clockwise about the normal; holes the other way.
        let area = |l: &[usize]| newell_normal(&l.iter().map(|&i| points[i]).collect::<Vec<_>>()).dot(normal);
        let (outers, holes): (Vec<Vec<usize>>, Vec<Vec<usize>>) = loops.into_iter().partition(|l| area(l) > 0.0);
        let project = projector(normal);
        let mut faces_here: Vec<Vec<Vec<usize>>> = outers.into_iter().map(|o| vec![o]).collect();
        for h in holes {
            let inside = |o: &[usize]| {
                let o2: Vec<DVec2> = o.iter().map(|&i| project(points[i])).collect();
                h.iter().all(|&i| locate(project(points[i]), &o2) != Location::Outside)
            };
            let host = (0..faces_here.len()).filter(|&k| inside(&faces_here[k][0])).min_by(|&x, &y| area(&faces_here[x][0]).total_cmp(&area(&faces_here[y][0])));
            if let Some(k) = host {
                faces_here[k].push(h);
            }
        }
        faces.extend(faces_here.into_iter().map(|ls| (g.plane, g.material, ls)));
    }

    // Corners that are straight through in every face using them aren't corners.
    let mut straight: BTreeMap<usize, bool> = BTreeMap::new();
    for (_, _, ls) in &faces {
        for l in ls {
            for k in 0..l.len() {
                let (p, c, n) = (points[l[(k + l.len() - 1) % l.len()]], points[l[k]], points[l[(k + 1) % l.len()]]);
                let (a, b) = (c - p, n - c);
                let through = a.cross(b).length() <= 1e-9 * a.length() * b.length() && a.dot(b) > 0.0;
                let s = straight.entry(l[k]).or_insert(true);
                *s = *s && through;
            }
        }
    }
    for (_, _, ls) in &mut faces {
        for l in ls.iter_mut() {
            l.retain(|i| !straight.get(i).copied().unwrap_or(false));
        }
        ls.retain(|l| l.len() >= 3);
    }
    faces.retain(|(_, _, ls)| !ls.is_empty());

    // Neighbors share edges: split each where another face's corner sits on it.
    let corners: BTreeSet<usize> = faces.iter().flat_map(|(_, _, ls)| ls.iter().flatten().copied()).collect();
    for (_, _, ls) in &mut faces {
        for l in ls.iter_mut() {
            *l = with_points_on_edges(&points, l, &corners);
        }
    }

    let hidden: Vec<[DVec3; 2]> = original.edges().filter(|(_, e)| e.hidden).map(|(_, e)| original.edge_points(e)).collect();
    let mut model = Model::default();
    let mut ids: BTreeMap<usize, VertexId> = BTreeMap::new();
    for (_, _, ls) in &faces {
        for &i in ls.iter().flatten() {
            ids.entry(i).or_insert_with(|| model.push_vertex(points[i]));
        }
    }
    let mut made: BTreeSet<(usize, usize)> = BTreeSet::new();
    for (plane, material, ls) in &faces {
        for l in ls {
            for k in 0..l.len() {
                let (a, b) = (l[k], l[(k + 1) % l.len()]);
                if made.insert((a.min(b), a.max(b))) {
                    let e = model.push_edge(ids[&a], ids[&b]);
                    let mid = (points[a] + points[b]) / 2.0;
                    let on = |[p, q]: &[DVec3; 2]| {
                        let pq = *q - *p;
                        let t = ((mid - *p).dot(pq) / pq.length_squared()).clamp(0.0, 1.0);
                        (*p + pq * t).distance_squared(mid) <= WELD * WELD
                    };
                    if hidden.iter().any(on) {
                        model.set_edge_hidden(e, true);
                    }
                }
            }
        }
        let map = |l: &Vec<usize>| l.iter().map(|i| ids[i]).collect::<Vec<VertexId>>();
        model.push_face(map(&ls[0]), ls[1..].iter().map(map).collect(), plane.normal, *material);
    }
    model
}

#[cfg(test)]
mod tests {
    use super::*;

    fn volume(m: &Model) -> f64 {
        m.faces()
            .flat_map(|(_, f)| m.triangulate(f).into_iter().map(move |t| (t, f.normal)))
            .map(|([a, b, c], n)| {
                let v = a.dot(b.cross(c)) / 6.0;
                if (b - a).cross(c - a).dot(n) >= 0.0 { v } else { -v }
            })
            .sum()
    }

    fn closed(m: &Model) -> bool {
        m.faces().count() > 0 && m.edges().all(|(id, _)| m.edge_faces(id).len() == 2)
    }

    /// A 10 x 10 x 2 box.
    fn block() -> Model {
        let mut m = Model::default();
        let base = m.add_rectangle(DVec3::ZERO, DVec3::X * 10.0, DVec3::Y * 10.0, DVec3::Z).unwrap();
        m.push_pull(base, 2.0).unwrap();
        m
    }

    /// The area of a regular polygon of `n` sides with corners `r` from its center.
    fn polygon_area(r: f64, n: u32) -> f64 {
        0.5 * f64::from(n) * r * r * (std::f64::consts::TAU / f64::from(n)).sin()
    }

    #[test]
    fn nothing_in_the_way_leaves_the_box_as_it_was() {
        let rod = Rod { from: DVec3::new(20.0, 0.0, 0.0), to: DVec3::new(20.0, 0.0, 5.0), radius: 1.0, sides: 12 };
        let m = drill(&block(), &[rod]).unwrap();
        assert!((volume(&m) - 200.0).abs() < 1e-9);
        assert_eq!((m.faces().count(), m.edges().count(), m.vertices().count()), (6, 12, 8), "pieces merged back");
        assert!(closed(&m));
    }

    #[test]
    fn a_hole_straight_through() {
        let rod = Rod { from: DVec3::new(5.0, 5.0, -1.0), to: DVec3::new(5.0, 5.0, 3.0), radius: 1.0, sides: 16 };
        let m = drill(&block(), &[rod]).unwrap();
        assert!((volume(&m) - (200.0 - 2.0 * polygon_area(1.0, 16))).abs() < 1e-9, "{}", volume(&m));
        assert_eq!(m.faces().count(), 6 + 16);
        assert!(m.faces().filter(|(_, f)| f.normal.z.abs() > 0.999).all(|(_, f)| f.holes.len() == 1));
        assert!(closed(&m));
    }

    #[test]
    fn a_blind_hole_at_an_angle() {
        // In from the top at 15° off flat, stopping inside.
        let dir = DVec3::new(15f64.to_radians().cos(), 0.0, -15f64.to_radians().sin());
        let rod = Rod { from: DVec3::new(2.0, 5.0, 2.5) - dir * 0.0, to: DVec3::new(2.0, 5.0, 2.5) + dir * 4.0, radius: 0.1875, sides: 16 };
        let m = drill(&block(), &[rod]).unwrap();
        assert!(closed(&m), "a closed solid");
        let taken = 200.0 - volume(&m);
        assert!(taken > 0.01 && taken < polygon_area(0.1875, 16) * 4.0, "{taken}");
        // The same again gives the same: nothing more to take.
        let again = drill(&m, &[rod]).unwrap();
        assert!((volume(&again) - volume(&m)).abs() < 1e-9);
        assert_eq!(serde_json::to_string(&drill(&block(), &[rod]).unwrap()).unwrap(), serde_json::to_string(&m).unwrap(), "the same every time");
    }

    #[test]
    fn paint_stays_on_its_faces() {
        let mut m = block();
        let paint = crate::Document::default().add_material(crate::MaterialKind::SolidWood);
        let top = m.faces().find(|(_, f)| f.normal.z > 0.999).map(|(id, _)| id).unwrap();
        m.set_face_material(top, Some(paint));
        let rod = Rod { from: DVec3::new(5.0, 5.0, 1.0), to: DVec3::new(5.0, 5.0, 3.0), radius: 1.0, sides: 16 };
        let m = drill(&m, &[rod]).unwrap();
        let top = m.faces().find(|(_, f)| f.normal.z > 0.999 && (m.vertex(f.outer[0]).z - 2.0).abs() < 1e-9).unwrap().1;
        assert_eq!(top.material, Some(paint));
        assert_eq!(top.holes.len(), 1);
        let floor = m.faces().find(|(_, f)| f.normal.z > 0.999 && (m.vertex(f.outer[0]).z - 1.0).abs() < 1e-9).unwrap().1;
        assert_eq!(floor.material, None, "the hole's own faces are bare");
        assert!(closed(&m));
    }
}
