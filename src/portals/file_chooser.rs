use super::PortalCtx;
use crate::dict::{self, Options};
use crate::filters::FileFilter;
use crate::paths::home_dir;
use crate::request::with_request;
use crate::response::PortalResponse;
use crate::ui::{FileChooserRequest, FileChooserResult, FileMode};
use crate::uri::file_uri;
use std::path::PathBuf;
use zbus::zvariant::{ObjectPath, SerializeDict, Type};

pub struct FileChooser(pub PortalCtx);

#[derive(SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct FileChooserOut {
    uris: Vec<String>,
    choices: Vec<(String, String)>,
    current_filter: Option<(String, Vec<(u32, String)>)>,
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
}

impl FileChooser {
    async fn run(
        &self,
        handle: ObjectPath<'_>,
        title: &str,
        options: Options,
        mode: FileMode,
    ) -> PortalResponse<FileChooserOut> {
        let req = build_request(title, options, mode);
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

fn build_request(title: &str, options: Options, mode: FileMode) -> FileChooserRequest {
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

    let current_folder = dict::as_path(&options, "current_folder")
        .filter(|p| p.exists())
        .or_else(|| {
            dict::as_path(&options, "current_file").and_then(|p| p.parent().map(PathBuf::from))
        })
        .filter(|p| p.exists())
        .unwrap_or_else(home_dir);

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
        current_name: dict::as_str(&options, "current_name").unwrap_or_default(),
        save_names: dict::as_files(&options, "files"),
    }
}

fn to_out(result: FileChooserResult) -> FileChooserOut {
    FileChooserOut {
        uris: result.paths.iter().filter_map(|p| file_uri(p)).collect(),
        choices: result.choices,
        current_filter: result.current_filter,
    }
}
