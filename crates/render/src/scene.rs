//! CPU-side geometry the renderer draws: the reference grid and axes, plus the model.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use glam::{DAffine3, DMat4, DVec3, Mat4, Vec3};
use woodsketch_doc::{
    Container, DefinitionId, Document, EdgeId, Entity, Face, Guide, InstanceId, MaterialId, Model, Owner, Scene as SceneData, Selection, VertexId,
    View,
};

/// Linear-space RGBA. The render target is sRGB, so colors are converted from sRGB on creation.
pub type Color = [f32; 4];

pub fn srgb(r: u8, g: u8, b: u8) -> Color {
    let lin = |c: u8| (f32::from(c) / 255.0).powf(2.2);
    [lin(r), lin(g), lin(b), 1.0]
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

/// Position + color, 7 floats.
pub type LineVertex = [f32; 7];
/// Position + normal + front color + back color, 14 floats.
pub type FaceVertex = [f32; 14];

#[derive(Default)]
pub struct Scene {
    /// Drawn first without writing depth, so faces always cover them (the ground grid).
    pub background_lines: Vec<LineVertex>,
    /// Pairs of vertices, one pair per segment, in world coordinates (the axes).
    pub lines: Vec<LineVertex>,
    /// The drawing's meshes, each in its own coordinates: triangles and edge segments, drawn at
    /// [`Self::placements`] as [`Self::batches`] say.
    pub mesh_faces: Vec<FaceVertex>,
    pub mesh_lines: Vec<LineVertex>,
    pub placements: Vec<Placement>,
    pub batches: Vec<Batch>,
    /// Guides, in world coordinates; turned into dashed lines each frame ([`guide_vertices`]).
    pub guides: Vec<SceneGuide>,
    /// The selection, drawn over the rest: tinted faces, blue edges and guides.
    pub selected_faces: Vec<FaceVertex>,
    pub selected_lines: Vec<LineVertex>,
    pub selected_guides: Vec<SceneGuide>,
}

/// Where a mesh is drawn: its placement (a column-major 4x4 matrix) and how faded it is (0, or
/// 1 outside the group being edited), padded to 20 floats.
pub type Placement = [f32; 20];

/// One mesh (ranges of [`Scene::mesh_faces`] and [`Scene::mesh_lines`]) drawn at each of a range
/// of [`Scene::placements`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub faces: Range<u32>,
    pub lines: Range<u32>,
    pub placements: Range<u32>,
}

/// A mesh: a container's geometry (`None` for the top level) with the paint it shows.
type MeshKey = (Option<DefinitionId>, Option<MaterialId>);

/// The meshes needed (a container's geometry with the paint it shows) and where each is drawn.
#[derive(Default)]
struct Uses {
    list: Vec<(MeshKey, Vec<Placement>)>,
    index: HashMap<MeshKey, usize>,
}

impl Uses {
    fn add(&mut self, key: MeshKey, placement: Placement) {
        let list = &mut self.list;
        let i = *self.index.entry(key).or_insert_with(|| {
            list.push((key, Vec::new()));
            list.len() - 1
        });
        self.list[i].1.push(placement);
    }
}

fn placement(world: &DAffine3, faded: bool) -> Placement {
    let mut out = [0.0; 20];
    out[..16].copy_from_slice(&DMat4::from(*world).as_mat4().to_cols_array());
    out[16] = if faded { 1.0 } else { 0.0 };
    out
}

fn line_vertex(p: Vec3, color: Color) -> LineVertex {
    [p.x, p.y, p.z, color[0], color[1], color[2], color[3]]
}

