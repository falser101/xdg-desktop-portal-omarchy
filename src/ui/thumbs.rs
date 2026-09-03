//! Async image thumbnails for the FileChooser.
//!
//! Visible image rows load in a small worker pool. Results stay in memory for
//! the dialog lifetime and are also written under
//! `$XDG_CACHE_HOME/xdg-desktop-portal-omarchy/thumbs/` so the next open of
//! the same files is cheap.

use egui::{ColorImage, Color32, Rect, TextureHandle, TextureOptions, Vec2};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::UNIX_EPOCH;

const MAX_INFLIGHT: usize = 3;
const MAX_FILE_BYTES: u64 = 80 * 1024 * 1024;
const LIST_PX: u32 = 64;
const PREVIEW_PX: u32 = 480;

struct Job {
    key: String,
    path: PathBuf,
    px: u32,
    cover: bool,
}

struct Ready {
    key: String,
    img: Option<ColorImage>,
}

enum Slot {
    Ready(TextureHandle),
    Miss,
}

pub struct ThumbCache {
    slots: HashMap<String, Slot>,
    dims: HashMap<PathBuf, Option<(u32, u32)>>,
    inflight: HashSet<String>,
    queued: HashSet<String>,
    queue: VecDeque<Job>,
    tx: Sender<Ready>,
    rx: Receiver<Ready>,
}

impl Default for ThumbCache {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            slots: HashMap::new(),
            dims: HashMap::new(),
            inflight: HashSet::new(),
            queued: HashSet::new(),
            queue: VecDeque::new(),
            tx,
            rx,
        }
    }
}

impl ThumbCache {
    pub fn poll(&mut self, ctx: &egui::Context) {
        let mut got = false;
        while let Ok(ready) = self.rx.try_recv() {
            got = true;
            self.inflight.remove(&ready.key);
            let tex = ready.img.map(|img| {
                ctx.load_texture(
                    format!("thumb:{}", ready.key),
                    img,
                    TextureOptions::LINEAR,
                )
            });
            self.slots.insert(
                ready.key,
                match tex {
                    Some(t) => Slot::Ready(t),
                    None => Slot::Miss,
                },
            );
        }
        self.pump();
        if got {
            ctx.request_repaint();
        } else if !self.queue.is_empty() || !self.inflight.is_empty() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }

    /// Paint a square list thumbnail. Returns true when a decoded image was drawn.
    pub fn paint_list(&mut self, ui: &egui::Ui, path: &Path, is_dir: bool, rect: Rect) -> bool {
        if is_dir || !is_image_file(path) {
            return false;
        }
        let Some(tex) = self.texture(ui.ctx(), path, LIST_PX, true) else {
            return false;
        };
        paint_image(ui, &tex, rect, true);
        true
    }

    pub fn preview_tex(&mut self, ctx: &egui::Context, path: &Path) -> Option<TextureHandle> {
        if !is_image_file(path) {
            return None;
        }
        self.texture(ctx, path, PREVIEW_PX, false)
    }

    pub fn dimensions(&mut self, path: &Path) -> Option<(u32, u32)> {
        if let Some(hit) = self.dims.get(path) {
            return *hit;
        }
        let dims = image::image_dimensions(path).ok();
        self.dims.insert(path.to_path_buf(), dims);
        dims
    }

    fn texture(
        &mut self,
        _ctx: &egui::Context,
        path: &Path,
        px: u32,
        cover: bool,
    ) -> Option<TextureHandle> {
        let key = cache_key(path, px, cover);
        match self.slots.get(&key) {
            Some(Slot::Ready(tex)) => return Some(tex.clone()),
            Some(Slot::Miss) => return None,
            None => {}
        }
        self.enqueue(path.to_path_buf(), px, cover);
        None
    }

    fn enqueue(&mut self, path: PathBuf, px: u32, cover: bool) {
        let key = cache_key(&path, px, cover);
        if self.slots.contains_key(&key)
            || self.inflight.contains(&key)
            || self.queued.contains(&key)
        {
            return;
        }
        self.queued.insert(key.clone());
        self.queue.push_back(Job {
            key,
            path,
            px,
            cover,
        });
        self.pump();
    }

    fn pump(&mut self) {
        while self.inflight.len() < MAX_INFLIGHT {
            let Some(job) = self.queue.pop_front() else {
                break;
            };
            self.queued.remove(&job.key);
            self.inflight.insert(job.key.clone());
            let tx = self.tx.clone();
            let _ = std::thread::Builder::new()
                .name("portal-thumb".into())
                .spawn(move || {
                    let img = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        decode_job(&job.path, job.px, job.cover)
                    }))
                    .ok()
                    .flatten();
                    let _ = tx.send(Ready { key: job.key, img });
                });
        }
    }
}

pub fn is_image_file(path: &Path) -> bool {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    mime.type_() == mime_guess::mime::IMAGE
}

pub fn paint_image(ui: &egui::Ui, tex: &TextureHandle, rect: Rect, cover: bool) {
    let uv = if cover {
        let size = tex.size_vec2();
        if size.x <= 0.0 || size.y <= 0.0 || rect.width() <= 0.0 || rect.height() <= 0.0 {
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
        } else {
            let tex_a = size.x / size.y;
            let rec_a = rect.width() / rect.height();
            if tex_a > rec_a {
                let w = rec_a / tex_a;
                let x = (1.0 - w) * 0.5;
                Rect::from_min_max(egui::pos2(x, 0.0), egui::pos2(x + w, 1.0))
            } else {
                let h = tex_a / rec_a;
                let y = (1.0 - h) * 0.5;
                Rect::from_min_max(egui::pos2(0.0, y), egui::pos2(1.0, y + h))
            }
        }
    } else {
        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
    };
    let dest = if cover {
        rect
    } else {
        contain_rect(tex.size_vec2(), rect)
    };
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    painter.image(tex.id(), dest, uv, Color32::WHITE);
}

