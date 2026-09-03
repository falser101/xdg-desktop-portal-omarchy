use crate::theme::OmarchyTheme;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use zbus::object_server::{InterfaceRef, SignalEmitter};
use zbus::zvariant::{OwnedValue, Value};
use zbus::Connection;

const APPEARANCE: &str = "org.freedesktop.appearance";
const GNOME_INTERFACE: &str = "org.gnome.desktop.interface";

pub struct Settings {
    theme: Mutex<OmarchyTheme>,
}

impl Settings {
    pub fn load() -> Self {
        Self {
            theme: Mutex::new(OmarchyTheme::load()),
        }
    }

    fn snapshot(&self) -> OmarchyTheme {
        self.theme.lock().unwrap().clone()
    }

    fn read_one_value(theme: &OmarchyTheme, namespace: &str, key: &str) -> Option<OwnedValue> {
        match (namespace, key) {
            (APPEARANCE, "color-scheme") => owned(theme.color_scheme_u32()),
            (APPEARANCE, "accent-color") => owned((theme.accent[0], theme.accent[1], theme.accent[2])),
            (APPEARANCE, "contrast") => owned(0u32),
            (APPEARANCE, "reduced-motion") => owned(0u32),
            (GNOME_INTERFACE, "color-scheme") => {
                let s = match theme.mode {
                    crate::theme::ColorScheme::PreferLight => "prefer-light",
                    _ => "prefer-dark",
                };
                owned(s.to_string())
            }
            (GNOME_INTERFACE, "gtk-theme") => owned(theme.gtk_theme.clone()),
            (GNOME_INTERFACE, "icon-theme") => owned(theme.icon_theme.clone()),
            (GNOME_INTERFACE, "font-name") => owned(format!("{},  {}", theme.font_family, theme.font_pt)),
            (GNOME_INTERFACE, "text-scaling-factor") => owned(f64::from(theme.type_scale())),
            _ => None,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Settings")]
impl Settings {
    async fn read(&self, namespace: &str, key: &str) -> zbus::fdo::Result<OwnedValue> {
        self.read_one(namespace, key).await
    }

    async fn read_all(
        &self,
        namespaces: Vec<&str>,
    ) -> HashMap<String, HashMap<String, OwnedValue>> {
        let theme = self.snapshot();
        let known = [APPEARANCE, GNOME_INTERFACE];
        let mut out = HashMap::new();
        for ns in known {
            if !namespace_matches(&namespaces, ns) {
                continue;
            }
            let keys = match ns {
                APPEARANCE => [
                    "color-scheme",
                    "accent-color",
                    "contrast",
                    "reduced-motion",
                ]
                .as_slice(),
                GNOME_INTERFACE => [
                    "color-scheme",
                    "gtk-theme",
                    "icon-theme",
                    "font-name",
                    "text-scaling-factor",
                ]
                .as_slice(),
                _ => &[],
            };
            let mut inner = HashMap::new();
            for key in keys {
                if let Some(v) = Self::read_one_value(&theme, ns, key) {
                    inner.insert((*key).to_string(), v);
                }
            }
            out.insert(ns.to_string(), inner);
        }
        out
    }

    async fn read_one(&self, namespace: &str, key: &str) -> zbus::fdo::Result<OwnedValue> {
        let theme = self.snapshot();
        Self::read_one_value(&theme, namespace, key).ok_or_else(|| {
            zbus::fdo::Error::Failed(format!("unknown setting {namespace} {key}"))
        })
    }

    #[zbus(signal)]
    async fn setting_changed(
        emitter: &SignalEmitter<'_>,
        namespace: &str,
        key: &str,
        value: Value<'_>,
    ) -> zbus::Result<()>;

    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        2
    }
}

fn owned<T: Into<Value<'static>>>(v: T) -> Option<OwnedValue> {
    OwnedValue::try_from(v.into()).ok()
}

pub fn namespace_matches(patterns: &[&str], namespace: &str) -> bool {
    if patterns.is_empty() || patterns.iter().any(|p| p.is_empty()) {
        return true;
    }
    patterns.iter().any(|pattern| {
        if let Some(prefix) = pattern.strip_suffix('*') {
            namespace.starts_with(prefix)
        } else {
            *pattern == namespace
        }
    })
}

pub async fn watch_theme(connection: Connection) {
    let iface: InterfaceRef<Settings> = match connection
        .object_server()
        .interface(crate::DBUS_PATH)
        .await
    {
        Ok(i) => i,
        Err(err) => {
            tracing::error!("settings interface: {err}");
            return;
        }
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(8);
    let tx_watch = tx.clone();
    let mut watcher = match RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if res.is_ok() {
                let _ = tx_watch.blocking_send(());
            }
        },
        notify::Config::default(),
    ) {
        Ok(w) => w,
        Err(err) => {
            tracing::error!("theme watcher: {err}");
            return;
        }
    };

