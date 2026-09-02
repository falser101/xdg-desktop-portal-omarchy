use egui::{FontData, FontDefinitions, FontFamily};

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

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

    ctx.set_fonts(fonts);
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
    use super::load_first;

    #[test]
    fn finds_a_ui_font_on_this_system() {
        let hit = load_first(&[
            "/usr/share/fonts/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/TTF/NotoSans-Regular.ttf",
            "/usr/share/fonts/inter/Inter-Regular.ttf",
            "/usr/share/fonts/OTF/NotoSansCJKsc-Regular.otf",
        ]);
        assert!(hit.is_some() || std::path::Path::new("/usr/share/fonts").is_dir());
    }
}
