//! XDG Desktop Portal backend for Omarchy.
//!
//! Implements `org.freedesktop.impl.portal.*` so `xdg-desktop-portal` can
//! route FileChooser, Settings, Screenshot, and related dialogs through
//! Omarchy instead of GTK. ScreenCast / GlobalShortcuts / InputCapture stay
//! on xdg-desktop-portal-hyprland (capture engine + our share picker).

pub mod desktop;
pub mod dict;
pub mod documents;
pub mod filters;
pub mod paths;
pub mod picker;
pub mod portals;
pub mod request;
pub mod response;
pub mod theme;
pub mod ui;
pub mod uri;

pub const DBUS_NAME: &str = "org.freedesktop.impl.portal.desktop.omarchy";
pub const DBUS_PATH: &str = "/org/freedesktop/portal/desktop";
pub const APP_ID: &str = "xdg-desktop-portal-omarchy";
