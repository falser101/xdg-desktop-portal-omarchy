use super::PortalCtx;
use crate::dict::{self, Options};
use crate::request::with_request;
use crate::response::PortalResponse;
use crate::ui::AccessRequest;
use zbus::zvariant::{ObjectPath, SerializeDict, Type};

pub struct Access(pub PortalCtx);

#[derive(SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct AccessOut {
    choices: Vec<(String, String)>,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Access")]
impl Access {
    async fn access_dialog(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        title: &str,
        subtitle: &str,
        body: &str,
        options: Options,
    ) -> PortalResponse<AccessOut> {
        tracing::info!(app_id, title, "Access.AccessDialog");
        let req = AccessRequest {
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            body: body.to_string(),
            deny_label: dict::as_str(&options, "deny_label").unwrap_or_else(|| "Deny".into()),
            grant_label: dict::as_str(&options, "grant_label").unwrap_or_else(|| "Allow".into()),
            choices: dict::as_choices(&options, "choices"),
        };
        with_request(&self.0.connection, &handle, |token| async move {
            match crate::picker::run(crate::picker::PickerRequest::Access(req), token).await {
                Some(crate::picker::PickerReply::Access(result)) if result.granted => {
                    PortalResponse::Success(AccessOut {
                        choices: result.choices,
                    })
                }
                _ => PortalResponse::Cancelled,
            }
        })
        .await
    }
}
