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
    let mut out = vec![
        ("Home".into(), home_dir()),
        ("Downloads".into(), user_dir("DOWNLOAD", "Downloads")),
        ("Documents".into(), user_dir("DOCUMENTS", "Documents")),
        ("Pictures".into(), user_dir("PICTURES", "Pictures")),
        ("Videos".into(), user_dir("VIDEOS", "Videos")),
        ("Music".into(), user_dir("MUSIC", "Music")),
        ("Projects".into(), user_dir("PROJECTS", "Projects")),
        ("Computer".into(), PathBuf::from("/")),
    ];
    for (path, label) in gtk_bookmarks() {
        if !out.iter().any(|(_, p)| *p == path) {
            out.push((label, path));
        }
    }
    out
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
