//! A picture of a drawing from the command line: a GPU device of our own, the shared offscreen
//! renderer ([`woodsketch_render::picture`]), and PNG out. For agents checking their work.

use woodsketch_doc::Document;
use woodsketch_view::StandardView;

pub use woodsketch_render::picture::Options;

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
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .map_err(|e| format!("No graphics adapter to draw with: {e}"))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .map_err(|e| format!("Couldn't open the graphics device: {e}"))?;

    let (w, h, pixels) = woodsketch_render::picture::pixels(&device, &queue, doc, options)?;
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
