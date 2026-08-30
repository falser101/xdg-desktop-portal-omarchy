use super::PortalCtx;
use crate::picker::{self, PickerReply, PickerRequest};
use crate::request::with_request;
use crate::response::PortalResponse;
use std::collections::HashMap;
use std::io::Write;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedValue, SerializeDict, Type};

pub struct Background(pub PortalCtx);

#[derive(SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct NotifyOut {
    result: u32,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Background")]
impl Background {
    async fn get_app_state(&self) -> HashMap<String, OwnedValue> {
        app_states()
    }

    async fn notify_background(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        name: &str,
    ) -> PortalResponse<NotifyOut> {
        tracing::info!(app_id, name, "Background.NotifyBackground");
        let title = format!("Allow {name} to run in the background?");
        with_request(&self.0.connection, &handle, |token| async move {
            match picker::run(
                PickerRequest::Confirm {
                    title,
                    subtitle: "The app asked to keep running after you close its window.".into(),
                    accept: "Allow".into(),
                },
                token,
            )
            .await
            {
                Some(PickerReply::Confirm { accepted: true }) => {
                    PortalResponse::Success(NotifyOut { result: 1 })
                }
                _ => PortalResponse::Success(NotifyOut { result: 0 }),
            }
        })
        .await
    }

    async fn enable_autostart(
        &self,
        app_id: &str,
        enable: bool,
        commandline: Vec<String>,
        _flags: u32,
    ) -> bool {
        tracing::info!(app_id, enable, "Background.EnableAutostart");
        write_autostart(app_id, enable, &commandline)
    }

    #[zbus(signal)]
    async fn running_applications_changed(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        2
    }
}

fn app_states() -> HashMap<String, OwnedValue> {
    let Ok(out) = std::process::Command::new("hyprctl")
        .args(["-j", "clients"])
        .output()
    else {
        return HashMap::new();
    };
    let Ok(clients) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) else {
        return HashMap::new();
    };
    let active = std::process::Command::new("hyprctl")
        .args(["-j", "activewindow"])
        .output()
        .ok()
        .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok())
        .and_then(|v| v.get("class")?.as_str().map(str::to_string));

    let mut map: HashMap<String, u32> = HashMap::new();
    for client in clients {
        let class = client
            .get("class")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if class.is_empty() {
            continue;
        }
        if client.get("mapped").and_then(|v| v.as_bool()) == Some(false) {
            continue;
        }
        let state: u32 = if active.as_deref() == Some(class) {
            2
        } else {
            1
        };
        map.entry(class.to_string())
            .and_modify(|cur| *cur = (*cur).max(state))
            .or_insert(state);
    }
    map.into_iter()
        .filter_map(|(k, v)| OwnedValue::try_from(v).ok().map(|ov| (k, ov)))
        .collect()
}

fn write_autostart(app_id: &str, enable: bool, commandline: &[String]) -> bool {
    let dir = crate::paths::config_home().join("autostart");
    let _ = std::fs::create_dir_all(&dir);
    let safe: String = app_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let path = dir.join(format!("{safe}.desktop"));
    if !enable {
        let _ = std::fs::remove_file(&path);
        return true;
    }
    if commandline.is_empty() {
        return false;
    }
    let exec = commandline
        .iter()
        .map(|a| shell_quote(a))
        .collect::<Vec<_>>()
        .join(" ");
    let body = format!(
        "[Desktop Entry]\nType=Application\nName={app_id}\nExec={exec}\nX-GNOME-Autostart-enabled=true\n"
    );
    match std::fs::File::create(&path) {
        Ok(mut f) => f.write_all(body.as_bytes()).is_ok(),
        Err(_) => false,
    }
}

fn shell_quote(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_./~".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_spaces() {
        assert_eq!(shell_quote("ok"), "ok");
        assert_eq!(shell_quote("a b"), "'a b'");
    }
}