impl Scene {
    /// SketchUp-colored axes, and optionally a ground grid (12" spacing).
    pub fn reference(grid: bool) -> Self {
        let mut scene = Self::default();

        let grid_color = srgb(222, 222, 222);
        let (spacing, half_count) = (12.0, if grid { 20 } else { 0 });
        let extent = spacing * half_count as f32;
        for i in -half_count..=half_count {
            if i == 0 {
                continue; // The axes are drawn there.
            }
            let o = i as f32 * spacing;
            for (a, b) in [
                (Vec3::new(o, -extent, 0.0), Vec3::new(o, extent, 0.0)),
                (Vec3::new(-extent, o, 0.0), Vec3::new(extent, o, 0.0)),
            ] {
                scene.background_lines.extend([line_vertex(a, grid_color), line_vertex(b, grid_color)]);
            }
        }

        // Solid on the positive side, faded on the negative side, as in SketchUp.
        let axis_len = 1.0e4;
        for (dir, color) in [
            (Vec3::X, srgb(220, 0, 0)),
            (Vec3::Y, srgb(0, 160, 0)),
            (Vec3::Z, srgb(0, 0, 220)),
        ] {
            let faded = mix(color, srgb(255, 255, 255), 0.6);
            scene.add_line(Vec3::ZERO, dir * axis_len, color);
            scene.add_line(Vec3::ZERO, -dir * axis_len, faded);
        }
        scene
    }

    pub fn add_line(&mut self, a: Vec3, b: Vec3, color: Color) {
        self.lines.extend([line_vertex(a, color), line_vertex(b, color)]);
    }

    /// Add the drawing as seen while editing `doc`: the geometry ([`Self::add_drawing`]) and
    /// the selection's highlight ([`Self::add_selection`]).
    pub fn add_view(&mut self, doc: &Document) {
        self.add_drawing(doc.scene(), doc.path());
        self.add_selection(doc.view(), doc.selection(), &doc.context_transform(), doc.scene());
    }

    /// Everything in the drawing, unselected, rebuilt only when the drawing (or the group
    /// being edited) changes. Each container's geometry is meshed once (per paint it shows) in its
    /// own coordinates and drawn at every place it's used, so a component used 800 times costs
    /// one mesh and 800 placements. While editing inside a group (`path`), everything outside it
    /// is faded, as in SketchUp.
    pub fn add_drawing(&mut self, scene: &SceneData, path: &[InstanceId]) {
        let mut uses = Uses::default();
        self.walk(scene, None, &scene.root, DAffine3::IDENTITY, None, !path.is_empty(), Some(path), &mut uses);
        for ((definition, paint), placements) in uses.list {
            let model = match definition {
                Some(d) => match scene.definition(d) {
                    Some(d) => &d.container.model,
                    None => continue,
                },
                None => &scene.root.model,
            };
            let (faces, lines, placed) = (self.mesh_faces.len() as u32, self.mesh_lines.len() as u32, self.placements.len() as u32);
            self.push_mesh(model, paint, scene);
            self.placements.extend(placements);
            self.batches.push(Batch {
                faces: faces..self.mesh_faces.len() as u32,
                lines: lines..self.mesh_lines.len() as u32,
                placements: placed..self.placements.len() as u32,
            });
        }
    }

    /// Note where `container` (of `definition`, `None` for the top level) is drawn, then its
    /// instances. `rest` is the part of the editing path still to go into (`None` once off it).
    #[allow(clippy::too_many_arguments)]
    fn walk(
        &mut self,
        scene: &SceneData,
        definition: Option<DefinitionId>,
        container: &Container,
        world: DAffine3,
        paint: Option<MaterialId>,
        faded: bool,
        rest: Option<&[InstanceId]>,
        uses: &mut Uses,
    ) {
        let model = &container.model;
        if model.faces().next().is_some() || model.edges().next().is_some() {
            uses.add((definition, paint), placement(&world, faded));
        }
        // Dimensions and text are drawn over the view by the UI (with their lengths and words).
        for (_, guide) in model.guides().filter(|(_, g)| !scene.guides_hidden && !g.is_note()) {
            let color = if faded { srgb(170, 170, 170) } else { srgb(70, 70, 70) };
            self.guides.push(SceneGuide { guide: guide.transformed(&world), color });
        }
        for (id, instance) in container.instances() {
            if !scene.is_visible(instance) {
                continue;
            }
            let Some(inner) = scene.definition(instance.definition) else { continue };
            // Into the group being edited (or one on the way to it), or just past it.
            let (rest, faded) = match rest {
                Some([first, deeper @ ..]) if *first == id => (Some(deeper), !deeper.is_empty()),
                _ => (None, faded),
            };
            self.walk(scene, Some(instance.definition), &inner.container, world * instance.transform, instance.material.or(paint), faded, rest, uses);
        }
    }

