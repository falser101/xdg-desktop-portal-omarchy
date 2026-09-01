use super::PortalCtx;
use crate::dict::{self, Options};
use crate::request::with_request;
use crate::response::PortalResponse;
use crate::ui::AccountRequest;
use crate::uri::file_uri;
use zbus::zvariant::{ObjectPath, SerializeDict, Type};

pub struct Account(pub PortalCtx);

#[derive(SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct AccountOut {
    id: String,
    name: String,
    image: String,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Account")]
impl Account {
    async fn get_user_information(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _window: &str,
        options: Options,
    ) -> PortalResponse<AccountOut> {
        tracing::info!(app_id, "Account.GetUserInformation");
        let reason = dict::as_str(&options, "reason").unwrap_or_default();
        let app_name = app_display_name(app_id);
        let (username, real_name, image) = crate::paths::account_identity().await;
        let title = format!("Share user info with {app_name}?");
        let subtitle = if reason.is_empty() {
            "The application will be able to see your username, full name, and profile picture.\n\nThe application did not provide a reason.".into()
        } else {
            format!(
                "The application will be able to see your username, full name, and profile picture.\n\nReason: “{reason}”"
            )
        };
        let image_path = image.to_string_lossy().into_owned();
        let req = AccountRequest {
            title,
            subtitle,
            username: username.clone(),
            real_name: real_name.clone(),
            image: Some(image_path),
        };
        with_request(&self.0.connection, &handle, |token| async move {
            match crate::picker::run(crate::picker::PickerRequest::Account(req), token).await {
                Some(crate::picker::PickerReply::Account(info)) => {
                    let image = info
                        .image
                        .as_deref()
                        .and_then(file_uri)
                        .unwrap_or_else(|| "file://".into());
                    PortalResponse::Success(AccountOut {
                        id: info.id,
                        name: info.name,
                        image,
                    })
                }
                _ => PortalResponse::Cancelled,
            }
        })
        .await
    }
}

fn app_display_name(app_id: &str) -> String {
    let id = app_id.trim();
    if id.is_empty() {
        return "this application".into();
    }
    crate::desktop::load_app(id)
        .map(|app| app.name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| id.to_string())
}
