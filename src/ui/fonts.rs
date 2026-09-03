use egui::{FontData, FontDefinitions, FontFamily};
use std::path::PathBuf;
use std::process::Command;

/// Noto Sans CJK *.ttc face order: JP, KR, SC, TC, HK.
const CJK_SC: u32 = 2;

pub fn install(ctx: &egui::Context, family: &str) {
    let mut fonts = FontDefinitions::default();
    let family = family.trim();
    let family = if family.is_empty() { "Inter" } else { family };

    if let Some((path, index)) = fc_match(&format!("{family}:style=Regular"))
        .or_else(|| fc_match(family))
        .or_else(|| fallback_ui_regular())
    {
        if let Some(data) = std::fs::read(&path).ok() {
            insert(&mut fonts, "ui", data, index);
            prefer(&mut fonts, FontFamily::Proportional, "ui");
        }
    }

    if let Some((path, index)) = fc_match(&format!("{family}:style=Medium"))
        .or_else(|| fc_match(&format!("{family}:style=SemiBold")))
        .or_else(|| fallback_ui_medium())
    {
        if let Some(data) = std::fs::read(&path).ok() {
            insert(&mut fonts, "ui-medium", data, index);
        }
    }

    let semibold = FontFamily::Name("semibold".into());
    let list = fonts.families.entry(semibold.clone()).or_default();
    if fonts.font_data.contains_key("ui-medium") {
        list.insert(0, "ui-medium".into());
    }
    if fonts.font_data.contains_key("ui") && !list.iter().any(|n| n == "ui") {
        list.push("ui".into());
    }
    if let Some(prop) = fonts.families.get(&FontFamily::Proportional).cloned() {
        let list = fonts.families.entry(semibold.clone()).or_default();
        for name in prop {
            if !list.iter().any(|n| *n == name) {
                list.push(name);
            }
        }
    }

    if let Some((data, index)) = load_cjk_regular() {
        insert(&mut fonts, "cjk", data, index);
        fallback(&mut fonts, FontFamily::Proportional, "cjk");
        fallback(&mut fonts, FontFamily::Monospace, "cjk");
        fallback(&mut fonts, semibold, "cjk");
    }

    ctx.set_fonts(fonts);
}

fn insert(fonts: &mut FontDefinitions, name: &str, data: Vec<u8>, index: u32) {
    let mut font = FontData::from_owned(data);
    font.index = index;
    fonts.font_data.insert(name.to_owned(), font.into());
}

/// Single-face OTF/TTF first; Arch/Omarchy ship a TTC (SC is face 2).
fn load_cjk_regular() -> Option<(Vec<u8>, u32)> {
    if let Some(data) = load_first(&[
        "/usr/share/fonts/OTF/NotoSansCJKsc-Regular.otf",
        "/usr/share/fonts/adobe-source-han-sans/SourceHanSansCN-Regular.otf",
        "/usr/share/fonts/noto-cjk/NotoSansCJKsc-Regular.otf",
    ]) {
        return Some((data, 0));
    }
    load_first(&["/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc"]).map(|data| (data, CJK_SC))
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

fn fc_match(query: &str) -> Option<(PathBuf, u32)> {
    let out = Command::new("fc-match")
        .args([query, "-f", "%{file}\n%{index}\n"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut lines = text.lines();
    let file = lines.next()?.trim();
    if file.is_empty() {
        return None;
    }
    let index = lines
        .next()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);
    Some((PathBuf::from(file), index))
}

fn fallback_ui_regular() -> Option<(PathBuf, u32)> {
    first_existing(&[
        "/usr/share/fonts/inter/Inter-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/TTF/NotoSans-Regular.ttf",
    ])
    .or_else(|| {
        dirs::data_local_dir().and_then(|d| {
            first_existing(&[d.join("fonts/Inter/extras/ttf/Inter-Regular.ttf")])
        })
    })
}

fn fallback_ui_medium() -> Option<(PathBuf, u32)> {
    first_existing(&[
        "/usr/share/fonts/inter/Inter-Medium.ttf",
        "/usr/share/fonts/noto/NotoSans-Medium.ttf",
        "/usr/share/fonts/TTF/NotoSans-Medium.ttf",
        "/usr/share/fonts/noto/NotoSans-Bold.ttf",
        "/usr/share/fonts/inter/Inter-SemiBold.ttf",
    ])
}

fn first_existing<P: AsRef<std::path::Path>>(paths: &[P]) -> Option<(PathBuf, u32)> {
    paths.iter().find_map(|p| {
        let p = p.as_ref();
        p.is_file().then(|| (p.to_path_buf(), 0))
    })
}

fn load_first(paths: &[&str]) -> Option<Vec<u8>> {
    paths.iter().find_map(|p| std::fs::read(p).ok())
}

#[cfg(test)]
mod tests {
    use super::load_first;

    #[test]
    fn finds_a_ui_font_on_this_system() {
        let hit = load_first(&[
            "/usr/share/fonts/inter/Inter-Regular.ttf",
            "/usr/share/fonts/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/noto/NotoSans-Medium.ttf",
            "/usr/share/fonts/TTF/NotoSans-Regular.ttf",
            "/usr/share/fonts/OTF/NotoSansCJKsc-Regular.otf",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        ]);
        assert!(hit.is_some() || std::path::Path::new("/usr/share/fonts").is_dir());
    }

    #[test]
    fn fc_match_resolves_a_ui_family() {
        let hit = super::fc_match("Inter:style=Regular").or_else(|| super::fc_match("sans-serif"));
        if hit.is_none() {
            return;
        }
        let (path, _) = hit.unwrap();
        assert!(path.is_file(), "fc-match path missing: {path:?}");
    }

    #[test]
    fn cjk_regular_loads_on_this_system() {
        let Some((data, index)) = super::load_cjk_regular() else {
            return;
        };
        assert!(data.len() > 1000, "CJK font is too small");
        if data.starts_with(b"ttcf") {
            assert_eq!(index, super::CJK_SC);
        } else {
            assert_eq!(index, 0);
        }
    }
}
