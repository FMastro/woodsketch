//! Little pictures of drawings for the Start page's cards.
//!
//! Making one is slow enough to be felt: a sample has to be built from its recipe (~200 ms) and
//! a recent drawing read off disk, before either is meshed and drawn. So the work happens on a
//! thread of its own and the cards show their text until a picture arrives. The thread draws on
//! the device the app is already running on ([`woodsketch_render::picture`] takes one rather
//! than opening its own), so nothing here sets up a second GPU.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui;
use woodsketch_doc::Document;

/// Drawn at twice the size they're shown at, so they stay sharp on a scaled-up screen.
const RENDER: [u32; 2] = [496, 280];
/// What a card gives a picture, in points.
pub const SIZE: egui::Vec2 = egui::vec2(248.0, 140.0);

/// What a thumbnail is of.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Of {
    /// The sample at this place in `woodsketch_doc::samples::SAMPLES`.
    Sample(usize),
    /// A drawing on disk (the recent list).
    Drawing(PathBuf),
}

type Drawn = (Of, Option<(u32, u32, Vec<u8>)>);

/// Pictures asked for, being drawn, and done. Dropping it ends the thread, because the job
/// channel's other end goes with it.
pub struct Thumbnails {
    ready: HashMap<Of, egui::TextureHandle>,
    /// Asked for already, so a card doesn't ask again every frame.
    asked: HashSet<Of>,
    jobs: Option<Sender<Of>>,
    drawn: Receiver<Drawn>,
}

impl Thumbnails {
    /// Start the thread. Without a GPU (the tests), nothing is drawn and every card stays text.
    pub fn new(render_state: Option<&egui_wgpu::RenderState>) -> Self {
        let (drawn_in, drawn) = channel();
        let jobs = render_state.map(|state| {
            let (jobs, wanted) = channel::<Of>();
            // wgpu's Device and Queue are handles onto shared state: cloning one is cheap,
            // and both are Send, so the thread draws on the app's own device.
            let (device, queue) = (state.device.clone(), state.queue.clone());
            std::thread::Builder::new()
                .name("thumbnails".into())
                .spawn(move || {
                    // Ends when the app drops its end of `wanted`.
                    while let Ok(of) = wanted.recv() {
                        let picture = draw(&device, &queue, &of);
                        if drawn_in.send((of, picture)).is_err() {
                            break;
                        }
                    }
                })
                .ok()
                .map(|_| jobs)
        });
        Self { ready: HashMap::new(), asked: HashSet::new(), jobs: jobs.flatten(), drawn }
    }

    /// Take in whatever the thread finished. Call once a frame before drawing the cards.
    pub fn collect(&mut self, ctx: &egui::Context) {
        while let Ok((of, picture)) = self.drawn.try_recv() {
            // A failure stays in `asked`, so a drawing that can't be read isn't tried again and
            // again; its card just keeps its text.
            if let Some((width, height, rgba)) = picture {
                let image = egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &rgba);
                let name = format!("thumb {of:?}");
                self.ready.insert(of, ctx.load_texture(name, image, egui::TextureOptions::LINEAR));
            }
            ctx.request_repaint();
        }
    }

    /// The picture of `of`, asking for it the first time it's wanted. Cards call this while they
    /// draw, so nothing is made for a card nobody has scrolled to.
    pub fn get(&mut self, of: Of) -> Option<&egui::TextureHandle> {
        if !self.asked.contains(&of)
            && let Some(jobs) = &self.jobs
            && jobs.send(of.clone()).is_ok()
        {
            self.asked.insert(of.clone());
        }
        self.ready.get(&of)
    }

    /// Whether the picture of `of` has been drawn yet.
    pub fn has(&self, of: &Of) -> bool {
        self.ready.contains_key(of)
    }

    /// Forget the picture of a drawing that has been saved over, so it is drawn again.
    pub fn changed(&mut self, path: &std::path::Path) {
        let of = Of::Drawing(path.to_path_buf());
        self.ready.remove(&of);
        self.asked.remove(&of);
    }
}

/// Build or read the drawing and draw it. Runs on the thumbnail thread.
fn draw(device: &wgpu::Device, queue: &wgpu::Queue, of: &Of) -> Option<(u32, u32, Vec<u8>)> {
    let doc = match of {
        Of::Sample(i) => woodsketch_doc::samples::SAMPLES.get(*i)?.build().ok()?,
        Of::Drawing(path) if crate::files::is_sketchup(path) => crate::files::Files::read_sketchup(path).ok()?.0,
        Of::Drawing(path) => crate::files::Files::read(path).ok()?,
    };
    // An empty drawing has no bounds to frame, and would come back as bare sky.
    doc.bounds()?;
    // No axes, and framed to fill the card rather than left where Zoom Extents puts it: a
    // thumbnail is a picture of the piece, not a view someone is about to orbit in.
    let options = woodsketch_render::picture::Options { size: RENDER, axes: false, fit: Some(0.88), ..Default::default() };
    render(device, queue, &doc, &options)
}

fn render(device: &wgpu::Device, queue: &wgpu::Queue, doc: &Document, options: &woodsketch_render::picture::Options) -> Option<(u32, u32, Vec<u8>)> {
    woodsketch_render::picture::pixels(device, queue, doc, options).ok()
}
