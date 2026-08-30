pub mod access;
pub mod account;
pub mod app_chooser;
pub mod background;
pub mod dynamic_launcher;
pub mod email;
pub mod file_chooser;
pub mod inhibit;
pub mod lockdown;
pub mod notification;
pub mod screenshot;
pub mod settings;
pub mod wallpaper;

use zbus::Connection;

#[derive(Clone)]
pub struct PortalCtx {
    pub connection: Connection,
}

impl PortalCtx {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }
}
