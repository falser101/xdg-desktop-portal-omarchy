use super::PortalCtx;
use crate::dict::{self, Options};
use crate::request::with_request;
use crate::response::PortalResponse;
use std::process::Command;
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
            match spawn_email(&options) {
                Ok(()) => PortalResponse::Success(Empty::default()),
                Err(err) => {
                    tracing::warn!("compose email: {err}");
                    PortalResponse::Other
                }
            }
        })
        .await
    }
}

fn spawn_email(options: &Options) -> anyhow::Result<()> {
    let mut cmd = Command::new("xdg-email");
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
        if let Some(path) = crate::uri::path_from_file_uri(&uri) {
            cmd.arg("--attach").arg(path);
        }
    }
    let mut addrs = dict::as_str_vec(options, "addresses").unwrap_or_default();
    if let Some(one) = dict::as_str(options, "address") {
        addrs.insert(0, one);
    }
    for addr in addrs {
        cmd.arg(addr);
    }
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("xdg-email exited {status}");
    }
}
