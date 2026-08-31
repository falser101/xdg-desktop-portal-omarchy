use std::path::{Path, PathBuf};

pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

pub fn current_theme_dir() -> PathBuf {
    state_home().join("omarchy/current/theme")
}

pub fn current_theme_name_file() -> PathBuf {
    state_home().join("omarchy/current/theme.name")
}

pub fn state_home() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local/state"))
}

pub fn config_home() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| home_dir().join(".config"))
}

pub fn data_home() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| home_dir().join(".local/share"))
}

pub fn user_dir(key: &str, fallback: &str) -> PathBuf {
    let dirs_file = config_home().join("user-dirs.dirs");
    if let Ok(text) = std::fs::read_to_string(dirs_file) {
        let prefix = format!("XDG_{key}_DIR=");
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix(&prefix) {
                let raw = rest.trim_matches('"');
                let expanded = raw.replace("$HOME", &home_dir().to_string_lossy());
                return PathBuf::from(expanded);
            }
        }
    }
    home_dir().join(fallback)
}

/// Sidebar entries for the file chooser: well-known XDG dirs plus GTK bookmarks.
pub fn places() -> Vec<(String, PathBuf)> {
    let home = home_dir();
    let mut out = Vec::new();
    push_place(&mut out, "Home", home.clone());
    push_place(&mut out, "Downloads", user_dir("DOWNLOAD", "Downloads"));
    push_place(&mut out, "Documents", user_dir("DOCUMENTS", "Documents"));
    push_place(&mut out, "Pictures", user_dir("PICTURES", "Pictures"));
    push_place(&mut out, "Videos", user_dir("VIDEOS", "Videos"));
    push_place(&mut out, "Music", user_dir("MUSIC", "Music"));
    push_place(&mut out, "Projects", user_dir("PROJECTS", "Projects"));
    push_place(&mut out, "Computer", PathBuf::from("/"));
    for (path, label) in gtk_bookmarks() {
        push_place(&mut out, &label, path);
    }
    out
}

fn push_place(out: &mut Vec<(String, PathBuf)>, label: &str, path: PathBuf) {
    if !path.exists() {
        return;
    }
    let canon = std::fs::canonicalize(&path).unwrap_or(path);
    if out.iter().any(|(_, p)| *p == canon) {
        return;
    }
    if label != "Home" && label != "Computer" && canon == home_dir() {
        return;
    }
    out.push((label.to_string(), canon));
}

pub const RECENT_PLACE: &str = "recent:";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecentFile {
    pub label: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
}

/// Recent files from the freedesktop `recently-used.xbel` list, newest first.
pub fn recent_files(limit: usize) -> Vec<RecentFile> {
    let path = data_home().join("recently-used.xbel");
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for href in parse_recently_used_xbel(&text) {
        let Some(path) = crate::uri::path_from_file_uri(&href) else {
            continue;
        };
        if !path.exists() || !seen.insert(path.clone()) {
            continue;
        }
        let meta = std::fs::metadata(&path).ok();
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = meta
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        out.push(RecentFile {
            label,
            path,
            is_dir,
            size,
            modified,
        });
        if out.len() >= limit {
            break;
        }
    }
    out
}

/// Bookmark hrefs from a recently-used.xbel document, newest (`visited`) first.
pub fn parse_recently_used_xbel(text: &str) -> Vec<String> {
    let mut items: Vec<(String, String)> = Vec::new();
    let mut search = text;
    while let Some(at) = search.find("<bookmark ") {
        let rest = &search[at..];
        let tag_end = rest.find('>').unwrap_or(rest.len());
        let tag = &rest[..tag_end];
        search = &rest[tag_end.min(rest.len())..];
        let Some(href) = xml_attr(tag, "href") else {
            continue;
        };
        if !href.starts_with("file:") {
            continue;
        }
        let visited = xml_attr(tag, "visited").unwrap_or_default();
        items.push((visited, href));
    }
    items.sort_by(|a, b| b.0.cmp(&a.0));
    items.into_iter().map(|(_, href)| href).collect()
}

fn xml_attr(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let rest = tag.get(start..)?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub fn gtk_bookmarks() -> Vec<(PathBuf, String)> {
    let file = config_home().join("gtk-3.0/bookmarks");
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let (uri, name) = match line.split_once(' ') {
                Some((uri, name)) => (uri, name.to_string()),
                None => (line, String::new()),
            };
            let path = crate::uri::path_from_file_uri(uri)?;
            let label = if name.is_empty() {
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string())
            } else {
                name
            };
            Some((path, label))
        })
        .collect()
}

pub fn face_image() -> Option<PathBuf> {
    let candidates = [
        home_dir().join(".face"),
        home_dir().join(".face.icon"),
        PathBuf::from("/var/lib/AccountsService/icons").join(whoami()),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn whoami() -> String {
    std::env::var("USER").unwrap_or_else(|_| {
        let uid = unsafe { libc::geteuid() };
        users_name(uid).unwrap_or_else(|| uid.to_string())
    })
}

pub fn real_name() -> String {
    let uid = unsafe { libc::geteuid() };
    gecos(uid).or_else(|| users_name(uid)).unwrap_or_else(whoami)
}

fn users_name(uid: u32) -> Option<String> {
    passwd(uid).and_then(|pw| {
        if pw.name.is_empty() {
            None
        } else {
            Some(pw.name)
        }
    })
}

fn gecos(uid: u32) -> Option<String> {
    passwd(uid).and_then(|pw| {
        let name = pw.gecos.split(',').next().unwrap_or("").trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    })
}

struct Passwd {
    name: String,
    gecos: String,
}

fn passwd(uid: u32) -> Option<Passwd> {
    let path = Path::new("/etc/passwd");
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 6 {
            continue;
        }
        if parts[2].parse::<u32>().ok() == Some(uid) {
            return Some(Passwd {
                name: parts[0].to_string(),
                gecos: parts[4].to_string(),
            });
        }
    }
    None
}

pub fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| filename.to_string());
    let ext = path
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();
    for n in 1..10_000 {
        let candidate = dir.join(format!("{stem} ({n}){ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn places_includes_home() {
        let labels: Vec<_> = places().into_iter().map(|(l, _)| l).collect();
        assert!(labels.contains(&"Home".to_string()));
        assert!(labels.contains(&"Computer".to_string()));
        let paths: Vec<_> = places().into_iter().map(|(_, p)| p).collect();
        let mut uniq = paths.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(paths.len(), uniq.len(), "places must not repeat a path");
    }

    #[test]
    fn parse_xbel_orders_by_visited() {
        let xml = r#"<?xml version="1.0"?>
<xbel>
  <bookmark href="file:///tmp/old.txt" visited="2024-01-01T00:00:00Z"/>
  <bookmark href="file:///tmp/new.txt" visited="2024-12-01T00:00:00Z"/>
  <bookmark href="https://example.com/skip" visited="2025-01-01T00:00:00Z"/>
</xbel>
"#;
        let hrefs = parse_recently_used_xbel(xml);
        assert_eq!(
            hrefs,
            vec![
                "file:///tmp/new.txt".to_string(),
                "file:///tmp/old.txt".to_string()
            ]
        );
    }

    #[test]
    fn unique_path_appends_number() {
        let dir = std::env::temp_dir().join(format!("omarchy-portal-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("note.txt"), b"a").unwrap();
        let next = unique_path(&dir, "note.txt");
        assert_eq!(next.file_name().unwrap(), "note (1).txt");
        let _ = fs::remove_dir_all(&dir);
    }
}
