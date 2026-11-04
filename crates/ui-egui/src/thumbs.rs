//! Little pictures of drawings for the Start page's cards.
//!
//! Making one is slow enough to be felt: a sample has to be built from its recipe (~200 ms) and
//! a recent drawing read off disk, before either is meshed and drawn. So the work happens on a
//! thread of its own and the cards show their text until a picture arrives. The thread draws on
//! the device the app is already running on ([`woodsketch_render::picture`] takes one rather
//! than opening its own), so nothing here sets up a second GPU.
//!
//! Drawn pictures are kept in the settings folder, so the second run of the app shows the Start
//! page with its pictures already on it. A cache file's name carries everything that decides
//! whether it is still right ([`cache_name`]), so a stale one is never found rather than found
//! and checked; nothing here has to be invalidated by hand.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui;
use woodsketch_doc::Document;

/// Drawn at twice the size they're shown at, so they stay sharp on a scaled-up screen.
const RENDER: [u32; 2] = [496, 280];
/// What a card gives a picture, in points.
pub const SIZE: egui::Vec2 = egui::vec2(248.0, 140.0);
/// How many cache files to keep. The recent list holds ten, and every drawing that leaves it
/// leaves its pictures behind, so the folder is trimmed to a few lists' worth at startup.
const KEEP: usize = 40;

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

/// Where pictures are kept between runs, and the work of reading and writing them there.
struct Cache {
    dir: PathBuf,
}

