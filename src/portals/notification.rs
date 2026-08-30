use crate::dict::{self, Options};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedValue, Value};

pub struct Notification {
    titles: Mutex<HashMap<(String, String), String>>,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            titles: Mutex::new(HashMap::new()),
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Notification")]
impl Notification {
    async fn add_notification(&self, app_id: &str, id: &str, notification: Options) {
        let title = dict::as_str(&notification, "title").unwrap_or_else(|| app_id.to_string());
        let body = dict::as_str(&notification, "body")
            .or_else(|| dict::as_str(&notification, "markup-body"))
            .unwrap_or_default();
        let priority = dict::as_str(&notification, "priority").unwrap_or_else(|| "normal".into());
        let urgency = match priority.as_str() {
            "high" | "urgent" => "critical",
            "low" => "low",
            _ => "normal",
        };
        let icon = notification
            .get("icon")
            .and_then(icon_name)
            .unwrap_or_default();

        self.titles
            .lock()
            .unwrap()
            .insert((app_id.to_string(), id.to_string()), title.clone());

        let mut cmd = Command::new("omarchy-notification-send");
        cmd.arg("--app-name").arg(app_id).arg("-u").arg(urgency);
        if !icon.is_empty() {
            cmd.arg("-i").arg(&icon);
        }
        cmd.arg(&title);
        if !body.is_empty() {
            cmd.arg(&body);
        }
        if let Err(err) = cmd.status() {
            tracing::warn!("omarchy-notification-send: {err}");
        }
    }

    async fn remove_notification(&self, app_id: &str, id: &str) {
        let title = self
            .titles
            .lock()
            .unwrap()
            .remove(&(app_id.to_string(), id.to_string()));
        if let Some(title) = title {
            let _ = Command::new("omarchy-notification-dismiss")
                .arg(&title)
                .status();
        }
    }

    #[zbus(property, name = "SupportedOptions")]
    fn supported_options(&self) -> HashMap<String, OwnedValue> {
        HashMap::new()
    }

    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        2
    }

    #[zbus(signal)]
    async fn action_invoked(
        emitter: &SignalEmitter<'_>,
        app_id: &str,
        id: &str,
        action: &str,
        parameter: Vec<Value<'_>>,
    ) -> zbus::Result<()>;
}

fn icon_name(value: &OwnedValue) -> Option<String> {
    let cloned = value.try_clone().ok()?;
    String::try_from(cloned).ok()
}
