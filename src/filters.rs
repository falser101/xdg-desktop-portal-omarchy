use glob::Pattern;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileFilter {
    pub label: String,
    pub patterns: Vec<FilterPattern>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FilterPattern {
    Glob(String),
    Mime(String),
}

impl FileFilter {
    pub fn from_portal(label: String, patterns: Vec<(u32, String)>) -> Self {
        let patterns = patterns
            .into_iter()
            .filter_map(|(kind, value)| match kind {
                0 => Some(FilterPattern::Glob(value)),
                1 => Some(FilterPattern::Mime(value)),
                _ => None,
            })
            .collect();
        Self { label, patterns }
    }

    pub fn matches(&self, path: &Path) -> bool {
        if self.patterns.is_empty() {
            return true;
        }
        self.patterns.iter().any(|p| p.matches(path))
    }

    pub fn to_portal(&self) -> (String, Vec<(u32, String)>) {
        let patterns = self
            .patterns
            .iter()
            .map(|p| match p {
                FilterPattern::Glob(v) => (0u32, v.clone()),
                FilterPattern::Mime(v) => (1u32, v.clone()),
            })
            .collect();
        (self.label.clone(), patterns)
    }

    /// Globs `FolderListModel.nameFilters` can apply. MIME types are expanded
    /// to extensions so the QML picker can filter without guessing.
    pub fn globs(&self) -> Vec<String> {
        let mut out = Vec::new();
        for pattern in &self.patterns {
            match pattern {
                FilterPattern::Glob(glob) if !glob.is_empty() => out.push(glob.clone()),
                FilterPattern::Mime(mime) => out.extend(globs_for_mime(mime)),
                FilterPattern::Glob(_) => {}
            }
        }
        out.sort();
        out.dedup();
        if out.is_empty() {
            vec!["*".into()]
        } else {
            out
        }
    }
}

impl FilterPattern {
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Self::Glob(pat) => glob_matches(pat, path),
            Self::Mime(mime) => mime_matches(mime, path),
        }
    }
}

fn glob_matches(pat: &str, path: &Path) -> bool {
    if pat == "*" || pat == "*.*" {
        return true;
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Pattern::new(pat)
        .map(|p| p.matches(&name))
        .unwrap_or(false)
}

fn globs_for_mime(mime: &str) -> Vec<String> {
    if mime.is_empty() || mime == "*/*" {
        return vec!["*".into()];
    }
    if let Some(top) = mime.strip_suffix("/*") {
        return wildcard_mime_globs(top);
    }
    mime_guess::get_mime_extensions_str(mime)
        .map(|exts| exts.iter().map(|e| format!("*.{e}")).collect())
        .unwrap_or_default()
}

fn wildcard_mime_globs(top: &str) -> Vec<String> {
    let exts: &[&str] = match top {
        "image" => &[
            "png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "tif", "tiff", "avif", "jxl",
            "heic", "ico",
        ],
        "audio" => &["mp3", "wav", "flac", "ogg", "m4a", "aac", "opus", "wma", "oga"],
        "video" => &["mp4", "mkv", "webm", "avi", "mov", "m4v", "wmv", "ogv"],
        "text" => &[
            "txt", "md", "csv", "json", "xml", "html", "htm", "css", "js", "rs", "py", "toml",
            "ini", "log",
        ],
        _ => return vec!["*".into()],
    };
    exts.iter().map(|e| format!("*.{e}")).collect()
}

fn mime_matches(mime: &str, path: &Path) -> bool {
    if mime == "*/*" {
        return true;
    }
    let guessed = mime_guess::from_path(path)
        .first()
        .map(|m| m.essence_str().to_string())
        .unwrap_or_else(|| "application/octet-stream".into());
    if let Some(prefix) = mime.strip_suffix("/*") {
        return guessed.starts_with(&format!("{prefix}/"));
    }
    guessed.eq_ignore_ascii_case(mime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_png() {
        let f = FileFilter::from_portal("PNG".into(), vec![(0, "*.png".into())]);
        assert!(f.matches(Path::new("photo.png")));
        assert!(!f.matches(Path::new("photo.jpg")));
    }

    #[test]
    fn mime_image_wildcard() {
        let f = FileFilter::from_portal("Images".into(), vec![(1, "image/*".into())]);
        assert!(f.matches(Path::new("a.png")));
        assert!(f.matches(Path::new("b.jpeg")));
        assert!(!f.matches(Path::new("c.txt")));
    }

    #[test]
    fn empty_filter_matches_all() {
        let f = FileFilter::from_portal("All".into(), vec![]);
        assert!(f.matches(Path::new("anything.bin")));
    }

    #[test]
    fn mime_globs_expand_png() {
        let f = FileFilter::from_portal("PNG".into(), vec![(1, "image/png".into())]);
        let globs = f.globs();
        assert!(globs.iter().any(|g| g == "*.png"));
    }

    #[test]
    fn image_wildcard_globs() {
        let f = FileFilter::from_portal("Images".into(), vec![(1, "image/*".into())]);
        let globs = f.globs();
        assert!(globs.contains(&"*.png".into()));
        assert!(globs.contains(&"*.jpg".into()));
    }
}
