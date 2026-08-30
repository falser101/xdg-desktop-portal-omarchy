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
            return parse_desktop(&path, &portal_app_id(id));
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
            if let Some(app) = parse_desktop(&path, &portal_app_id(&file)) {
                if app.no_display {
                    continue;
                }
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
}
