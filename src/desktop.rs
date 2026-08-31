use crate::paths::{config_home, data_home};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct DesktopApp {
    pub id: String,
    pub name: String,
    pub comment: String,
    pub icon: String,
    pub exec: String,
    pub no_display: bool,
}

/// xdg-desktop-portal appends ".desktop" itself in OpenURI
/// (`g_strconcat(choice_id, ".desktop")`). Returning "zen.desktop"
/// makes it look up "zen.desktop.desktop" and the launch is a no-op.
pub fn portal_app_id(id: &str) -> String {
    id.trim_end_matches(".desktop").to_string()
}

pub fn load_app(id: &str) -> Option<DesktopApp> {
    let file = if id.ends_with(".desktop") {
        id.to_string()
    } else {
        format!("{id}.desktop")
    };
    for dir in application_dirs() {
        let path = dir.join(&file);
        if path.is_file() {
            let mut app = parse_desktop(&path, &portal_app_id(id))?;
            enrich_icon(&mut app);
            return Some(app);
        }
    }
    None
}

pub fn load_apps(ids: &[String]) -> Vec<DesktopApp> {
    if ids.is_empty() {
        return list_all();
    }
    ids.iter().filter_map(|id| load_app(id)).collect()
}

/// Prefer an absolute icon path so QML does not depend on the active icon
/// theme (breeze-dark often lacks third-party app icons that live in
/// hicolor / Papirus / pixmaps).
fn enrich_icon(app: &mut DesktopApp) {
    if app.icon.is_empty() {
        if let Some(name) = inherit_icon_name(&app.id) {
            app.icon = name;
        } else {
            for cand in icon_name_candidates(&app.id) {
                if resolve_icon_path(&cand).is_some() {
                    app.icon = cand;
                    break;
                }
            }
        }
    }
    if let Some(path) = resolve_icon_path(&app.icon) {
        app.icon = path.to_string_lossy().into_owned();
    }
}

fn icon_name_candidates(id: &str) -> Vec<String> {
    let base = portal_app_id(id);
    let mut out = vec![base.clone()];
    for suffix in ["-url-handler", "-handler", "_url_handler"] {
        if let Some(stripped) = base.strip_suffix(suffix) {
            if !stripped.is_empty() {
                out.push(stripped.to_string());
            }
        }
    }
    if base == "com.google.Chrome" {
        out.push("google-chrome".into());
    }
    if base == "chromium-browser" || base == "Chromium" {
        out.push("chromium".into());
        out.push("google-chrome".into());
    }
    out
}

fn inherit_icon_name(id: &str) -> Option<String> {
    for cand in icon_name_candidates(id) {
        if cand == portal_app_id(id) {
            continue;
        }
        if let Some(other) = load_app_raw(&cand) {
            if !other.icon.is_empty() {
                return Some(other.icon);
            }
        }
    }
    None
}

fn load_app_raw(id: &str) -> Option<DesktopApp> {
    let file = if id.ends_with(".desktop") {
        id.to_string()
    } else {
        format!("{id}.desktop")
    };
    for dir in application_dirs() {
        let path = dir.join(&file);
        if path.is_file() {
            return parse_desktop(&path, &portal_app_id(id));
        }
    }
    None
}

