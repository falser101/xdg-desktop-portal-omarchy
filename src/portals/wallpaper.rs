use super::PortalCtx;
use crate::dict::{self, Options};
use crate::request::with_request;
use crate::response::{CANCELLED, OTHER, SUCCESS};
use crate::uri::path_from_file_uri;
use std::process::Command;
use zbus::zvariant::ObjectPath;

pub struct Wallpaper(pub PortalCtx);

#[zbus::interface(name = "org.freedesktop.impl.portal.Wallpaper")]
impl Wallpaper {
    #[zbus(name = "SetWallpaperURI")]
    async fn set_wallpaper_uri(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        uri: &str,
        options: Options,
    ) -> u32 {
        tracing::info!(app_id, uri, "Wallpaper.SetWallpaperURI");
        let uri = uri.to_string();
        let show = dict::bool_or(&options, "show-preview", true);
        let result = with_request(&self.0.connection, &handle, |token| async move {
            let path = match path_from_file_uri(&uri) {
                Some(p) if p.is_file() => p,
                _ => return crate::response::PortalResponse::Other,
            };
            if show {
                let uri_show = uri.clone();
                let ok = matches!(
                    crate::picker::run(
                        crate::picker::PickerRequest::Wallpaper { uri: uri_show },
                        token
                    )
                    .await,
                    Some(crate::picker::PickerReply::Wallpaper { granted: true })
                );
                if !ok {
                    return crate::response::PortalResponse::Cancelled;
                }
            }
            match Command::new("omarchy-theme-bg-set").arg(&path).status() {
                Ok(s) if s.success() => crate::response::PortalResponse::Success(()),
                Ok(_) | Err(_) => crate::response::PortalResponse::Other,
            }
        })
        .await;
        match result {
            crate::response::PortalResponse::Success(()) => SUCCESS,
            crate::response::PortalResponse::Cancelled => CANCELLED,
            crate::response::PortalResponse::Other => OTHER,
        }
    }
}
