use super::PortalCtx;
use crate::picker::{self, PickerReply, PickerRequest};
use crate::request::with_request;
use crate::response::PortalResponse;
use crate::DBUS_PATH;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedValue, SerializeDict, Type};
use zbus::Connection;

const FORBID: u32 = 0;
const ALLOW: u32 = 1;
const ALLOW_ONCE: u32 = 2;

pub struct Background {
    ctx: PortalCtx,
    /// app_ids that already got a Background prompt this session (dedupe → Allow once).
    warned: Arc<Mutex<HashSet<String>>>,
}

impl Background {
    pub fn new(ctx: PortalCtx) -> Self {
        let this = Self {
            ctx: ctx.clone(),
            warned: Arc::new(Mutex::new(HashSet::new())),
        };
        this.spawn_watch();
        this
    }

    fn spawn_watch(&self) {
        let conn = self.ctx.connection.clone();
        tokio::spawn(watch_running_apps(conn));
    }
}

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

        // Only suppress while a prompt for this app is still open; after
        // it closes the app can be asked again. Keeping the id forever made
        // every subsequent test return Allow once with no dialog.
        if !app_id.is_empty() {
            let mut warned = self.warned.lock().unwrap();
            if warned.contains(app_id) {
                return PortalResponse::Success(NotifyOut {
                    result: ALLOW_ONCE,
                });
            }
            warned.insert(app_id.to_string());
        }

        let app_name = if name.is_empty() {
            if app_id.is_empty() {
                "This application".into()
            } else {
                crate::desktop::load_app(app_id)
                    .map(|a| a.name)
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| app_id.to_string())
            }
        } else {
            name.to_string()
        };

        let title = "Background Activity".into();
        let subtitle = format!("{app_name} wants to remain running when it has no visible windows.");
        let body = "If you deny this request, the application will quit when its last window is closed.".into();

        let warned = Arc::clone(&self.warned);
        let app_id_owned = app_id.to_string();
        with_request(&self.ctx.connection, &handle, |token| async move {
            let result = match picker::run(
                PickerRequest::Background {
                    title,
                    subtitle,
                    body,
                },
                token,
            )
            .await
            {
                Some(PickerReply::Background { result }) => result,
                // Stale Confirm UI (shell not reloaded): map Allow/Cancel sanely.
                Some(PickerReply::Confirm { accepted: true }) => ALLOW,
                Some(PickerReply::Confirm { accepted: false }) => FORBID,
                Some(PickerReply::Access(r)) if r.granted => ALLOW,
                Some(PickerReply::Access(_)) => FORBID,
                // Close without choosing → Allow once.
                Some(PickerReply::Cancel) | None => ALLOW_ONCE,
                other => {
                    tracing::warn!(?other, "Background: unexpected picker reply; Allow once");
                    ALLOW_ONCE
                }
            };
            if !app_id_owned.is_empty() {
                warned.lock().unwrap().remove(&app_id_owned);
            }
            tracing::info!(
                result,
                meaning = match result {
                    FORBID => "Forbid",
                    ALLOW => "Allow",
                    ALLOW_ONCE => "Allow once",
                    _ => "unknown",
                },
                "Background.NotifyBackground decided"
            );
            PortalResponse::Success(NotifyOut { result })
        })
        .await
    }

    async fn enable_autostart(
        &self,
        app_id: &str,
        enable: bool,
        commandline: Vec<String>,
        flags: u32,
    ) -> bool {
        tracing::info!(app_id, enable, flags, "Background.EnableAutostart");
        write_autostart(app_id, enable, &commandline, flags)
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
        // Spec: Background=0, Running=1, Active=2
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

fn write_autostart(app_id: &str, enable: bool, commandline: &[String], flags: u32) -> bool {
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
        // Spec: return whether the app will be autostarted afterwards.
        return false;
    }
    if commandline.is_empty() {
        return false;
    }
    let exec = commandline
        .iter()
        .map(|a| shell_quote(a))
        .collect::<Vec<_>>()
        .join(" ");
    let activatable = flags & 1 != 0;
    let mut body = format!(
        "[Desktop Entry]\nType=Application\nName={app_id}\nExec={exec}\nX-GNOME-Autostart-enabled=true\n"
    );
    if activatable {
        body.push_str("DBusActivatable=true\n");
    }
    // Stamp Flatpak id for portal-requested autostart entries.
    body.push_str(&format!("X-Flatpak={app_id}\n"));
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

fn socket2_path() -> Option<std::path::PathBuf> {
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".into());
    Some(
        std::path::PathBuf::from(runtime)
            .join("hypr")
            .join(sig)
            .join(".socket2.sock"),
    )
}

async fn watch_running_apps(conn: Connection) {
    loop {
        let Some(path) = socket2_path() else {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        };
        match tokio::net::UnixStream::connect(&path).await {
            Ok(stream) => {
                tracing::info!("Background: watching Hyprland socket2 for app state");
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            let ev = line.trim();
                            if ev.starts_with("openwindow>>")
                                || ev.starts_with("closewindow>>")
                                || ev.starts_with("activewindow>>")
                                || ev.starts_with("activewindowv2>>")
                                || ev.starts_with("urgent>>")
                            {
                                emit_running_changed(&conn).await;
                            }
                        }
                    }
                }
            }
            Err(_) => tokio::time::sleep(std::time::Duration::from_secs(2)).await,
        }
    }
}

async fn emit_running_changed(conn: &Connection) {
    let Ok(iface) = conn
        .object_server()
        .interface::<_, Background>(DBUS_PATH)
        .await
    else {
        return;
    };
    let _ = Background::running_applications_changed(iface.signal_emitter()).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_spaces() {
        assert_eq!(shell_quote("ok"), "ok");
        assert_eq!(shell_quote("a b"), "'a b'");
    }

    #[test]
    fn result_constants_match_spec() {
        assert_eq!(FORBID, 0);
        assert_eq!(ALLOW, 1);
        assert_eq!(ALLOW_ONCE, 2);
    }
}