pub fn resolve_icon_path(icon: &str) -> Option<PathBuf> {
    let icon = icon.trim();
    if icon.is_empty() {
        return None;
    }
    if icon.contains("://") {
        return None;
    }
    let path = Path::new(icon);
    if path.is_absolute() && path.is_file() {
        return Some(path.to_path_buf());
    }
    // Theme name → search pixmaps + icon themes (hicolor / Papirus / …).
    let name = icon.trim_end_matches(".png")
        .trim_end_matches(".svg")
        .trim_end_matches(".xpm");

    for ext in ["svg", "png", "xpm"] {
        let pix = PathBuf::from("/usr/share/pixmaps").join(format!("{name}.{ext}"));
        if pix.is_file() {
            return Some(pix);
        }
        let local = data_home().join("pixmaps").join(format!("{name}.{ext}"));
        if local.is_file() {
            return Some(local);
        }
    }

    let sizes = [
        "scalable",
        "512x512",
        "256x256",
        "128x128",
        "64x64",
        "48x48",
        "32x32",
        "24x24",
        "16x16",
    ];
    for root in icon_roots() {
        // Prefer well-known themes that actually ship third-party apps.
        for theme in ["hicolor", "Papirus", "Papirus-Dark", "breeze-dark", "breeze", "Adwaita"] {
            for size in sizes {
                for ext in ["svg", "png"] {
                    for kind in ["apps", "devices"] {
                        let p = root
                            .join(theme)
                            .join(size)
                            .join(kind)
                            .join(format!("{name}.{ext}"));
                        if p.is_file() {
                            return Some(p);
                        }
                    }
                }
            }
        }
        // Any theme: shallow apps/ lookup for common sizes only.
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let theme_dir = entry.path();
                if !theme_dir.is_dir() {
                    continue;
                }
                for size in ["scalable", "128x128", "64x64", "48x48"] {
                    for ext in ["svg", "png"] {
                        let p = theme_dir
                            .join(size)
                            .join("apps")
                            .join(format!("{name}.{ext}"));
                        if p.is_file() {
                            return Some(p);
                        }
                    }
                }
            }
        }
    }
    None
}

fn icon_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        home_dir_icons(),
        data_home().join("icons"),
    ];
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    for part in data_dirs.split(':') {
        if !part.is_empty() {
            roots.push(PathBuf::from(part).join("icons"));
        }
    }
    roots.into_iter().filter(|p| p.is_dir()).collect()
}

fn home_dir_icons() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".icons")
}

/// Write `mime` → `app_id` into `~/.config/mimeapps.list`.
///
/// `app_id` may be with or without `.desktop`; the file always stores the
/// `.desktop` form. Updates `[Default Applications]` and appends to
/// `[Added Associations]` (xdg desktop entry spec).
pub fn set_default_for_mime(mime: &str, app_id: &str) -> std::io::Result<()> {
    let mime = mime.trim();
    if mime.is_empty() {
        return Ok(());
    }
    let desktop = if app_id.ends_with(".desktop") {
        app_id.to_string()
    } else {
        format!("{app_id}.desktop")
    };

    let path = config_home().join("mimeapps.list");
    let text = if path.is_file() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };

    let mut sections = parse_ini_sections(&text);
    {
        let defaults = section_mut(&mut sections, "Default Applications");
        upsert_key(defaults, mime, desktop.clone());
    }
    {
        let added = section_mut(&mut sections, "Added Associations");
        match added.iter().position(|(k, _)| k == mime) {
            Some(i) => {
                let current = &added[i].1;
                if !current.split(';').any(|p| p == desktop) {
                    let mut next = current.trim_end_matches(';').to_string();
                    if !next.is_empty() {
                        next.push(';');
                    }
                    next.push_str(&desktop);
                    next.push(';');
                    added[i].1 = next;
                }
            }
            None => added.push((mime.to_string(), format!("{desktop};"))),
        }
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serialize_ini_sections(&sections))?;
    Ok(())
}

fn parse_ini_sections(text: &str) -> Vec<(String, Vec<(String, String)>)> {
    let mut sections: Vec<(String, Vec<(String, String)>)> = Vec::new();
    let mut current: Option<usize> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(name) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            sections.push((name.to_string(), Vec::new()));
            current = Some(sections.len() - 1);
            continue;
        }
        let Some(idx) = current else {
            continue;
        };
        if let Some((k, v)) = trimmed.split_once('=') {
            sections[idx].1.push((k.to_string(), v.to_string()));
        }
    }
    sections
}

fn section_mut<'a>(
    sections: &'a mut Vec<(String, Vec<(String, String)>)>,
    name: &str,
) -> &'a mut Vec<(String, String)> {
    if let Some(i) = sections.iter().position(|(n, _)| n == name) {
        return &mut sections[i].1;
    }
    sections.push((name.to_string(), Vec::new()));
    let i = sections.len() - 1;
    &mut sections[i].1
}