impl Thumbnails {
    /// Start the thread. Without a GPU (the tests), nothing is drawn and every card stays text.
    /// Without a folder to keep them in, pictures are drawn afresh every run.
    pub fn new(render_state: Option<&egui_wgpu::RenderState>, cache_dir: Option<PathBuf>) -> Self {
        let cache = cache_dir.map(|dir| Cache { dir });
        if let Some(cache) = &cache {
            cache.prepare();
        }
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
                        let picture = make(&device, &queue, &of, cache.as_ref());
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

/// The picture of `of`: the one kept from a previous run if it is still right, else drawn now
/// and kept. Runs on the thumbnail thread.
fn make(device: &wgpu::Device, queue: &wgpu::Queue, of: &Of, cache: Option<&Cache>) -> Option<(u32, u32, Vec<u8>)> {
    let file = cache.and_then(|cache| Some(cache.dir.join(cache_name(of)?)));
    if let Some(file) = &file
        && let Some(picture) = read_png(file)
    {
        return Some(picture);
    }
    let picture = draw(device, queue, of)?;
    if let Some(file) = &file {
        // A picture that can't be written is still shown; it is only drawn again next run.
        let _ = write_png(file, &picture);
        if let Some(cache) = cache {
            cache.forget_older(of);
        }
    }
    Some(picture)
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

/// What makes a kept picture still right, as its file name.
///
/// Everything that would make a picture wrong is in the name, so a lookup either finds a picture
/// that is certainly current or finds nothing. A drawing's name carries when it was last written
/// and how long it is (what [`crate::watch`] compares, for the same reason: no reading the whole
/// file to notice a change); a sample's carries the app's version, since a sample is built from
/// a recipe inside the binary and changes only when the app does.
///
/// `None` for anything whose name can't be worked out, which is then simply not kept.
fn cache_name(of: &Of) -> Option<String> {
    match of {
        Of::Sample(i) => Some(format!("sample-{i}-{}.png", crate::updates::VERSION)),
        Of::Drawing(path) => {
            let about = std::fs::metadata(path).ok()?;
            let written = about.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
            Some(format!("{}-{written}-{}.png", fingerprint(path), about.len()))
        }
    }
}

/// What a file name starts with for `of`, to find the ones it has left behind.
fn cache_prefix(of: &Of) -> Option<String> {
    match of {
        Of::Sample(i) => Some(format!("sample-{i}-")),
        Of::Drawing(path) => Some(format!("{}-", fingerprint(path))),
    }
}

/// A path as a short, settled string: FNV-1a, written out here rather than taken from the
/// standard library's hasher, whose results are not promised to stay the same between Rust
/// versions. A name that changed under us would quietly fill the folder with pictures nothing
/// ever looks for again.
fn fingerprint(path: &Path) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in path.as_os_str().to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    format!("{hash:016x}")
}

impl Cache {
    /// Make the folder, and keep it from growing without end: every drawing that falls off the
    /// recent list, and every sample from a version that has been updated past, leaves a picture
    /// behind that nothing will ask for again.
    fn prepare(&self) {
        if std::fs::create_dir_all(&self.dir).is_err() {
            return;
        }
        let Ok(entries) = std::fs::read_dir(&self.dir) else { return };
        let mut files: Vec<_> = entries
            .flatten()
            .filter_map(|e| {
                let written = e.metadata().ok()?.modified().ok()?;
                Some((written, e.path()))
            })
            .collect();
        if files.len() <= KEEP {
            return;
        }
        // Oldest first, and drop enough of them to come back under the limit.
        files.sort_by_key(|(written, _)| *written);
        for (_, path) in files.iter().take(files.len() - KEEP) {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Throw away the pictures this thing used to have: a drawing saved again, or a sample from
    /// the version before this one. Without this a drawing edited daily would leave a picture a
    /// day behind.
    fn forget_older(&self, of: &Of) {
        let (Some(prefix), Some(current)) = (cache_prefix(of), cache_name(of)) else { return };
        let Ok(entries) = std::fs::read_dir(&self.dir) else { return };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(&prefix) && name != current {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

/// A kept picture, or `None` if there isn't one or it can't be read.
fn read_png(file: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(file).ok()?));
    let mut reader = decoder.read_info().ok()?;
    let mut pixels = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut pixels).ok()?;
    // Only what `write_png` writes is understood; anything else is treated as not being there,
    // and drawn again.
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    pixels.truncate(info.buffer_size());
    Some((info.width, info.height, pixels))
}

/// Keep a picture for next time, whole or not at all: written beside and renamed over, so a run
/// that stops halfway through can't leave a half-written file to be read back as a picture.
fn write_png(file: &Path, (width, height, pixels): &(u32, u32, Vec<u8>)) -> std::io::Result<()> {
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, *width, *height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(pixels)?;
    }
    woodsketch_doc::file::write_safely(file, &encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("woodsketch-thumbs-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Nothing that would make a kept picture wrong is left out of its name, so a lookup can
    /// trust what it finds without opening it.
    #[test]
    fn a_name_changes_when_the_drawing_does() {
        let dir = scratch("names");
        let path = dir.join("piece.woodsketch");
        std::fs::write(&path, "one").unwrap();
        let of = Of::Drawing(path.clone());
        let first = cache_name(&of).expect("a drawing that exists has a name");

        // Longer, and written later: either on its own has to be enough.
        std::fs::write(&path, "one and a bit more").unwrap();
        let second = cache_name(&of).expect("still there");
        assert_ne!(first, second, "a drawing written again needs a new name");
        assert!(second.starts_with(&cache_prefix(&of).unwrap()), "both belong to the same drawing");

        // A sample's picture is only out of date when the app is, since its recipe ships inside.
        let sample = cache_name(&Of::Sample(0)).unwrap();
        assert!(sample.contains(crate::updates::VERSION), "a sample's name carries the version: {sample}");
        assert_eq!(cache_name(&Of::Drawing(dir.join("not here.woodsketch"))), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A picture written out comes back the same, and the one it replaces is cleared away
    /// rather than left for nothing to ask for.
    #[test]
    fn a_kept_picture_comes_back_and_the_old_one_goes() {
        let dir = scratch("keep");
        let path = dir.join("piece.woodsketch");
        std::fs::write(&path, "one").unwrap();
        let of = Of::Drawing(path.clone());
        let cache = Cache { dir: dir.join("cache") };
        cache.prepare();

        let picture = (2, 2, vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 1, 2, 3, 4]);
        let first = cache.dir.join(cache_name(&of).unwrap());
        write_png(&first, &picture).unwrap();
        assert_eq!(read_png(&first), Some(picture.clone()), "a picture reads back as it was written");

        // Written again, so the name changes; the one before it shouldn't be left behind.
        std::fs::write(&path, "one and a bit more").unwrap();
        let second = cache.dir.join(cache_name(&of).unwrap());
        write_png(&second, &picture).unwrap();
        cache.forget_older(&of);
        assert!(second.exists(), "the current picture stays");
        assert!(!first.exists(), "the one it replaced goes");

        assert_eq!(read_png(&cache.dir.join("nothing.png")), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The folder doesn't grow forever: drawings leave the recent list, and their pictures with
    /// them.
    #[test]
    fn the_oldest_pictures_are_cleared_away() {
        let dir = scratch("prune");
        let cache = Cache { dir: dir.join("cache") };
        cache.prepare();
        let picture = (1, 1, vec![9, 9, 9, 255]);
        for i in 0..KEEP + 6 {
            write_png(&cache.dir.join(format!("{i:04}.png")), &picture).unwrap();
            // Distinct modified times, so "oldest" means something on a fast filesystem.
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        cache.prepare();
        let left: Vec<_> = std::fs::read_dir(&cache.dir).unwrap().flatten().map(|e| e.file_name()).collect();
        assert_eq!(left.len(), KEEP, "trimmed back to the limit");
        assert!(!left.iter().any(|n| n == "0000.png"), "the oldest went first");
        assert!(left.iter().any(|n| n == format!("{:04}.png", KEEP + 5).as_str()), "the newest stayed");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
