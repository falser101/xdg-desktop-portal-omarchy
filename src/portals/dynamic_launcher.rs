use super::PortalCtx;
use crate::dict::{self, Options};
use crate::picker::{self, PickerReply, PickerRequest};
use crate::request::with_request;
use crate::response::{OTHER, SUCCESS, PortalResponse};
use zbus::zvariant::{ObjectPath, OwnedValue, SerializeDict, Type};

pub struct DynamicLauncher(pub PortalCtx);

const ALLOWED_TOKEN_APPS: &[&str] = &[
    "org.gnome.Software",
    "org.gnome.SoftwareDevel",
    "io.elementary.appcenter",
    "org.kde.discover",
];

#[derive(SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct InstallOut {
    name: String,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.DynamicLauncher")]
impl DynamicLauncher {
    async fn prepare_install(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        name: &str,
        _icon: OwnedValue,
        options: Options,
    ) -> PortalResponse<InstallOut> {
        tracing::info!(app_id, name, "DynamicLauncher.PrepareInstall");
        let launcher_type = dict::as_u32(&options, "launcher_type").unwrap_or(1);
        let target = dict::as_str(&options, "target").unwrap_or_default();
        let shown = name.to_string();
        let title = if launcher_type == 2 {
            format!("Install web app “{shown}”?")
        } else {
            format!("Install “{shown}”?")
        };
        let subtitle = if target.is_empty() {
            "This adds a launcher to your applications menu.".to_string()
        } else {
            format!("Opens {target}")
        };
        with_request(&self.0.connection, &handle, |token| async move {
            match picker::run(
                PickerRequest::Confirm {
                    title,
                    subtitle,
                    accept: "Install".into(),
                },
                token,
            )
            .await
            {
                Some(PickerReply::Confirm { accepted: true }) => {
                    PortalResponse::Success(InstallOut { name: shown })
                }
                _ => PortalResponse::Cancelled,
            }
        })
        .await
    }

    async fn request_install_token(&self, app_id: &str, _options: Options) -> u32 {
        tracing::info!(app_id, "DynamicLauncher.RequestInstallToken");
        if ALLOWED_TOKEN_APPS.contains(&app_id) {
            SUCCESS
        } else {
            OTHER
        }
    }

    #[zbus(property, name = "SupportedLauncherTypes")]
    fn supported_launcher_types(&self) -> u32 {
        3
    }

    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }
}