fn upsert_key(entries: &mut Vec<(String, String)>, key: &str, value: String) {
    if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key) {
        *v = value;
    } else {
        entries.push((key.to_string(), value));
    }
}

fn serialize_ini_sections(sections: &[(String, Vec<(String, String)>)]) -> String {
    let mut out = String::new();
    for (name, entries) in sections {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push('[');
        out.push_str(name);
        out.push_str("]\n");
        for (k, v) in entries {
            out.push_str(k);
            out.push('=');
            out.push_str(v);
            out.push('\n');
        }
    }
    out
}

pub fn list_all() -> Vec<DesktopApp> {
    let mut by_id: HashMap<String, DesktopApp> = HashMap::new();
    for dir in application_dirs() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            let file = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if let Some(mut app) = parse_desktop(&path, &portal_app_id(&file)) {
                if app.no_display {
                    continue;
                }
                enrich_icon(&mut app);
                by_id.entry(app.id.clone()).or_insert(app);
            }
        }
    }
    let mut apps: Vec<_> = by_id.into_values().collect();
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        data_home().join("applications"),
        config_home().join("applications"),
    ];
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    for part in data_dirs.split(':') {
        if !part.is_empty() {
            dirs.push(PathBuf::from(part).join("applications"));
        }
    }
    dirs
}

fn parse_desktop(path: &Path, id: &str) -> Option<DesktopApp> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let mut name = String::new();
    let mut comment = String::new();
    let mut icon = String::new();
    let mut exec = String::new();
    let mut no_display = false;
    let mut hidden = false;
    let mut typ = String::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "Type" => typ = value.to_string(),
            "Name" => name = value.to_string(),
            "Comment" => comment = value.to_string(),
            "Icon" => icon = value.to_string(),
            "Exec" => exec = value.to_string(),
            "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
            "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
            _ => {}
        }
    }

    if hidden || (!typ.is_empty() && typ != "Application") {
        return None;
    }
    if name.is_empty() {
        name = portal_app_id(id);
    }
    Some(DesktopApp {
        id: portal_app_id(id),
        name,
        comment,
        icon,
        exec,
        no_display,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_id_strips_desktop_suffix() {
        assert_eq!(portal_app_id("zen"), "zen");
        assert_eq!(portal_app_id("zen.desktop"), "zen");
        assert_eq!(portal_app_id("org.mozilla.firefox.desktop"), "org.mozilla.firefox");
    }

    #[test]
    fn resolve_known_pixmap_or_hicolor() {
        // At least one of these should exist on a typical Arch desktop.
        let hit = ["discord", "docker", "chatgpt", "google-chrome", "zen-browser"]
            .iter()
            .find_map(|n| resolve_icon_path(n));
        assert!(hit.is_some(), "expected to resolve a common app icon path");
    }

    #[test]
    fn mimeapps_upsert_and_added() {
        let text = "[Default Applications]\ntext/plain=old.desktop\n\n[Added Associations]\ntext/plain=old.desktop;\n";
        let mut sections = parse_ini_sections(text);
        upsert_key(
            section_mut(&mut sections, "Default Applications"),
            "text/plain",
            "new.desktop".into(),
        );
        let added = section_mut(&mut sections, "Added Associations");
        let i = added.iter().position(|(k, _)| k == "text/plain").unwrap();
        let current = added[i].1.clone();
        assert!(current.contains("old.desktop"));
        if !current.split(';').any(|p| p == "new.desktop") {
            added[i].1 = format!("{}new.desktop;", current.trim_end_matches(';').to_string() + ";");
        }
        let out = serialize_ini_sections(&sections);
        assert!(out.contains("text/plain=new.desktop"));
        assert!(out.contains("new.desktop;"));
        assert!(out.find("[Default Applications]").unwrap() < out.find("[Added Associations]").unwrap());
    }
}
