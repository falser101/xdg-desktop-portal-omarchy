//! Resolve document-portal sandbox paths (`/run/user/UID/doc/…`) to host paths.
//!
//! Mirrors KDE `kioUrlFromSandboxPath` without KIOFuse: Documents
//! `GetMountPoint` + `Info` only.

use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;
use zbus::Connection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entity {
    File,
    Folder,
}

static MOUNT_POINT: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Resolve a portal-supplied path. Non-document paths are returned unchanged
/// (after NUL-safe handling is done by the caller). Failures fall back to
/// `path` and only log a warning.
pub async fn resolve_sandbox_path(conn: &Connection, path: &Path, entity: Entity) -> PathBuf {
    match resolve_sandbox_path_inner(conn, path, entity).await {
        Ok(Some(resolved)) => resolved,
        Ok(None) => path.to_path_buf(),
        Err(err) => {
            tracing::warn!(
                path = %path.display(),
                ?entity,
                "documents portal resolve failed: {err}"
            );
            path.to_path_buf()
        }
    }
}

async fn resolve_sandbox_path_inner(
    conn: &Connection,
    path: &Path,
    entity: Entity,
) -> anyhow::Result<Option<PathBuf>> {
    let Some(mount) = documents_mount_point(conn).await? else {
        return Ok(None);
    };
    let Some(doc_id) = doc_id_for_path(path, &mount, entity) else {
        return Ok(None);
    };
    let Some(host) = documents_info(conn, &doc_id).await? else {
        return Ok(None);
    };

    let resolved = match entity {
        Entity::Folder => {
            if host.is_dir() {
                host
            } else {
                // File documents (or stale file paths) → browse the parent.
                // `existing_dir` later walks up if that parent is also missing.
                host.parent()
                    .map(Path::to_path_buf)
                    .unwrap_or(host)
            }
        }
        Entity::File => {
            if host.is_dir() {
                if let Some(name) = path.file_name() {
                    host.join(name)
                } else {
                    host
                }
            } else {
                host
            }
        }
    };
    if resolved != path {
        tracing::info!(
            from = %path.display(),
            to = %resolved.display(),
            ?entity,
            "resolved document portal path"
        );
    }
    Ok(Some(resolved))
}

async fn documents_mount_point(conn: &Connection) -> anyhow::Result<Option<PathBuf>> {
    if let Some(cached) = MOUNT_POINT.get() {
        return Ok(cached.clone());
    }
    let proxy = documents_proxy(conn).await?;
    let ay: Vec<u8> = proxy.call("GetMountPoint", &()).await?;
    let mount = path_from_ay(&ay);
    let value = if mount.as_os_str().is_empty() {
        None
    } else {
        Some(mount)
    };
    let _ = MOUNT_POINT.set(value.clone());
    Ok(value)
}

async fn documents_info(conn: &Connection, doc_id: &str) -> anyhow::Result<Option<PathBuf>> {
    let proxy = documents_proxy(conn).await?;
    // Info(s) → (ay, a{sas})
    let (ay, _apps): (Vec<u8>, std::collections::HashMap<String, Vec<String>>) =
        proxy.call("Info", &(doc_id,)).await?;
    let path = path_from_ay(&ay);
    if path.as_os_str().is_empty() {
        Ok(None)
    } else {
        Ok(Some(path))
    }
}

async fn documents_proxy(conn: &Connection) -> anyhow::Result<zbus::Proxy<'static>> {
    Ok(zbus::Proxy::new(
        conn,
        "org.freedesktop.portal.Documents",
        "/org/freedesktop/portal/documents",
        "org.freedesktop.portal.Documents",
    )
    .await?)
}

/// Extract the document id under the fuse mount, if this path is a portal doc path.
pub fn doc_id_for_path(path: &Path, mount: &Path, entity: Entity) -> Option<String> {
    let rel = path.strip_prefix(mount).ok()?;
    let mut comps = rel.components().filter_map(|c| match c {
        Component::Normal(s) => s.to_str(),
        _ => None,
    });
    let id = comps.next()?;
    if id.is_empty() || id == "by-app" {
        return None;
    }
    match entity {
        Entity::Folder => Some(id.to_string()),
        Entity::File => {
            // Prefer paths like mount/DOCID/name; bare mount/DOCID still maps to that doc.
            Some(id.to_string())
        }
    }
}

pub fn path_from_ay(bytes: &[u8]) -> PathBuf {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    PathBuf::from(String::from_utf8_lossy(&bytes[..end]).as_ref())
}

/// Prefer an absolute existing directory; walk parents if needed; else `fallback`.
pub fn existing_dir(path: Option<PathBuf>, fallback: impl FnOnce() -> PathBuf) -> PathBuf {
    let Some(path) = path.filter(|p| p.is_absolute()) else {
        return fallback();
    };
    if path.is_dir() {
        return path;
    }
    let mut cur = path.as_path();
    while let Some(parent) = cur.parent() {
        if parent.as_os_str().is_empty() {
            break;
        }
        if parent.is_dir() {
            return parent.to_path_buf();
        }
        cur = parent;
    }
    fallback()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_from_ay_strips_nul() {
        assert_eq!(
            path_from_ay(b"/run/user/1000/doc\0"),
            PathBuf::from("/run/user/1000/doc")
        );
    }

    #[test]
    fn doc_id_folder_and_file() {
        let mount = Path::new("/run/user/1000/doc");
        assert_eq!(
            doc_id_for_path(Path::new("/run/user/1000/doc/abc123"), mount, Entity::Folder)
                .as_deref(),
            Some("abc123")
        );
        assert_eq!(
            doc_id_for_path(
                Path::new("/run/user/1000/doc/abc123/photo.png"),
                mount,
                Entity::File
            )
            .as_deref(),
            Some("abc123")
        );
        assert_eq!(
            doc_id_for_path(Path::new("/run/user/1000/doc/by-app/foo"), mount, Entity::Folder),
            None
        );
        assert_eq!(
            doc_id_for_path(Path::new("/home/me/Downloads"), mount, Entity::Folder),
            None
        );
    }

    #[test]
    fn existing_dir_walks_parents() {
        let missing = PathBuf::from("/tmp/omarchy-portal-missing-dir-xyz/nested");
        let got = existing_dir(Some(missing), || PathBuf::from("/should-not"));
        assert_eq!(got, PathBuf::from("/tmp"));
    }

    #[test]
    fn existing_dir_rejects_relative() {
        let got = existing_dir(Some(PathBuf::from("relative/path")), || {
            PathBuf::from("/fallback")
        });
        assert_eq!(got, PathBuf::from("/fallback"));
    }
}

#[cfg(test)]
mod live {
    use super::*;

    #[tokio::test]
    #[ignore = "needs session Documents portal"]
    async fn resolves_real_doc_path() {
        let conn = Connection::session().await.expect("session bus");
        let mount = documents_mount_point(&conn)
            .await
            .expect("mount")
            .expect("mount point");
        let proxy = documents_proxy(&conn).await.expect("proxy");
        let docs: std::collections::HashMap<String, Vec<u8>> =
            proxy.call("List", &("".to_string(),)).await.expect("List");
        let (doc_id, host_ay) = docs.into_iter().next().expect("at least one doc");
        let host = path_from_ay(&host_ay);
        let sandboxed = mount.join(&doc_id);
        let resolved = resolve_sandbox_path(&conn, &sandboxed, Entity::Folder).await;
        assert_ne!(resolved, sandboxed, "should leave the fuse path");
        let expected = if host.is_dir() {
            host
        } else {
            host.parent().unwrap().to_path_buf()
        };
        assert_eq!(resolved, expected);
    }
}