    /// `model`'s faces and edges in its own coordinates; faces without paint of their own show
    /// `paint`. Edges between a curve's facets are left out (soft).
    fn push_mesh(&mut self, model: &Model, paint: Option<MaterialId>, scene: &SceneData) {
        for (_, face) in model.faces() {
            let (front, back) = colors(face.material.or(paint), scene);
            let n = face.normal.as_vec3();
            for triangle in model.triangulate(face) {
                for p in triangle.map(|p| p.as_vec3()) {
                    self.mesh_faces.push([
                        p.x, p.y, p.z, n.x, n.y, n.z, front[0], front[1], front[2], front[3], back[0], back[1], back[2], back[3],
                    ]);
                }
            }
        }
        let soft = soft_edges(model, model.faces().map(|(_, f)| f));
        let black = srgb(0, 0, 0);
        for (id, e) in model.edges() {
            if !soft.contains(&id) && !e.hidden {
                let [a, b] = model.edge_points(e);
                self.mesh_lines.extend([line_vertex(a.as_vec3(), black), line_vertex(b.as_vec3(), black)]);
            }
        }
    }

    /// The selection's highlight, drawn over the geometry: selected faces (all of a selected
    /// group's) tinted blue, their edges and selected guides in blue. Cheap to rebuild when only
    /// the selection changes.
    pub fn add_selection(&mut self, view: &View, selection: &Selection, to_world: &DAffine3, scene: &SceneData) {
        if selection.is_empty() {
            return;
        }
        let model = view.model();
        let world = |p: DVec3| to_world.transform_point3(p).as_vec3();
        let is_selected = |entity: Entity| match view.owner(entity) {
            Owner::Active => selection.contains(entity),
            Owner::Instance(id) => selection.contains(Entity::Instance(id)),
            Owner::Outside => false,
        };
        let mut selected_faces = Vec::new();
        for (id, face) in model.faces() {
            if is_selected(Entity::Face(id)) {
                let (front, back) = colors(view.face_material(id), scene);
                self.push_face(model, face, to_world, tint(front), tint(back));
                selected_faces.push(face);
            }
        }
        // A selected group's soft edges (between a curve's facets) stay hidden.
        let soft = soft_edges(model, selected_faces.into_iter());
        let blue = srgb(0, 64, 255);
        for (id, e) in model.edges() {
            let in_selected_group = matches!(view.owner(Entity::Edge(id)), Owner::Instance(_));
            if is_selected(Entity::Edge(id)) && !(in_selected_group && (soft.contains(&id) || e.hidden)) {
                let [a, b] = model.edge_points(e);
                self.selected_lines.extend([line_vertex(world(a), blue), line_vertex(world(b), blue)]);
            }
        }
        for (id, guide) in model.guides().filter(|(_, g)| !g.is_note()) {
            if is_selected(Entity::Guide(id)) {
                self.selected_guides.push(SceneGuide { guide: guide.transformed(to_world), color: blue });
            }
        }
    }

    fn push_face(&mut self, model: &Model, face: &Face, to_world: &DAffine3, front: Color, back: Color) {
        let n = to_world.transform_vector3(face.normal).normalize().as_vec3();
        let target = &mut self.selected_faces;
        for triangle in model.triangulate(face) {
            for p in triangle.map(|p| to_world.transform_point3(p).as_vec3()) {
                target.push([
                    p.x, p.y, p.z, n.x, n.y, n.z, front[0], front[1], front[2], front[3], back[0], back[1], back[2], back[3],
                ]);
            }
        }
    }
}


/// A guide to draw, in world coordinates.
#[derive(Clone, Debug)]
pub struct SceneGuide {
    pub guide: Guide,
    pub color: Color,
}

/// Position + color + the segment's start (for measuring dashes) + 1 for dashed, 0 for solid.
pub type DashedVertex = [f32; 11];

/// How far guide lines reach either side of the point nearest the eye, in inches: far beyond
/// any furniture, yet small enough for f32 precision.
const GUIDE_REACH: f64 = 20_000.0;

