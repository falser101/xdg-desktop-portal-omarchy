use super::PortalCtx;
use crate::dict::{self, Options};
use crate::request::with_request;
use crate::response::PortalResponse;
use crate::ui::AppChooserRequest;
use zbus::zvariant::{ObjectPath, SerializeDict, Type};

pub struct AppChooser(pub PortalCtx);

#[derive(SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct AppChooserOut {
    choice: String,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.AppChooser")]
impl AppChooser {
    async fn choose_application(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        choices: Vec<String>,
        options: Options,
    ) -> PortalResponse<AppChooserOut> {
        tracing::info!(app_id, "AppChooser.ChooseApplication");
        let content_type = dict::as_str(&options, "content_type");
        let req = AppChooserRequest {
            title: "Open with".into(),
            choices,
            last_choice: dict::as_str(&options, "last_choice"),
            content_type: content_type.clone(),
            uri: dict::as_str(&options, "uri"),
            filename: dict::as_str(&options, "filename"),
        };
        with_request(&self.0.connection, &handle, |token| async move {
            match crate::picker::run(crate::picker::PickerRequest::AppChooser(req), token).await {
                Some(crate::picker::PickerReply::App { choice, remember }) => {
                    let choice = crate::desktop::portal_app_id(&choice);
                    if remember {
                        if let Some(mime) = content_type.as_deref().filter(|s| !s.is_empty()) {
                            if let Err(err) = crate::desktop::set_default_for_mime(mime, &choice) {
                                tracing::warn!(%mime, %choice, %err, "failed to write mimeapps.list");
                            } else {
                                tracing::info!(%mime, %choice, "set default handler in mimeapps.list");
                            }
                        } else {
                            tracing::warn!("remember checked but content_type empty; skip mimeapps");
                        }
                    }
                    tracing::info!(%choice, remember, "AppChooser selected");
                    PortalResponse::Success(AppChooserOut { choice })
                }
                _ => PortalResponse::Cancelled,
            }
        })
        .await
    }

    async fn update_choices(&self, _handle: ObjectPath<'_>, _choices: Vec<String>) {}
}
