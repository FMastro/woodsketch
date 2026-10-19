//! A picture of a drawing without a window: the app's renderer on an offscreen GPU device, framed
//! like Zoom Extents from a standard view (or turned to any angle), saved as PNG. For agents
//! checking their work: in the light or dark theme, with parts picked out in the selection's blue.

use glam::Vec2;
use woodsketch_doc::{Document, Entity, Selection};
use woodsketch_render::{Scene, Sky, ViewportRenderer};
use woodsketch_view::{OrbitCamera, StandardView};

/// How to draw it.
pub struct Options {
    pub view: StandardView,
    /// Turned to this angle instead: degrees around (from the front, toward the right) and up.
    pub angle: Option<(f32, f32)>,
    pub size: [u32; 2],
    pub dark: bool,
    /// Parts or groups (by name) drawn picked out, as when selected.
    pub highlight: Vec<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self { view: StandardView::Iso, angle: None, size: [1200, 800], dark: false, highlight: Vec::new() }
    }
}

/// `iso`, `top`, `front` or `right`.
pub fn view(name: &str) -> Result<StandardView, String> {
    match name.to_ascii_lowercase().as_str() {
        "iso" | "" => Ok(StandardView::Iso),
        "top" | "plan" => Ok(StandardView::Top),
        "front" => Ok(StandardView::Front),
        "right" | "side" => Ok(StandardView::Right),
        other => Err(format!("\"{other}\" isn't a view (iso, top, front or right).")),
    }
}

/// `around,up` in degrees, like `30,20`.
pub fn angle(text: &str) -> Result<(f32, f32), String> {
    let bad = || format!("An angle is \"around,up\" in degrees, like 30,20 (not \"{text}\").");
    let (around, up) = text.split_once(',').ok_or_else(bad)?;
    Ok((around.trim().parse().map_err(|_| bad())?, up.trim().parse().map_err(|_| bad())?))
}

/// The drawing drawn as `options` say, as PNG bytes.
pub fn render(doc: &Document, options: &Options) -> Result<Vec<u8>, String> {
    let [width, height] = options.size;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .map_err(|e| format!("No graphics adapter to draw with: {e}"))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .map_err(|e| format!("Couldn't open the graphics device: {e}"))?;

    let mut scene = Scene::reference(false);
    scene.add_drawing(doc.scene(), doc.path());
    // The parts to pick out, by name: top-level parts and groups.
    let mut picked = Selection::default();
    for name in &options.highlight {
        let found: Vec<Entity> = doc
            .scene()
            .root
            .instances()
            .filter(|(_, i)| doc.scene().definition(i.definition).is_some_and(|d| d.name.eq_ignore_ascii_case(name)))
            .map(|(id, _)| Entity::Instance(id))
            .collect();
        if found.is_empty() {
            return Err(format!("No part or group named \"{name}\" at the top of the drawing to highlight."));
        }
        picked.extend(found);
    }
    scene.add_selection(doc.view(), &picked, &doc.context_transform(), doc.scene());
    let mut renderer = ViewportRenderer::new(&device, &scene);
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
    }
    renderer.render(&device, &queue, [width, height], camera.view_proj(viewport), camera.eye(), 1.0);
    let (w, h, pixels) = renderer.read_pixels(&device, &queue).ok_or("Couldn't read the picture back from the GPU.")?;

    let mut png = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&pixels).map_err(|e| e.to_string())?;
    }
    Ok(png)
}