/// The guides as line segments for this camera: guide lines cut where they pass behind the
/// eye (so the GPU can measure dashes from a visible start), guide points as small crosses
/// facing the eye.
pub fn guide_vertices(guides: &[SceneGuide], view_proj: Mat4, eye: Vec3) -> Vec<DashedVertex> {
    let eye = eye.as_dvec3();
    let view_proj = view_proj.as_dmat4();
    // Depth in front of the eye (the projection's w).
    let w = |p: DVec3| (view_proj * p.extend(1.0)).w;
    let min_w = 1e-3;
    let vertex = |p: DVec3, color: Color, start: DVec3, dash: f32| -> DashedVertex {
        let (p, s) = (p.as_vec3(), start.as_vec3());
        [p.x, p.y, p.z, color[0], color[1], color[2], color[3], s.x, s.y, s.z, dash]
    };
    let mut out = Vec::new();
    for g in guides {
        match g.guide {
            Guide::Line { .. } | Guide::Segment { .. } => {
                let (mut a, mut b) = match g.guide {
                    Guide::Line { point, dir } => {
                        let middle = point + dir * (eye - point).dot(dir);
                        (middle - dir * GUIDE_REACH, middle + dir * GUIDE_REACH)
                    }
                    Guide::Segment { a, b } => (a, b),
                    _ => unreachable!("matched above"),
                };
                let (wa, wb) = (w(a), w(b));
                if wa < min_w && wb < min_w {
                    continue;
                }
                if wa < min_w {
                    a += (b - a) * ((min_w - wa) / (wb - wa));
                } else if wb < min_w {
                    b = a + (b - a) * ((wa - min_w) / (wa - wb));
                }
                // Dashes are measured from a point that projects near the view, not from a far
                // end that lands way off screen (where f32 pixel distances lose precision): the
                // point nearest the line of sight through the middle of the view.
                let ahead = view_proj.inverse().project_point3(DVec3::new(0.0, 0.0, 1.0)) - eye;
                let start = closest_to_ray(a, b, eye, ahead.normalize_or_zero());
                out.extend([vertex(a, g.color, start, 1.0), vertex(b, g.color, start, 1.0)]);
            }
            Guide::Point(p) => {
                let to = (p - eye).normalize_or_zero();
                let right = to.cross(DVec3::Z).try_normalize().unwrap_or(DVec3::X);
                let up = right.cross(to);
                let size = p.distance(eye) * 0.008;
                for d in [right + up, right - up] {
                    let d = d * size;
                    out.extend([vertex(p - d, g.color, p - d, 0.0), vertex(p + d, g.color, p - d, 0.0)]);
                }
            }
            // Drawn by the UI.
            Guide::Dimension { .. } | Guide::Text { .. } => {}
        }
    }
    out
}


/// The point of segment `a`–`b` nearest the ray from `origin` along unit `dir`.
fn closest_to_ray(a: DVec3, b: DVec3, origin: DVec3, dir: DVec3) -> DVec3 {
    let along = b - a;
    let length = along.length();
    if length < 1e-9 {
        return a;
    }
    let d = along / length;
    let w = a - origin;
    let k = d.dot(dir);
    let denom = 1.0 - k * k;
    let t = if denom < 1e-12 { 0.0 } else { (k * dir.dot(w) - d.dot(w)) / denom };
    a + d * t.clamp(0.0, length)
}


/// A face's front and back colors: SketchUp's default material (warm white front, blue-grey
/// back), or its paint (its own, else its group's) on both sides.
/// A face's front and back colors for the material it shows (`None`: the default material).
fn colors(material: Option<MaterialId>, scene: &SceneData) -> (Color, Color) {
    match material.and_then(|m| scene.material(m)) {
        Some(m) => {
            let c = srgb(m.color[0], m.color[1], m.color[2]);
            (c, c)
        }
        None => (srgb(250, 250, 246), srgb(164, 180, 204)),
    }
}

/// Selected faces: tinted blue.
fn tint(c: Color) -> Color {
    mix(c, srgb(80, 120, 255), 0.45)
}

