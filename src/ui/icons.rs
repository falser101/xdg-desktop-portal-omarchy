use egui::{ColorImage, TextureHandle, TextureOptions, Vec2};
use std::collections::HashMap;
use std::path::Path;

const ICON_PT: f32 = 16.0;

#[derive(Default)]
pub struct IconCache {
    textures: HashMap<String, Option<TextureHandle>>,
}

impl IconCache {
    pub fn show(&mut self, ui: &mut egui::Ui, names: &[String]) {
        self.paint(ui, names);
    }

    pub fn show_names(&mut self, ui: &mut egui::Ui, names: &[&str]) {
        let owned: Vec<String> = names.iter().map(|s| (*s).to_string()).collect();
        self.paint(ui, &owned);
    }

    pub fn button(&mut self, ui: &mut egui::Ui, names: &[&str], fallback: &str) -> bool {
        let owned: Vec<String> = names.iter().map(|s| (*s).to_string()).collect();
        if let Some(tex) = self.get(ui.ctx(), &owned) {
            ui.add(egui::ImageButton::new((tex.id(), Vec2::splat(ICON_PT))))
                .clicked()
        } else {
            ui.button(fallback).clicked()
        }
    }

    pub fn paint_at(&mut self, ui: &egui::Ui, names: &[String], rect: egui::Rect) {
        if let Some(tex) = self.get(ui.ctx(), names) {
            ui.painter().image(
                tex.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }

    fn paint(&mut self, ui: &mut egui::Ui, names: &[String]) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(ICON_PT), egui::Sense::hover());
        self.paint_at(ui, names, rect);
    }

    fn get(&mut self, ctx: &egui::Context, names: &[String]) -> Option<TextureHandle> {
        let key = names.join("|");
        if let Some(hit) = self.textures.get(&key) {
            return hit.clone();
        }
        let px = ((ICON_PT * ctx.pixels_per_point()).round() as u32).clamp(16, 64);
        let tex = crate::desktop::resolve_file_icon(names).and_then(|path| {
            load_pixels(&path, px).map(|img| {
                ctx.load_texture(format!("icon:{key}"), img, TextureOptions::LINEAR)
            })
        });
        self.textures.insert(key, tex.clone());
        tex
    }
}

fn load_pixels(path: &Path, px: u32) -> Option<ColorImage> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "svg" {
        return rasterize_svg(path, px);
    }
    let img = image::open(path).ok()?;
    let img = img.resize(px, px, image::imageops::FilterType::Triangle);
    let rgba = img.to_rgba8();
    Some(ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rasterize_mp3_theme_icon() {
        let names = crate::desktop::file_icon_names(false, Path::new("tts.mp3"));
        let path = crate::desktop::resolve_file_icon(&names).expect("mp3 icon path");
        let img = load_pixels(&path, 32).expect(&format!("rasterize {path:?}"));
        assert_eq!(img.width(), 32);
        assert_eq!(img.height(), 32);
        assert!(img.pixels.iter().any(|c| c.a() > 0), "icon is not empty");
    }
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
