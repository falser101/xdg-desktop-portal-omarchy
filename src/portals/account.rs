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
        let req = AccountRequest {
            title: "Share account information".into(),
            reason: dict::as_str(&options, "reason").unwrap_or_default(),
        };
        with_request(&self.0.connection, &handle, |token| async move {
            match crate::picker::run(crate::picker::PickerRequest::Account(req), token).await {
                Some(crate::picker::PickerReply::Account(info)) => {
                    PortalResponse::Success(AccountOut {
                        id: info.id,
                        name: info.name,
                        image: info
                            .image
                            .as_deref()
                            .and_then(file_uri)
                            .unwrap_or_default(),
                    })
                }
                _ => PortalResponse::Cancelled,
            }
        })
        .await
    }
}