/// The edges of `model` between two of `faces` that meet at a shallow angle (a curve's facets).
fn soft_edges<'a>(model: &Model, faces: impl Iterator<Item = &'a Face>) -> HashSet<EdgeId> {
    let mut sides: HashMap<(VertexId, VertexId), Vec<DVec3>> = HashMap::new();
    for face in faces {
        for l in face.loops() {
            for i in 0..l.len() {
                let (a, b) = (l[i], l[(i + 1) % l.len()]);
                sides.entry((a.min(b), a.max(b))).or_default().push(face.normal);
            }
        }
    }
    model
        .edges()
        .filter(|(_, e)| sides.get(&(e.a.min(e.b), e.a.max(e.b))).is_some_and(|n| matches!(n.as_slice(), [n, m] if is_soft(*n, *m))))
        .map(|(id, _)| id)
        .collect()
}

/// Whether the edge between faces with normals `n` and `m` is soft, as SketchUp draws the edges
/// between a circle's or arc's facets: hidden, because the faces meet at a shallow angle (under
/// 30°). Edges between faces on one plane stay visible: they're real seams.
fn is_soft(n: DVec3, m: DVec3) -> bool {
    let cos = n.dot(m);
    cos > 30f64.to_radians().cos() && cos < 1.0 - 1e-6
}

#[cfg(test)]
mod tests {
    use woodsketch_doc::Document;

    use super::*;

    fn two_boards() -> (Document, woodsketch_doc::InstanceId) {
        let mut doc = Document::default();
        let face = doc.edit(|m| m.add_rectangle(DVec3::ZERO, DVec3::X * 1.5, DVec3::Y * 3.5, DVec3::Z)).unwrap();
        let group = doc.push_pull(face, 10.0, true).unwrap().unwrap();
        let loose = doc.edit(|m| m.add_rectangle(DVec3::X * 20.0, DVec3::X * 1.5, DVec3::Y * 3.5, DVec3::Z)).unwrap();
        doc.push_pull(loose, 10.0, false).unwrap();
        (doc, group)
    }

    #[test]
    fn edges_between_a_holes_facets_are_hidden() {
        let mut doc = Document::default();
        doc.edit(|m| {
            let base = m.add_rectangle(DVec3::ZERO, DVec3::X * 6.0, DVec3::Y * 4.0, DVec3::Z).unwrap();
            m.push_pull(base, 0.75).unwrap();
            let disk = m.draw_circle(DVec3::new(3.0, 2.0, 0.75), DVec3::Z, 0.5, 24).unwrap();
            m.push_pull(disk, -0.75).unwrap();
        });
        let mut scene = Scene::default();
        scene.add_view(&doc);
        // The board's 12 edges and the hole's two rims; not the 24 seams down its wall.
        assert_eq!(scene.mesh_lines.len(), (12 + 24 + 24) * 2);
    }

    #[test]
    fn guide_lines_start_in_front_of_the_eye() {
        let guides = [SceneGuide { guide: Guide::Line { point: DVec3::ZERO, dir: DVec3::X }, color: srgb(0, 0, 0) }];
        let mut camera = woodsketch_view::OrbitCamera { target: glam::Vec3::ZERO, distance: 50.0, ..Default::default() };
        camera.set_view(woodsketch_view::StandardView::Iso);
        let size = glam::Vec2::new(1000.0, 800.0);
        let vp = camera.view_proj(size);
        let vertices = guide_vertices(&guides, vp, camera.eye());
        assert_eq!(vertices.len(), 2);
        for v in &vertices {
            let w = (vp * glam::Vec4::new(v[0], v[1], v[2], 1.0)).w;
            assert!(w > 0.0, "every end is in front of the camera");
        }
    }

    #[test]
    fn a_closed_groups_guides_are_drawn_and_faded_outside_the_open_group() {
        let (mut doc, group) = two_boards();
        doc.enter(group);
        doc.edit(|m| m.add_guide(Guide::Line { point: DVec3::ZERO, dir: DVec3::Z }));
        doc.exit();
        doc.selection_mut().clear();
        let mut scene = Scene::default();
        scene.add_view(&doc);
        assert_eq!(scene.guides.len(), 1, "a closed group's guide still shows");
        assert_eq!(scene.guides[0].color, srgb(70, 70, 70));
        // Editing the other (loose) geometry's group puts this one outside: faded.
        let loose_group = doc.edit(|m| m.add_rectangle(DVec3::new(40.0, 0.0, 0.0), DVec3::X, DVec3::Y, DVec3::Z)).unwrap();
        let other = doc.push_pull(loose_group, 1.0, true).unwrap().unwrap();
        doc.enter(other);
        let mut scene = Scene::default();
        scene.add_view(&doc);
        assert_eq!(scene.guides[0].color, srgb(170, 170, 170));
    }

