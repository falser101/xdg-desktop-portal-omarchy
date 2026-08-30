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
        let req = AppChooserRequest {
            title: "Open with".into(),
            choices,
            last_choice: dict::as_str(&options, "last_choice"),
            content_type: dict::as_str(&options, "content_type"),
            uri: dict::as_str(&options, "uri"),
            filename: dict::as_str(&options, "filename"),
        };
        with_request(&self.0.connection, &handle, |token| async move {
            match crate::picker::run(crate::picker::PickerRequest::AppChooser(req), token).await {
                Some(crate::picker::PickerReply::App { choice }) => {
                    let choice = crate::desktop::portal_app_id(&choice);
                    tracing::info!(%choice, "AppChooser selected");
                    PortalResponse::Success(AppChooserOut { choice })
                }
                _ => PortalResponse::Cancelled,
            }
        })
        .await
    }

    async fn update_choices(&self, _handle: ObjectPath<'_>, _choices: Vec<String>) {}
}
