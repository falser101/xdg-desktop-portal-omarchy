mod app_chooser;
mod chrome;
mod confirm;
mod file_chooser;
mod fonts;
mod glyphs;
mod icons;
mod thumbs;
mod visuals;

pub use app_chooser::{run_app_chooser, AppChooserRequest};
pub use confirm::{
    run_access, run_account, run_background, run_confirm, run_wallpaper_confirm, AccessRequest,
    AccessResult, AccountRequest, AccountResult,
};
pub use file_chooser::{run_file_chooser, FileChooserRequest, FileChooserResult, FileMode};

use tokio_util::sync::CancellationToken;

pub fn cancelled(token: &CancellationToken) -> bool {
    token.is_cancelled()
}

pub fn run_native(
    title: impl Into<String>,
    size: [f32; 2],
    app: impl eframe::App + 'static,
) -> Result<(), eframe::Error> {
    run_native_sized(title, size, [400.0, 260.0], app)
}

pub fn run_native_sized(
    title: impl Into<String>,
    size: [f32; 2],
    min_size: [f32; 2],
    app: impl eframe::App + 'static,
) -> Result<(), eframe::Error> {
    let title = title.into();
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_min_inner_size(min_size)
            .with_title(&title)
            .with_app_id(crate::APP_ID),
        centered: true,
        run_and_return: true,
        persist_window: false,
        ..Default::default()
    };
    options.event_loop_builder = Some(Box::new(|builder| {
        // The portal daemon's "main" thread is tokio. Dialogs run on worker
        // threads, which winit forbids unless any_thread is set.
        winit::platform::wayland::EventLoopBuilderExtWayland::with_any_thread(builder, true);
        winit::platform::x11::EventLoopBuilderExtX11::with_any_thread(builder, true);
    }));
    eframe::run_native(
        crate::APP_ID,
        options,
        Box::new(|cc| {
            let theme = crate::theme::OmarchyTheme::load();
            fonts::install(&cc.egui_ctx, &theme.font_family);
            visuals::apply(&cc.egui_ctx, &theme);
            Ok(Box::new(ThemedApp {
                inner: app,
                applied: theme,
            }))
        }),
    )
}

struct ThemedApp<A> {
    inner: A,
    applied: crate::theme::OmarchyTheme,
}

impl<A: eframe::App> eframe::App for ThemedApp<A> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let theme = crate::theme::OmarchyTheme::load();
        if theme != self.applied {
            if theme.font_family != self.applied.font_family {
                fonts::install(ctx, &theme.font_family);
            }
            visuals::apply(ctx, &theme);
            self.applied = theme;
            ctx.request_repaint();
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
        self.inner.update(ctx, frame);
    }
}

pub async fn on_ui_thread<T, F>(f: F) -> Option<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(f).await.ok()
}