    #[test]
    fn groups_and_loose_geometry_both_draw() {
        let (doc, _) = two_boards();
        let mut scene = Scene::default();
        scene.add_view(&doc);
        assert_eq!(scene.mesh_faces.len(), 2 * 6 * 6); // two boards x 6 faces x 2 triangles x 3
        assert_eq!(scene.mesh_lines.len(), 2 * 12 * 2);
        assert_eq!(scene.batches.len(), 2, "the loose board and the group's, one placement each");
    }

    #[test]
    fn a_selected_group_is_tinted_as_a_whole() {
        let (mut doc, group) = two_boards();
        let plain = |doc: &Document| {
            let mut scene = Scene::default();
            scene.add_view(doc);
            scene
        };
        let before = plain(&doc);
        assert!(before.selected_faces.is_empty() && before.selected_lines.is_empty());
        doc.selection_mut().extend([Entity::Instance(group)]);
        let after = plain(&doc);
        assert_eq!(after.mesh_faces, before.mesh_faces, "the drawing itself doesn't change with the selection");
        assert_eq!(after.selected_faces.len(), 36, "exactly the group's 6 faces (36 vertices) are tinted");
        assert_eq!(after.selected_lines.len(), 24, "and its 12 edges drawn blue");
        // The highlight alone, as the viewport rebuilds it when only the selection changes.
        let mut highlight = Scene::default();
        highlight.add_selection(doc.view(), doc.selection(), &doc.context_transform(), doc.scene());
        assert_eq!(highlight.selected_faces, after.selected_faces);
    }

    /// Every triangle corner where it lands in the world.
    fn world_faces(scene: &Scene) -> Vec<[f32; 3]> {
        let mut out = Vec::new();
        for b in &scene.batches {
            for p in &scene.placements[b.placements.start as usize..b.placements.end as usize] {
                let m = Mat4::from_cols_array(p[..16].try_into().unwrap());
                for v in &scene.mesh_faces[b.faces.start as usize..b.faces.end as usize] {
                    out.push(m.transform_point3(Vec3::new(v[0], v[1], v[2])).to_array());
                }
            }
        }
        out
    }

    #[test]
    fn editing_inside_fades_the_outside_and_keeps_world_positions() {
        let (mut doc, group) = two_boards();
        let view = |doc: &Document| {
            let mut scene = Scene::default();
            scene.add_view(doc);
            scene
        };
        let key = |p: &[f32; 3]| (p[0].to_bits(), p[1].to_bits(), p[2].to_bits());
        let mut before = world_faces(&view(&doc));
        assert!(view(&doc).placements.iter().all(|p| p[16] == 0.0), "nothing faded at the top level");
        doc.enter(group);
        let scene = view(&doc);
        let mut after = world_faces(&scene);
        before.sort_by_key(key);
        after.sort_by_key(key);
        assert_eq!(before, after, "nothing moves on screen when opening a group");
        let faded: Vec<f32> = scene.placements.iter().map(|p| p[16]).collect();
        assert_eq!(faded.iter().filter(|f| **f == 1.0).count(), 1, "the loose board outside is faded: {faded:?}");
        assert_eq!(faded.iter().filter(|f| **f == 0.0).count(), 1, "the open group isn't");
    }

    #[test]
    fn a_component_used_many_times_is_meshed_once() {
        let (mut doc, group) = two_boards();
        let copies = [1.0, 2.0, 3.0].map(|y| glam::DAffine3::from_translation(DVec3::Y * 10.0 * y));
        doc.transform_entities(&[Entity::Instance(group)], &copies, true).unwrap();
        let mut scene = Scene::default();
        scene.add_view(&doc);
        assert_eq!(scene.mesh_faces.len(), 2 * 36, "still one mesh for the group (plus the loose board)");
        assert_eq!(scene.placements.len(), 1 + 4, "the loose board, and the group four times");
        assert_eq!(world_faces(&scene).len(), 5 * 36);
    }
}
