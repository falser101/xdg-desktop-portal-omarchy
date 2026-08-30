use std::path::{Path, PathBuf};
use url::Url;

pub fn file_uri(path: &Path) -> Option<String> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    Url::from_file_path(abs).ok().map(|u| u.to_string())
}

pub fn path_from_file_uri(uri: &str) -> Option<PathBuf> {
    let url = Url::parse(uri).ok()?;
    if url.scheme() != "file" {
        return None;
    }
    url.to_file_path().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_absolute_path() {
        let uri = file_uri(Path::new("/tmp/example.txt")).unwrap();
        assert!(uri.starts_with("file://"));
        assert_eq!(
            path_from_file_uri(&uri).unwrap(),
            PathBuf::from("/tmp/example.txt")
        );
    }

    #[test]
    fn rejects_non_file_uri() {
        assert!(path_from_file_uri("https://example.com/a").is_none());
    }
}
