use super::PortalCtx;
use crate::dict::{self, Options};
use crate::request::{export_request, export_session, remove_interface, Request, Session};
use crate::response::{OTHER, SUCCESS};
use crate::DBUS_PATH;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedValue};
use zbus::Connection;

const SESSION_RUNNING: u32 = 1;

pub struct Inhibit {
    pub ctx: PortalCtx,
    monitors: Arc<Mutex<HashSet<String>>>,
    lock_active: Arc<AtomicBool>,
}

impl Inhibit {
    pub fn new(ctx: PortalCtx) -> Self {
        Self {
            ctx,
            monitors: Arc::new(Mutex::new(HashSet::new())),
            lock_active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn_watch(&self, connection: Connection) {
        let monitors = self.monitors.clone();
        let lock_active = self.lock_active.clone();
        tokio::spawn(watch_lock(connection, monitors, lock_active));
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Inhibit")]
impl Inhibit {
    async fn inhibit(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _window: &str,
        flags: u32,
        options: Options,
    ) {
        tracing::info!(app_id, flags, "Inhibit.Inhibit");
        let reason = dict::as_str(&options, "reason").unwrap_or_else(|| "Application inhibit".into());
        let what = inhibit_what(flags);
        let token = export_request(&self.ctx.connection, &handle).await;
        let mut child = start_inhibit(app_id, &reason, &what);
        token.cancelled().await;
        if let Some(child) = child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        remove_interface::<Request>(&self.ctx.connection, &handle).await;
    }

    async fn create_monitor(
        &self,
        handle: ObjectPath<'_>,
        session_handle: ObjectPath<'_>,
        app_id: &str,
        _window: &str,
    ) -> u32 {
        tracing::info!(app_id, "Inhibit.CreateMonitor");
        {
            let mut monitors = self.monitors.lock().unwrap();
            if !monitors.insert(session_handle.to_string()) {
                return OTHER;
            }
        }
        let _ = export_session(&self.ctx.connection, &session_handle).await;
        let token = export_request(&self.ctx.connection, &handle).await;
        remove_interface::<Request>(&self.ctx.connection, &handle).await;
        drop(token);

        let conn = self.ctx.connection.clone();
        let path = session_handle.to_string();
        let locked = self.lock_active.load(Ordering::Relaxed);
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            emit_state(&conn, &path, locked).await;
        });
        SUCCESS
    }

    async fn query_end_response(&self, _session_handle: ObjectPath<'_>) {}

    #[zbus(signal)]
    async fn state_changed(
        emitter: &SignalEmitter<'_>,
        session_handle: ObjectPath<'_>,
        state: HashMap<String, OwnedValue>,
    ) -> zbus::Result<()>;

    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        3
    }
}

fn inhibit_what(flags: u32) -> String {
    let mut parts = Vec::new();
    if flags & 4 != 0 {
        parts.push("sleep");
    }
    if flags & 8 != 0 {
        parts.push("idle");
    }
    if parts.is_empty() {
        parts.push("idle");
    }
    parts.join(":")
}

fn start_inhibit(who: &str, why: &str, what: &str) -> Option<std::process::Child> {
    std::process::Command::new("systemd-inhibit")
        .arg(format!("--what={what}"))
        .arg(format!("--who={who}"))
        .arg(format!("--why={why}"))
        .arg("--mode=block")
        .arg("sleep")
        .arg("infinity")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()
}

async fn watch_lock(
    conn: Connection,
    monitors: Arc<Mutex<HashSet<String>>>,
    lock_active: Arc<AtomicBool>,
) {
    loop {
        let Some(path) = socket2_path() else {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        };
        match tokio::net::UnixStream::connect(&path).await {
            Ok(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            let ev = line.trim();
                            let locked = if ev == "lock" || ev.starts_with("lock>>") {
                                Some(true)
                            } else if ev == "unlock" || ev.starts_with("unlock>>") {
                                Some(false)
                            } else {
                                None
                            };
                            if let Some(locked) = locked {
                                lock_active.store(locked, Ordering::Relaxed);
                                let sessions: Vec<String> = monitors
                                    .lock()
                                    .unwrap()
                                    .iter()
                                    .cloned()
                                    .collect();
                                let mut stale = Vec::new();
                                for session in sessions {
                                    if !session_alive(&conn, &session).await {
                                        stale.push(session);
                                        continue;
                                    }
                                    emit_state(&conn, &session, locked).await;
                                }
                                if !stale.is_empty() {
                                    let mut set = monitors.lock().unwrap();
                                    for s in stale {
                                        set.remove(&s);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => tokio::time::sleep(std::time::Duration::from_secs(2)).await,
        }
    }
}

fn socket2_path() -> Option<std::path::PathBuf> {
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".into());
    Some(std::path::PathBuf::from(runtime).join("hypr").join(sig).join(".socket2.sock"))
}

async fn session_alive(conn: &Connection, path: &str) -> bool {
    let Ok(obj) = ObjectPath::try_from(path.to_string()) else {
        return false;
    };
    conn.object_server()
        .interface::<_, Session>(&obj)
        .await
        .is_ok()
}

async fn emit_state(conn: &Connection, session: &str, locked: bool) {
    let Ok(iface) = conn.object_server().interface::<_, Inhibit>(DBUS_PATH).await else {
        return;
    };
    let Ok(path) = ObjectPath::try_from(session.to_string()) else {
        return;
    };
    let mut state = HashMap::new();
    state.insert("screensaver-active".into(), OwnedValue::from(locked));
    state.insert("session-state".into(), OwnedValue::from(SESSION_RUNNING));
    let _ = Inhibit::state_changed(iface.signal_emitter(), path, state).await;
}
