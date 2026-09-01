//! Portal Notification → Omarchy FDO notification server (quickshell).
//!
//! Flatpak/sandboxed apps call `org.freedesktop.portal.Notification`. We translate
//! into `org.freedesktop.Notifications.Notify` so the shell toast stack handles
//! display, then map FDO `ActionInvoked` back to the portal signal (or
//! `org.freedesktop.Application.ActivateAction` for `app.*` actions).

use super::PortalCtx;
use crate::dict::{self, Options};
use crate::DBUS_PATH;
use futures::StreamExt;
use std::collections::HashMap;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd as StdOwnedFd};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use zbus::zvariant::{OwnedValue, Structure, Value};
use zbus::{Connection, Proxy};

const FDO_DEST: &str = "org.freedesktop.Notifications";
const FDO_PATH: &str = "/org/freedesktop/Notifications";
const FDO_IFACE: &str = "org.freedesktop.Notifications";
const IMPL_IFACE: &str = "org.freedesktop.impl.portal.Notification";

pub struct Notification {
    ctx: PortalCtx,
    state: Arc<Mutex<NotifState>>,
}

#[derive(Default)]
struct NotifState {
    /// portal (app_id, id) → live notification
    by_portal: HashMap<(String, String), LiveNotif>,
    /// FDO notification id → portal key
    by_fdo: HashMap<u32, (String, String)>,
}

struct LiveNotif {
    fdo_id: u32,
    /// FDO action key → portal action name + optional target
    actions: HashMap<String, PortalAction>,
    temp_files: Vec<PathBuf>,
}

struct PortalAction {
    /// Name reported on portal ActionInvoked / ActivateAction
    name: String,
    target: Option<OwnedValue>,
}

