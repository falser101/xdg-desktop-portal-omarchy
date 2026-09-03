use super::PortalCtx;
use crate::dict::{self, Options};
use crate::request::with_request;
use crate::response::PortalResponse;
use zbus::zvariant::{ObjectPath, SerializeDict, Type};

pub struct Email(pub PortalCtx);

#[derive(Default, SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct Empty {}

#[zbus::interface(name = "org.freedesktop.impl.portal.Email")]
impl Email {
    async fn compose_email(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        options: Options,
    ) -> PortalResponse<Empty> {
        tracing::info!(app_id, "Email.ComposeEmail");
        with_request(&self.0.connection, &handle, |_token| async move {
            match spawn_email(&options).await {
                Ok(()) => PortalResponse::Success(Empty::default()),
                Err(err) => {
                    tracing::warn!("compose email: {err}");
                    PortalResponse::Other
                }
            }
        })
        .await
    }

    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        4
    }
}

async fn spawn_email(options: &Options) -> anyhow::Result<()> {
    let mut cmd = tokio::process::Command::new("xdg-email");
    cmd.arg("--utf8");

    if let Some(subject) = dict::as_str(options, "subject") {
        cmd.arg("--subject").arg(subject);
    }
    if let Some(body) = dict::as_str(options, "body") {
        cmd.arg("--body").arg(body);
    }
    for addr in dict::as_str_vec(options, "cc").unwrap_or_default() {
        cmd.arg("--cc").arg(addr);
    }
    for addr in dict::as_str_vec(options, "bcc").unwrap_or_default() {
        cmd.arg("--bcc").arg(addr);
    }
    for uri in dict::as_str_vec(options, "attachments").unwrap_or_default() {
        match attachment_path(&uri) {
            Some(path) => {
                cmd.arg("--attach").arg(path);
            }
            None => {
                tracing::warn!(%uri, "skipping attachment: not a usable file:// URI or local path");
            }
        }
    }

    let mut addrs = dict::as_str_vec(options, "addresses").unwrap_or_default();
    if let Some(one) = dict::as_str(options, "address") {
        addrs.insert(0, one);
    }
    for addr in addrs {
        cmd.arg(addr);
    }

    // Wayland xdg-activation (and X11 startup-notification) so the mail client
    // can take focus. Frontend Email v4 passes this through as activation_token.
    if let Some(token) = dict::as_str(options, "activation_token") {
        if !token.is_empty() {
            cmd.env("XDG_ACTIVATION_TOKEN", &token);
            cmd.env("DESKTOP_STARTUP_ID", &token);
        }
    }

    let status = cmd.status().await?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("xdg-email exited {status}");
    }
}

/// Frontend usually sends `file://` URIs; some builds pass bare absolute paths.
fn attachment_path(uri: &str) -> Option<std::path::PathBuf> {
    if let Some(path) = crate::uri::path_from_file_uri(uri) {
        return Some(path);
    }
    let path = std::path::PathBuf::from(uri);
    if path.is_absolute() {
        Some(path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::attachment_path;
    use std::path::PathBuf;

    #[test]
    fn accepts_file_uri_and_absolute_path() {
        assert_eq!(
            attachment_path("file:///tmp/a.txt").unwrap(),
            PathBuf::from("/tmp/a.txt")
        );
        assert_eq!(
            attachment_path("/tmp/a.txt").unwrap(),
            PathBuf::from("/tmp/a.txt")
        );
        assert!(attachment_path("https://example.com/a").is_none());
        assert!(attachment_path("relative.txt").is_none());
    }
}
