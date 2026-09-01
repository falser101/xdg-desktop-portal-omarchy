use super::PortalCtx;
use crate::dict::{self, Options};
use crate::picker::{self, PickerReply, PickerRequest};
use crate::request::with_request;
use crate::response::{OTHER, SUCCESS, PortalResponse};
use std::io::Write;
use std::path::PathBuf;
use zbus::zvariant::{ObjectPath, OwnedValue, SerializeDict, Structure, Type, Value};

pub struct DynamicLauncher(pub PortalCtx);

// Common software-center app ids that may request a launcher token.
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
    /// Frontend requires type `v` (nested variant) containing the GBytesIcon `(sv)`.
    icon: Value<'static>,
}

fn icon_preview_path(icon: &OwnedValue) -> Option<PathBuf> {
    let mut value: Value<'static> = icon.try_clone().ok()?.into();
    while let Value::Value(inner) = value {
        value = *inner;
    }
    let structure = Structure::try_from(value).ok()?;
    let fields: Vec<Value<'static>> = structure.into_fields();
    if fields.len() < 2 {
        return None;
    }
    let kind = String::try_from(fields[0].try_clone().ok()?).ok()?;
    if kind != "bytes" {
        return None;
    }
    let bytes = Vec::<u8>::try_from(fields[1].try_clone().ok()?).ok()?;
    if bytes.is_empty() {
        return None;
    }
    let path = std::env::temp_dir().join(format!(
        "omarchy-portal-dl-{}-{}.png",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos()
    ));
    let mut file = std::fs::File::create(&path).ok()?;
    file.write_all(&bytes).ok()?;
    Some(path)
}

#[zbus::interface(name = "org.freedesktop.impl.portal.DynamicLauncher")]
impl DynamicLauncher {
    async fn prepare_install(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        name: &str,
        icon: OwnedValue,
        options: Options,
    ) -> PortalResponse<InstallOut> {
        tracing::info!(app_id, name, "DynamicLauncher.PrepareInstall");
        let launcher_type = dict::as_u32(&options, "launcher_type").unwrap_or(1);
        let target = dict::as_str(&options, "target").unwrap_or_default();
        let editable_name = dict::as_bool(&options, "editable_name").unwrap_or(true);
        let default_name = name.to_string();
        let preview = icon_preview_path(&icon);
        // Echo the caller icon back; icon editing is not supported yet.
        let icon_out = Value::Value(Box::new(Value::from(icon)));

        let main_text = if launcher_type == 2 {
            "Create Web Application Launcher?".to_string()
        } else {
            "Create Application Launcher?".to_string()
        };
        let subtitle = if launcher_type == 2 {
            "A launcher to open the following website will appear in application launchers:"
                .to_string()
        } else {
            "A launcher to open the following application will appear in application launchers:"
                .to_string()
        };

        with_request(&self.0.connection, &handle, |token| async move {
            let reply = picker::run(
                PickerRequest::DynamicLauncher {
                    main_text,
                    subtitle,
                    name: default_name.clone(),
                    icon_path: preview
                        .as_ref()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                    target: target.clone(),
                    editable_name,
                },
                token,
            )
            .await;

            if let Some(path) = preview.as_ref() {
                let _ = std::fs::remove_file(path);
            }

            match reply {
                Some(PickerReply::DynamicLauncher { accepted: true, name }) => {
                    let name = if name.trim().is_empty() {
                        default_name
                    } else {
                        name
                    };
                    PortalResponse::Success(InstallOut {
                        name,
                        icon: icon_out,
                    })
                }
                // Stale Confirm UI (shell not reloaded yet).
                Some(PickerReply::Confirm { accepted: true }) => PortalResponse::Success(InstallOut {
                    name: default_name,
                    icon: icon_out,
                }),
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