    let current = crate::paths::state_home().join("omarchy/current");
    let _ = watcher.watch(&current, RecursiveMode::Recursive);
    let theme_dir = crate::paths::current_theme_dir();
    if theme_dir.exists() {
        let _ = watcher.watch(&theme_dir, RecursiveMode::NonRecursive);
    }

    let mut debounce = tokio::time::interval(Duration::from_millis(250));
    let mut dirty = false;
    loop {
        tokio::select! {
            Some(()) = rx.recv() => dirty = true,
            _ = debounce.tick() => {
                if !dirty {
                    continue;
                }
                dirty = false;
                let next = OmarchyTheme::load();
                let prev = {
                    let iface_ref = iface.get().await;
                    let mut guard = iface_ref.theme.lock().unwrap();
                    let prev = guard.clone();
                    *guard = next.clone();
                    prev
                };
                if prev.mode != next.mode {
                    let _ = Settings::setting_changed(
                        iface.signal_emitter(),
                        APPEARANCE,
                        "color-scheme",
                        Value::from(next.color_scheme_u32()),
                    )
                    .await;
                    let scheme = match next.mode {
                        crate::theme::ColorScheme::PreferLight => "prefer-light",
                        _ => "prefer-dark",
                    };
                    let _ = Settings::setting_changed(
                        iface.signal_emitter(),
                        GNOME_INTERFACE,
                        "color-scheme",
                        Value::from(scheme),
                    )
                    .await;
                }
                if prev.accent != next.accent {
                    let _ = Settings::setting_changed(
                        iface.signal_emitter(),
                        APPEARANCE,
                        "accent-color",
                        Value::from((next.accent[0], next.accent[1], next.accent[2])),
                    )
                    .await;
                }
                if prev.icon_theme != next.icon_theme {
                    let _ = Settings::setting_changed(
                        iface.signal_emitter(),
                        GNOME_INTERFACE,
                        "icon-theme",
                        Value::from(next.icon_theme.clone()),
                    )
                    .await;
                }
                if prev.gtk_theme != next.gtk_theme {
                    let _ = Settings::setting_changed(
                        iface.signal_emitter(),
                        GNOME_INTERFACE,
                        "gtk-theme",
                        Value::from(next.gtk_theme.clone()),
                    )
                    .await;
                }
                if prev.font_family != next.font_family || (prev.font_pt - next.font_pt).abs() > 0.01
                {
                    let _ = Settings::setting_changed(
                        iface.signal_emitter(),
                        GNOME_INTERFACE,
                        "font-name",
                        Value::from(format!("{},  {}", next.font_family, next.font_pt)),
                    )
                    .await;
                    let _ = Settings::setting_changed(
                        iface.signal_emitter(),
                        GNOME_INTERFACE,
                        "text-scaling-factor",
                        Value::from(f64::from(next.type_scale())),
                    )
                    .await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_namespace() {
        assert!(namespace_matches(&["org.freedesktop.*"], APPEARANCE));
        assert!(!namespace_matches(&["org.gnome.*"], APPEARANCE));
        assert!(namespace_matches(&[], APPEARANCE));
    }
}
