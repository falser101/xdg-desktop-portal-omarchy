use super::PortalCtx;
use crate::dict::{self, Options};
use crate::documents::{self, Entity};
use crate::filters::FileFilter;
use crate::paths::home_dir;
use crate::request::with_request;
use crate::response::PortalResponse;
use crate::ui::{FileChooserRequest, FileChooserResult, FileMode};
use crate::uri::file_uri;
use std::path::{Path, PathBuf};
use zbus::zvariant::{ObjectPath, SerializeDict, Type};
use zbus::Connection;

pub struct FileChooser(pub PortalCtx);

#[derive(SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct FileChooserOut {
    uris: Vec<String>,
    choices: Vec<(String, String)>,
    current_filter: Option<(String, Vec<(u32, String)>)>,
    writable: bool,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooser {
    async fn open_file(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        title: &str,
        options: Options,
    ) -> PortalResponse<FileChooserOut> {
        tracing::info!(app_id, title, "FileChooser.OpenFile");
        self.run(handle, title, options, FileMode::Open).await
    }

    async fn save_file(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        title: &str,
        options: Options,
    ) -> PortalResponse<FileChooserOut> {
        tracing::info!(app_id, title, "FileChooser.SaveFile");
        self.run(handle, title, options, FileMode::Save).await
    }

    async fn save_files(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        title: &str,
        options: Options,
    ) -> PortalResponse<FileChooserOut> {
        tracing::info!(app_id, title, "FileChooser.SaveFiles");
        self.run(handle, title, options, FileMode::SaveFiles).await
    }

    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }
}

impl FileChooser {
    async fn run(
        &self,
        handle: ObjectPath<'_>,
        title: &str,
        options: Options,
        mode: FileMode,
    ) -> PortalResponse<FileChooserOut> {
        let req = build_request(&self.0.connection, title, options, mode).await;
        with_request(&self.0.connection, &handle, |token| async move {
            match crate::picker::run(crate::picker::PickerRequest::FileChooser(req), token).await {
                Some(crate::picker::PickerReply::FileChooser(result)) => {
                    let out = to_out(result);
                    if out.uris.is_empty() {
                        PortalResponse::Other
                    } else {
                        PortalResponse::Success(out)
                    }
                }
                _ => PortalResponse::Cancelled,
            }
        })
        .await
    }
}

async fn build_request(
    conn: &Connection,
    title: &str,
    options: Options,
    mode: FileMode,
) -> FileChooserRequest {
    let filters_raw = dict::as_filters(&options, "filters");
    let mut filters: Vec<FileFilter> = filters_raw
        .into_iter()
        .map(|(label, pats)| FileFilter::from_portal(label, pats))
        .collect();
    let current = dict::as_filter(&options, "current_filter").map(|(label, pats)| {
        FileFilter::from_portal(label, pats)
    });
    let current_filter = if let Some(current) = current {
        if let Some(i) = filters.iter().position(|f| *f == current) {
            Some(i)
        } else {
            filters.insert(0, current);
            Some(0)
        }
    } else if filters.is_empty() {
        None
    } else {
        Some(0)
    };

    let raw_file = dict::as_path(&options, "current_file");
    let raw_folder = dict::as_path(&options, "current_folder");

    let resolved_file = match raw_file {
        Some(path) if path.is_absolute() => {
            Some(documents::resolve_sandbox_path(conn, &path, Entity::File).await)
        }
        _ => None,
    };

    let folder_hint = match raw_folder {
        Some(path) if path.is_absolute() => {
            Some(documents::resolve_sandbox_path(conn, &path, Entity::Folder).await)
        }
        Some(_) => {
            tracing::debug!("ignoring relative current_folder");
            None
        }
        None => resolved_file.as_deref().and_then(parent_dir),
    };

    let current_folder = documents::existing_dir(folder_hint, home_dir);
    let current_name =
        save_current_name(dict::as_str(&options, "current_name"), resolved_file.as_deref());
    let current_file = resolved_file.filter(|p| p.exists());

    let accept = dict::as_str(&options, "accept_label").unwrap_or_else(|| match mode {
        FileMode::Open => "Open".into(),
        FileMode::Save | FileMode::SaveFiles => "Save".into(),
    });

    FileChooserRequest {
        title: title.to_string(),
        accept_label: accept.replace('_', ""),
        mode,
        multiple: dict::bool_or(&options, "multiple", false),
        directory: dict::bool_or(&options, "directory", false) || mode == FileMode::SaveFiles,
        filters,
        current_filter,
        choices: dict::as_choices(&options, "choices"),
        current_folder,
        current_file,
        current_name,
        save_names: dict::as_files(&options, "files"),
    }
}

fn parent_dir(path: &Path) -> Option<PathBuf> {
    path.parent().map(Path::to_path_buf)
}

fn save_current_name(current_name: Option<String>, current_file: Option<&Path>) -> String {
    if let Some(name) = current_name.filter(|s| !s.is_empty()) {
        return name;
    }
    current_file
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn to_out(result: FileChooserResult) -> FileChooserOut {
    FileChooserOut {
        uris: result.paths.iter().filter_map(|p| file_uri(p)).collect(),
        choices: result.choices,
        current_filter: result.current_filter,
        writable: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_name_prefers_current_name() {
        let name = save_current_name(Some("note.md".into()), Some(Path::new("/tmp/old.txt")));
        assert_eq!(name, "note.md");
    }

    #[test]
    fn save_name_falls_back_to_current_file_basename() {
        let name = save_current_name(None, Some(Path::new("/home/me/draft.txt")));
        assert_eq!(name, "draft.txt");
    }

    #[test]
    fn save_name_ignores_empty_current_name() {
        let name = save_current_name(Some(String::new()), Some(Path::new("/tmp/photo.png")));
        assert_eq!(name, "photo.png");
    }
}
