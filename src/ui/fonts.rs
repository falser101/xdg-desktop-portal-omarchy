use egui::{FontData, FontDefinitions, FontFamily};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static NERD: AtomicBool = AtomicBool::new(false);

pub fn has_nerd() -> bool {
    NERD.load(Ordering::Relaxed)
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut nerd = false;

    if let Some(data) = load_first(&[
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/TTF/NotoSans-Regular.ttf",
        "/usr/share/fonts/inter/Inter-Regular.ttf",
    ])
    .or_else(|| load_under_data_home("Inter/extras/ttf/Inter-Regular.ttf"))
    {
        insert(&mut fonts, "ui", data);
        prefer(&mut fonts, FontFamily::Proportional, "ui");
    }

    if let Some(data) = load_first(&[
        "/usr/share/fonts/OTF/NotoSansCJKsc-Regular.otf",
        "/usr/share/fonts/adobe-source-han-sans/SourceHanSansCN-Regular.otf",
        "/usr/share/fonts/noto-cjk/NotoSansCJKsc-Regular.otf",
    ]) {
        insert(&mut fonts, "cjk", data);
        fallback(&mut fonts, FontFamily::Proportional, "cjk");
        fallback(&mut fonts, FontFamily::Monospace, "cjk");
    }

    if let Some(data) = load_first(&[
        "/usr/share/fonts/TTF/CaskaydiaCoveNerdFont-Regular.ttf",
        "/usr/share/fonts/OTF/CaskaydiaCoveNerdFont-Regular.otf",
    ])
    .or_else(|| load_under_data_home("CascadiaCove/CaskaydiaCoveNerdFont-Regular.ttf"))
    .or_else(|| load_under_data_home("CaskaydiaCoveNerdFont-Regular.ttf"))
    {
        insert(&mut fonts, "nerd", data);
        fallback(&mut fonts, FontFamily::Proportional, "nerd");
        prefer(&mut fonts, FontFamily::Monospace, "nerd");
        nerd = true;
    }

    NERD.store(nerd, Ordering::Relaxed);
    ctx.set_fonts(fonts);
}

/// Nerd Font / ASCII glyph for a directory or file name.
pub fn file_glyph(is_dir: bool, name: &str) -> &'static str {
    let nerd = has_nerd();
    if is_dir {
        return if nerd { "\u{f07b}" } else { "▸" };
    }
    let ext = Path::new(name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" | "opus" | "wma" => {
            if nerd { "\u{f001}" } else { "♪" }
        }
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" | "avif" => {
            if nerd { "\u{f03e}" } else { "▣" }
        }
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "m4v" => {
            if nerd { "\u{f03d}" } else { "▶" }
        }
        "pdf" => {
            if nerd { "\u{f1c1}" } else { "P" }
        }
        "zip" | "tar" | "gz" | "tgz" | "7z" | "rar" | "xz" | "bz2" => {
            if nerd { "\u{f1c6}" } else { "Z" }
        }
        "rs" | "py" | "js" | "ts" | "go" | "c" | "h" | "cpp" | "java" | "kt" | "rb" | "sh"
        | "qml" | "toml" | "json" | "yml" | "yaml" | "xml" | "html" | "css" => {
            if nerd { "\u{f1c9}" } else { "{}" }
        }
        "txt" | "md" | "log" | "rst" => {
            if nerd { "\u{f0f6}" } else { "T" }
        }
        _ => {
            if nerd { "\u{f15b}" } else { "·" }
        }
    }
}

pub fn up_glyph() -> &'static str {
    if has_nerd() { "\u{f062}" } else { ".." }
}

fn insert(fonts: &mut FontDefinitions, name: &str, data: Vec<u8>) {
    fonts
        .font_data
        .insert(name.to_owned(), FontData::from_owned(data).into());
}

fn prefer(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
    let list = fonts.families.entry(family).or_default();
    list.retain(|n| n != name);
    list.insert(0, name.to_owned());
}

fn fallback(fonts: &mut FontDefinitions, family: FontFamily, name: &str) {
    let list = fonts.families.entry(family).or_default();
    if !list.iter().any(|n| n == name) {
        list.push(name.to_owned());
    }
}

fn load_first(paths: &[&str]) -> Option<Vec<u8>> {
    paths.iter().find_map(|p| std::fs::read(p).ok())
}

fn load_under_data_home(rel: &str) -> Option<Vec<u8>> {
    let base = dirs::data_local_dir()?.join("fonts").join(rel);
    std::fs::read(base).ok()
}

#[cfg(test)]
mod tests {
    use super::file_glyph;

    #[test]
    fn audio_files_are_not_generic_docs() {
        let g = file_glyph(false, "tts_20260415_103612.mp3");
        assert_ne!(g, "📄");
        assert!(g == "\u{f001}" || g == "♪");
    }

    #[test]
    fn directories_differ_from_files() {
        assert_ne!(file_glyph(true, "Music"), file_glyph(false, "x.bin"));
    }
}
