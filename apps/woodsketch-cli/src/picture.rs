//! A picture of a drawing without a window: the app's renderer on an offscreen GPU device, framed
//! like Zoom Extents from a standard view, saved as PNG. For agents checking their work.

use glam::Vec2;
use woodsketch_doc::Document;
use woodsketch_render::{Scene, ViewportRenderer};
use woodsketch_view::{OrbitCamera, StandardView};

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

/// The drawing seen from `view`, `width` x `height` pixels, as PNG bytes.
pub fn render(doc: &Document, view: StandardView, [width, height]: [u32; 2]) -> Result<Vec<u8>, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .map_err(|e| format!("No graphics adapter to draw with: {e}"))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .map_err(|e| format!("Couldn't open the graphics device: {e}"))?;

    let mut scene = Scene::reference(false);
    scene.add_view(doc);
    let mut renderer = ViewportRenderer::new(&device, &scene);
    let viewport = Vec2::new(width as f32, height as f32);
    let mut camera = OrbitCamera::default();
    camera.set_view(view);
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
