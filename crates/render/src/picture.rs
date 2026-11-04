//! A picture of a drawing without a window: the app's renderer drawn offscreen, framed like
//! Zoom Extents from a standard view (or turned to any angle), read back as RGBA pixels.
//!
//! The device is handed in rather than made here, so this serves both callers: the command line
//! opens a device of its own for one picture, and the app draws thumbnails on the device it is
//! already running on, where opening a second one for each would be wasteful and slow.

use glam::{DAffine3, Vec2};
use woodsketch_doc::{Container, Document, Entity, Instance, MaterialId, Selection};
use woodsketch_view::{OrbitCamera, StandardView};

use crate::{Scene, Sky, ViewportRenderer};

/// How to draw it.
pub struct Options {
    pub view: StandardView,
    /// Turned to this angle instead: degrees around (from the front, toward the right) and up.
    pub angle: Option<(f32, f32)>,
    pub size: [u32; 2],
    pub dark: bool,
    /// Parts or groups (by name) drawn picked out, as when selected.
    pub highlight: Vec<String>,
    /// Draw the red/green/blue axes through the origin, as the app does. Off for a picture of
    /// the piece by itself, such as a thumbnail, where they are only clutter.
    pub axes: bool,
    /// Fit the drawing to this much of the frame (0.9 being nine tenths of the shorter side),
    /// instead of leaving it where Zoom Extents puts it. Zoom Extents fits the sphere around
    /// the drawing, which leaves a tall, thin piece small in the middle of a wide frame.
    pub fit: Option<f32>,
}

impl Default for Options {
    fn default() -> Self {
        Self { view: StandardView::Iso, angle: None, size: [1200, 800], dark: false, highlight: Vec::new(), axes: true, fit: None }
    }
}

/// Every shown part or group called `name` inside `container` (at `world`, painted `paint` from
/// around it), as an instance placed in the world.
fn named(scene: &woodsketch_doc::Scene, container: &Container, world: DAffine3, paint: Option<MaterialId>, name: &str, found: &mut Vec<Instance>) {
    for (_, instance) in container.instances().filter(|(_, i)| scene.is_visible(i)) {
        let Some(def) = scene.definition(instance.definition) else { continue };
        let placed = world * instance.transform;
        let paint = instance.material.or(paint);
        if def.name.trim().eq_ignore_ascii_case(name.trim()) {
            found.push(Instance { definition: instance.definition, transform: placed, tag: None, material: paint });
        } else {
            named(scene, &def.container, placed, paint, name, found);
        }
    }
}

/// Draw `doc` as `options` say, on `device`, and read it back as `(width, height, RGBA)`.
pub fn pixels(device: &wgpu::Device, queue: &wgpu::Queue, doc: &Document, options: &Options) -> Result<(u32, u32, Vec<u8>), String> {
    let [width, height] = options.size;
    let mut scene = if options.axes { Scene::reference(false) } else { Scene::default() };
    scene.add_drawing(doc.scene(), doc.path());
    // The parts to pick out, by name, at any depth: each lifted to the top of a copy of the
    // drawing where it is in the world, and highlighted there over the drawing.
    if !options.highlight.is_empty() {
        let mut lifted = doc.scene().clone();
        lifted.root = Default::default();
        for name in &options.highlight {
            let mut found = Vec::new();
            named(doc.scene(), &doc.scene().root, DAffine3::IDENTITY, None, name, &mut found);
            if found.is_empty() {
                return Err(format!("No part or group named \"{name}\" in the drawing to highlight."));
            }
            for instance in found {
                lifted.root.add_instance(instance);
            }
        }
        let lifted = Document::from_scene(lifted);
        let mut picked = Selection::default();
        picked.extend(lifted.scene().root.instances().map(|(id, _)| Entity::Instance(id)));
        scene.add_selection(lifted.view(), &picked, &lifted.context_transform(), lifted.scene());
    }
    let mut renderer = ViewportRenderer::new(device, &scene);
    if options.dark {
        renderer.sky = Sky::dark();
    }
    let viewport = Vec2::new(width as f32, height as f32);
    let mut camera = OrbitCamera::default();
    camera.set_view(options.view);
    if let Some((around, up)) = options.angle {
        // From the front (looking toward +y), turned toward the right, and raised.
        camera.yaw = (around - 90.0).to_radians();
        camera.pitch = up.clamp(-89.0, 89.0).to_radians();
    }
    if let Some((lo, hi)) = doc.bounds() {
        camera.zoom_extents(lo.as_vec3(), hi.as_vec3(), viewport);
        if let Some(fit) = options.fit {
            fit_to_frame(&mut camera, lo.as_vec3(), hi.as_vec3(), viewport, fit);
        }
    }
    renderer.render(device, queue, [width, height], camera.view_proj(viewport), camera.eye(), 1.0);
    renderer.read_pixels(device, queue).ok_or_else(|| "Couldn't read the picture back from the GPU.".to_owned())
}

/// Move in until the drawing's corners fill `fraction` of the frame.
///
/// [`OrbitCamera::zoom_extents`] fits the sphere around the drawing, which is right for a
/// viewport someone is about to orbit in — nothing leaves the frame whichever way they turn —
/// but wasteful for one picture from one angle: a 72" bookshelf's diagonal is far longer than
/// its outline on screen, so it ends up small with sky all round it.
///
/// How big the drawing looks is very nearly inversely proportional to the distance, so stepping
/// the distance by how far off we are converges in a couple of rounds; three is plenty, and
/// each one only projects eight points.
fn fit_to_frame(camera: &mut OrbitCamera, lo: glam::Vec3, hi: glam::Vec3, viewport: Vec2, fraction: f32) {
    let corners: Vec<glam::Vec3> = (0..8)
        .map(|i| {
            glam::Vec3::new(
                if i & 1 == 0 { lo.x } else { hi.x },
                if i & 2 == 0 { lo.y } else { hi.y },
                if i & 4 == 0 { lo.z } else { hi.z },
            )
        })
        .collect();
    let target = fraction.clamp(0.1, 1.0);
    for _ in 0..3 {
        let view_proj = camera.view_proj(viewport);
        let mut reach: f32 = 0.0;
        for corner in &corners {
            let ndc = view_proj.project_point3(*corner);
            // Behind the camera or outside the near plane: leave the framing alone rather than
            // chase a projection that has flipped.
            if !ndc.x.is_finite() || !ndc.y.is_finite() {
                return;
            }
            reach = reach.max(ndc.x.abs()).max(ndc.y.abs());
        }
        if reach <= f32::EPSILON {
            return;
        }
        camera.distance = (camera.distance * reach / target).max(1.0e-3);
    }
}