fn contain_rect(size: Vec2, rect: Rect) -> Rect {
    if size.x <= 0.0 || size.y <= 0.0 {
        return rect;
    }
    let scale = (rect.width() / size.x).min(rect.height() / size.y);
    Rect::from_center_size(rect.center(), size * scale)
}

fn cache_key(path: &Path, px: u32, cover: bool) -> String {
    let (mtime, len) = std::fs::metadata(path)
        .ok()
        .map(|m| {
            let mt = m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            (mt, m.len())
        })
        .unwrap_or((0, 0));
    format!(
        "{}|{mtime}|{len}|{px}|{}",
        path.display(),
        if cover { "c" } else { "f" }
    )
}

fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn disk_cache_path(key: &str) -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("xdg-desktop-portal-omarchy/thumbs");
    Some(dir.join(format!("{:016x}.png", fnv1a(key))))
}

fn load_disk(key: &str) -> Option<ColorImage> {
    let path = disk_cache_path(key)?;
    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    Some(ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    ))
}

fn save_disk(key: &str, img: &ColorImage) {
    let Some(path) = disk_cache_path(key) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut rgba = Vec::with_capacity(img.width() * img.height() * 4);
    for p in &img.pixels {
        rgba.extend_from_slice(&[p.r(), p.g(), p.b(), p.a()]);
    }
    let Some(buf) = image::RgbaImage::from_raw(img.width() as u32, img.height() as u32, rgba) else {
        return;
    };
    let _ = buf.save(path);
}

fn decode_job(path: &Path, px: u32, cover: bool) -> Option<ColorImage> {
    let key = cache_key(path, px, cover);
    if let Some(hit) = load_disk(&key) {
        return Some(hit);
    }
    let img = decode_thumb(path, px, cover)?;
    save_disk(&key, &img);
    Some(img)
}

pub fn decode_thumb(path: &Path, px: u32, cover: bool) -> Option<ColorImage> {
    let px = px.clamp(16, 1024);
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() == 0 || meta.len() > MAX_FILE_BYTES {
        return None;
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "svg" {
        return rasterize_svg(path, px);
    }
    let mut reader = image::ImageReader::open(path).ok()?;
    reader = reader.with_guessed_format().ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16_384);
    limits.max_image_height = Some(16_384);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let img = reader.decode().ok()?;
    Some(fit_image(img, px, cover))
}

fn fit_image(img: image::DynamicImage, px: u32, cover: bool) -> ColorImage {
    let out = if cover {
        let w = img.width();
        let h = img.height();
        if w == 0 || h == 0 {
            img
        } else {
            let scale = px as f32 / w.min(h) as f32;
            let nw = (w as f32 * scale).round().max(px as f32) as u32;
            let nh = (h as f32 * scale).round().max(px as f32) as u32;
            let img = img.resize_exact(nw, nh, image::imageops::FilterType::Triangle);
            let x = img.width().saturating_sub(px) / 2;
            let y = img.height().saturating_sub(px) / 2;
            img.crop_imm(x, y, px.min(img.width()), px.min(img.height()))
        }
    } else {
        img.resize(px, px, image::imageops::FilterType::Triangle)
    };
    let rgba = out.to_rgba8();
    ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    )
}

fn rasterize_svg(path: &Path, px: u32) -> Option<ColorImage> {
    let data = std::fs::read(path).ok()?;
    let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default()).ok()?;
    let size = tree.size();
    let w = size.width().max(1.0);
    let h = size.height().max(1.0);
    let scale = px as f32 / w.max(h);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(px, px)?;
    let tx = (px as f32 - w * scale) * 0.5;
    let ty = (px as f32 - h * scale) * 0.5;
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(tx, ty);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Some(ColorImage::from_rgba_premultiplied(
        [px as usize, px as usize],
        pixmap.data(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_image_extensions() {
        assert!(is_image_file(Path::new("photo.jpg")));
        assert!(is_image_file(Path::new("photo.JPEG")));
        assert!(is_image_file(Path::new("a.png")));
        assert!(is_image_file(Path::new("a.webp")));
        assert!(is_image_file(Path::new("a.svg")));
        assert!(!is_image_file(Path::new("song.mp3")));
        assert!(!is_image_file(Path::new("notes.pdf")));
        assert!(!is_image_file(Path::new("file")));
    }

    #[test]
    fn cover_thumb_is_square() {
        let path = temp_png("cover", 12, 6, [200, 40, 40, 255]);
        let img = decode_thumb(&path, 32, true).expect("decode cover");
        assert_eq!(img.width(), 32);
        assert_eq!(img.height(), 32);
        assert!(img.pixels.iter().any(|c| c.a() > 0));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn contain_thumb_keeps_aspect() {
        let path = temp_png("contain", 16, 8, [40, 80, 200, 255]);
        let img = decode_thumb(&path, 32, false).expect("decode contain");
        assert_eq!(img.width(), 32);
        assert_eq!(img.height(), 16);
        let _ = std::fs::remove_file(path);
    }

    fn temp_png(tag: &str, w: u32, h: u32, rgba: [u8; 4]) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("omarchy-thumb-{tag}-{nanos}.png"));
        let mut img = image::RgbaImage::new(w, h);
        for p in img.pixels_mut() {
            *p = image::Rgba(rgba);
        }
        img.save(&path).expect("write test png");
        path
    }
}