impl Notification {
    pub fn new(ctx: PortalCtx) -> Self {
        let state = Arc::new(Mutex::new(NotifState::default()));
        spawn_fdo_watch(ctx.connection.clone(), Arc::clone(&state));
        Self { ctx, state }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Notification")]
impl Notification {
    async fn add_notification(&self, app_id: &str, id: &str, notification: Options) {
        tracing::info!(app_id, id, "Notification.AddNotification");
        if let Err(err) = self.add_inner(app_id, id, notification).await {
            tracing::warn!(app_id, id, error = %err, "Notification.AddNotification failed");
        }
    }

    async fn remove_notification(&self, app_id: &str, id: &str) {
        tracing::info!(app_id, id, "Notification.RemoveNotification");
        let live = {
            let mut state = self.state.lock().unwrap();
            state.by_portal.remove(&(app_id.to_string(), id.to_string()))
        };
        if let Some(live) = live {
            {
                let mut state = self.state.lock().unwrap();
                state.by_fdo.remove(&live.fdo_id);
            }
            let _ = close_fdo(&self.ctx.connection, live.fdo_id).await;
            cleanup_temps(&live.temp_files);
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
        emitter: &zbus::object_server::SignalEmitter<'_>,
        app_id: &str,
        id: &str,
        action: &str,
        parameter: Vec<Value<'_>>,
    ) -> zbus::Result<()>;
}

impl Notification {
    async fn add_inner(
        &self,
        app_id: &str,
        id: &str,
        notification: Options,
    ) -> anyhow::Result<()> {
        let title = dict::as_str(&notification, "title").unwrap_or_else(|| app_id.to_string());
        let body = dict::as_str(&notification, "body")
            .or_else(|| dict::as_str(&notification, "markup-body"))
            .unwrap_or_default();
        let priority = dict::as_str(&notification, "priority").unwrap_or_else(|| "normal".into());
        let urgency: u8 = match priority.as_str() {
            "low" => 0,
            "high" | "urgent" => 2,
            _ => 1,
        };
        let persistent = display_hints(&notification)
            .iter()
            .any(|h| h == "persistent");

        let mut temp_files = Vec::new();
        let (app_icon, image_path) =
            resolve_icon(notification.get("icon"), &mut temp_files).unwrap_or_default();

        let default_action = dict::as_str(&notification, "default-action");
        let default_target = notification
            .get("default-action-target")
            .and_then(|v| v.try_clone().ok());
        let buttons = parse_buttons(&notification);

        // Quickshell treats FDO action id "default" as the body-click action.
        let mut fdo_actions: Vec<String> = Vec::new();
        let mut action_map: HashMap<String, PortalAction> = HashMap::new();
        if let Some(name) = default_action.clone() {
            fdo_actions.push("default".into());
            fdo_actions.push(String::new());
            action_map.insert(
                "default".into(),
                PortalAction {
                    name,
                    target: default_target,
                },
            );
        }
        for button in &buttons {
            if button.action.is_empty() {
                continue;
            }
            // Avoid colliding with the reserved "default" key.
            let fdo_key = if button.action == "default" {
                "portal.default".to_string()
            } else {
                button.action.clone()
            };
            fdo_actions.push(fdo_key.clone());
            fdo_actions.push(button.label.clone());
            action_map.insert(
                fdo_key,
                PortalAction {
                    name: button.action.clone(),
                    target: button.target.clone(),
                },
            );
        }

        // Replace an existing portal notification with the same id.
        let replaces = {
            let mut state = self.state.lock().unwrap();
            if let Some(old) = state
                .by_portal
                .remove(&(app_id.to_string(), id.to_string()))
            {
                state.by_fdo.remove(&old.fdo_id);
                let rid = old.fdo_id;
                cleanup_temps(&old.temp_files);
                rid
            } else {
                0
            }
        };

        let mut hints: HashMap<String, Value<'_>> = HashMap::new();
        hints.insert("urgency".into(), Value::U8(urgency));
        hints.insert("desktop-entry".into(), Value::Str(app_id.into()));
        if persistent {
            // Shell advertises "persistence"; FDO resident keeps the toast.
            hints.insert("resident".into(), Value::Bool(true));
            hints.insert("persistence".into(), Value::Bool(true));
        }
        if let Some(path) = &image_path {
            hints.insert(
                "image-path".into(),
                Value::Str(path.to_string_lossy().into_owned().into()),
            );
        }

        let expire_timeout: i32 = if persistent { 0 } else { -1 };
        let app_name = if app_id.is_empty() {
            "portal"
        } else {
            app_id
        };

        let fdo_id = notify_fdo(
            &self.ctx.connection,
            app_name,
            replaces,
            &app_icon,
            &title,
            &body,
            &fdo_actions,
            hints,
            expire_timeout,
        )
        .await?;

        let live = LiveNotif {
            fdo_id,
            actions: action_map,
            temp_files,
        };
        let mut state = self.state.lock().unwrap();
        state
            .by_fdo
            .insert(fdo_id, (app_id.to_string(), id.to_string()));
        state
            .by_portal
            .insert((app_id.to_string(), id.to_string()), live);
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Button {
    label: String,
    action: String,
    target: Option<OwnedValue>,
}

fn display_hints(opts: &Options) -> Vec<String> {
    dict::as_str_vec(opts, "display-hint").unwrap_or_default()
}

fn parse_buttons(opts: &Options) -> Vec<Button> {
    let Some(raw) = opts.get("buttons") else {
        return Vec::new();
    };
    let Ok(value) = raw.try_clone() else {
        return Vec::new();
    };
    let value: Value<'static> = value.into();
    // aa{sv}
    let Ok(arr) = <Vec<HashMap<String, OwnedValue>>>::try_from(value) else {
        tracing::debug!("Notification buttons: unexpected type");
        return Vec::new();
    };
    arr.into_iter()
        .map(|m| {
            let label = m
                .get("label")
                .and_then(owned_string)
                .unwrap_or_default();
            let action = m
                .get("action")
                .and_then(owned_string)
                .unwrap_or_default();
            let target = m.get("target").and_then(|v| v.try_clone().ok());
            Button {
                label,
                action,
                target,
            }
        })
        .filter(|b| !b.action.is_empty())
        .collect()
}

fn owned_string(v: &OwnedValue) -> Option<String> {
    let value: Value<'static> = v.try_clone().ok()?.into();
    String::try_from(value).ok()
}

fn resolve_icon(
    value: Option<&OwnedValue>,
    temps: &mut Vec<PathBuf>,
) -> Option<(String, Option<PathBuf>)> {
    let value = value?;
    let cloned = value.try_clone().ok()?;
    let v: Value<'static> = cloned.into();

    // Historical: plain string icon name
    if let Ok(name) = String::try_from(v.try_clone().ok()?) {
        let name = name.trim().to_string();
        if name.is_empty() {
            return None;
        }
        if name.starts_with('/') || name.starts_with("file:") {
            let path = name.strip_prefix("file://").unwrap_or(&name);
            return Some((String::new(), Some(PathBuf::from(path))));
        }
        return Some((name, None));
    }

    // Serialized GIcon: (s, v) — themed | bytes | file-descriptor
    let structure = Structure::try_from(v).ok()?;
    let fields: Vec<Value<'static>> = structure.into_fields();
    if fields.len() < 2 {
        return None;
    }
    let kind = String::try_from(fields[0].try_clone().ok()?).ok()?;
    let data = fields[1].try_clone().ok()?;

    match kind.as_str() {
        "themed" => {
            let names = Vec::<String>::try_from(data).ok()?;
            let name = names.into_iter().next().unwrap_or_default();
            if name.is_empty() {
                None
            } else {
                Some((name, None))
            }
        }
        "bytes" => {
            let bytes = Vec::<u8>::try_from(data).ok()?;
            let path = write_temp_icon(&bytes, temps)?;
            Some((String::new(), Some(path)))
        }
        "file-descriptor" => {
            let path = icon_from_fd(data, temps)?;
            Some((String::new(), Some(path)))
        }
        other => {
            tracing::debug!(kind = other, "unsupported portal notification icon kind");
            None
        }
    }
}

fn icon_from_fd(data: Value<'static>, temps: &mut Vec<PathBuf>) -> Option<PathBuf> {
    let raw = match data {
        Value::Fd(fd) => fd.as_raw_fd(),
        Value::Value(inner) => match *inner {
            Value::Fd(fd) => fd.as_raw_fd(),
            _ => return None,
        },
        _ => return None,
    };
    read_fd_to_temp(raw, temps)
}

fn read_fd_to_temp(raw: i32, temps: &mut Vec<PathBuf>) -> Option<PathBuf> {
    // SAFETY: dup the portal-provided fd so closing our File does not close theirs.
    let dup = unsafe { libc::dup(raw) };
    if dup < 0 {
        return None;
    }
    let owned = unsafe { StdOwnedFd::from_raw_fd(dup) };
    let mut file = std::fs::File::from(owned);
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    write_temp_icon(&bytes, temps)
}

fn write_temp_icon(bytes: &[u8], temps: &mut Vec<PathBuf>) -> Option<PathBuf> {
    if bytes.is_empty() {
        return None;
    }
    let ext = if bytes.starts_with(b"\x89PNG") {
        "png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
        "svg"
    } else {
        "bin"
    };
    let path = std::env::temp_dir().join(format!(
        "omarchy-portal-notif-{}-{}.{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos(),
        ext
    ));
    std::fs::write(&path, bytes).ok()?;
    temps.push(path.clone());
    Some(path)
}

fn cleanup_temps(paths: &[PathBuf]) {
    for path in paths {
        let _ = std::fs::remove_file(path);
    }
}

async fn notify_fdo(
    conn: &Connection,
    app_name: &str,
    replaces_id: u32,
    app_icon: &str,
    summary: &str,
    body: &str,
    actions: &[String],
    hints: HashMap<String, Value<'_>>,
    expire_timeout: i32,
) -> anyhow::Result<u32> {
    let reply = conn
        .call_method(
            Some(FDO_DEST),
            FDO_PATH,
            Some(FDO_IFACE),
            "Notify",
            &(
                app_name,
                replaces_id,
                app_icon,
                summary,
                body,
                actions,
                hints,
                expire_timeout,
            ),
        )
        .await?;
    Ok(reply.body().deserialize::<u32>()?)
}

async fn close_fdo(conn: &Connection, id: u32) -> anyhow::Result<()> {
    conn.call_method(
        Some(FDO_DEST),
        FDO_PATH,
        Some(FDO_IFACE),
        "CloseNotification",
        &(id,),
    )
    .await?;
    Ok(())
}

fn spawn_fdo_watch(conn: Connection, state: Arc<Mutex<NotifState>>) {
    tokio::spawn(async move {
        let proxy = match Proxy::new(&conn, FDO_DEST, FDO_PATH, FDO_IFACE).await {
            Ok(p) => p,
            Err(err) => {
                tracing::warn!(error = %err, "Notification: FDO proxy failed");
                return;
            }
        };
        let mut action_stream = match proxy.receive_signal("ActionInvoked").await {
            Ok(s) => s,
            Err(err) => {
                tracing::warn!(error = %err, "Notification: ActionInvoked watch failed");
                return;
            }
        };
        let mut closed_stream = match proxy.receive_signal("NotificationClosed").await {
            Ok(s) => s,
            Err(err) => {
                tracing::warn!(error = %err, "Notification: NotificationClosed watch failed");
                return;
            }
        };

        tracing::info!("Notification: watching FDO ActionInvoked / NotificationClosed");
        loop {
            tokio::select! {
                msg = action_stream.next() => {
                    let Some(msg) = msg else { break };
                    let body = msg.body();
                    let (fdo_id, action_key): (u32, String) = match body.deserialize() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    handle_action(&conn, &state, fdo_id, &action_key).await;
                }
                msg = closed_stream.next() => {
                    let Some(msg) = msg else { break };
                    let body = msg.body();
                    let (fdo_id, _reason): (u32, u32) = match body.deserialize() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let live = {
                        let mut state = state.lock().unwrap();
                        if let Some(key) = state.by_fdo.remove(&fdo_id) {
                            state.by_portal.remove(&key)
                        } else {
                            None
                        }
                    };
                    if let Some(live) = live {
                        cleanup_temps(&live.temp_files);
                    }
                }
            }
        }
    });
}

async fn handle_action(
    conn: &Connection,
    state: &Arc<Mutex<NotifState>>,
    fdo_id: u32,
    action_key: &str,
) {
    let (app_id, notif_id, portal_action) = {
        let state = state.lock().unwrap();
        let Some(key) = state.by_fdo.get(&fdo_id) else {
            return;
        };
        let Some(live) = state.by_portal.get(key) else {
            return;
        };
        let portal_action = match live.actions.get(action_key) {
            Some(action) => PortalAction {
                name: action.name.clone(),
                target: action.target.as_ref().and_then(|v| v.try_clone().ok()),
            },
            None => PortalAction {
                name: action_key.to_string(),
                target: None,
            },
        };
        (key.0.clone(), key.1.clone(), portal_action)
    };

    tracing::info!(
        app_id = %app_id,
        id = %notif_id,
        action = %portal_action.name,
        "Notification action invoked"
    );

    let mut params: Vec<Value<'_>> = Vec::new();
    if let Some(target) = portal_action.target.as_ref().and_then(|v| v.try_clone().ok()) {
        let owned: Value<'static> = target.into();
        params.push(owned);
    }

    if portal_action.name.starts_with("app.") {
        let action_id = portal_action.name.trim_start_matches("app.");
        activate_app_action(conn, &app_id, action_id, &params).await;
        return;
    }

    // Spec: activate the app, then emit ActionInvoked for non-app.* actions.
    activate_app(conn, &app_id).await;
    if let Err(err) = emit_portal_action(conn, &app_id, &notif_id, &portal_action.name, &params).await
    {
        tracing::warn!(error = %err, "emit ActionInvoked failed");
    }
}

async fn emit_portal_action(
    conn: &Connection,
    app_id: &str,
    id: &str,
    action: &str,
    params: &[Value<'_>],
) -> zbus::Result<()> {
    // Frontend xdg-desktop-portal listens for impl ActionInvoked.
    conn.emit_signal(
        None::<&str>,
        DBUS_PATH,
        IMPL_IFACE,
        "ActionInvoked",
        &(app_id, id, action, params),
    )
    .await
}

fn app_object_path(app_id: &str) -> String {
    let mut path = String::from('/');
    for ch in app_id.chars() {
        match ch {
            '.' => path.push('/'),
            '-' => path.push('_'),
            c => path.push(c),
        }
    }
    path
}

async fn activate_app(conn: &Connection, app_id: &str) {
    if app_id.is_empty() {
        return;
    }
    let path = app_object_path(app_id);
    let platform: HashMap<String, Value<'_>> = HashMap::new();
    let _ = conn
        .call_method(
            Some(app_id),
            path.as_str(),
            Some("org.freedesktop.Application"),
            "Activate",
            &(platform,),
        )
        .await;
}

async fn activate_app_action(
    conn: &Connection,
    app_id: &str,
    action_id: &str,
    params: &[Value<'_>],
) {
    if app_id.is_empty() {
        return;
    }
    let path = app_object_path(app_id);
    let platform: HashMap<String, Value<'_>> = HashMap::new();
    let param_owned: Vec<Value<'_>> = params.to_vec();
    let _ = conn
        .call_method(
            Some(app_id),
            path.as_str(),
            Some("org.freedesktop.Application"),
            "ActivateAction",
            &(action_id, param_owned, platform),
        )
        .await;
}
