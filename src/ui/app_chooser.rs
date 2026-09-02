use super::chrome::{
    caption_text, muted_of, search_field, title_text, trailing_actions, ROW_H,
};
use super::icons::IconCache;
use super::{cancelled, run_native_sized};
use crate::desktop::{load_apps, DesktopApp};
use egui::{Align2, FontId, Frame, Margin, Pos2, Rect, ScrollArea, Sense, Vec2};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AppChooserRequest {
    pub title: String,
    pub choices: Vec<String>,
    pub last_choice: Option<String>,
    pub content_type: Option<String>,
    pub uri: Option<String>,
    pub filename: Option<String>,
}

pub fn run_app_chooser(
    req: AppChooserRequest,
    token: CancellationToken,
) -> Option<(String, bool)> {
    let apps = load_apps(&req.choices);
    if apps.is_empty() {
        return None;
    }
    let state = Arc::new(Mutex::new(Chooser {
        req,
        apps,
        selected: None,
        query: String::new(),
        remember: false,
        done: false,
        accepted: false,
    }));
    {
        let mut app = state.lock().unwrap();
        if let Some(last) = app.req.last_choice.clone() {
            if app.apps.iter().any(|a| a.id == last) {
                app.selected = Some(last);
            }
        }
        if app.selected.is_none() {
            app.selected = app.apps.first().map(|a| a.id.clone());
        }
    }
    let ui_state = Arc::clone(&state);
    let title = ui_state.lock().unwrap().req.title.clone();
    let _ = run_native_sized(
        title,
        [480.0, 560.0],
        [400.0, 360.0],
        AppUi {
            state: ui_state,
            token,
            icons: IconCache::default(),
        },
    );
    let app = state.lock().unwrap();
    if app.accepted {
        app.selected.clone().map(|id| (id, app.remember))
    } else {
        None
    }
}

struct Chooser {
    req: AppChooserRequest,
    apps: Vec<DesktopApp>,
    selected: Option<String>,
    query: String,
    remember: bool,
    done: bool,
    accepted: bool,
}

struct AppUi {
    state: Arc<Mutex<Chooser>>,
    token: CancellationToken,
    icons: IconCache,
}

impl eframe::App for AppUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if cancelled(&self.token) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(120));
        let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let mut close = false;
        {
            let mut app = self.state.lock().unwrap();
            if app.done {
                close = true;
            } else {
                let fill = ctx.style().visuals.panel_fill;
                egui::TopBottomPanel::bottom("act")
                    .frame(
                        Frame::new()
                            .fill(fill)
                            .inner_margin(Margin::symmetric(20, 14)),
                    )
                    .show(ctx, |ui| {
                        if let Some(ct) = app.req.content_type.clone() {
                            ui.checkbox(
                                &mut app.remember,
                                format!("Always Open With This App for {ct}"),
                            );
                            ui.add_space(10.0);
                        }
                        let (cancelled, accepted) = trailing_actions(ui, "Cancel", "Open");
                        if cancelled {
                            app.accepted = false;
                            app.done = true;
                            close = true;
                        }
                        if accepted {
                            app.accepted = true;
                            app.done = true;
                            close = true;
                        }
                    });
                egui::CentralPanel::default()
                    .frame(
                        Frame::new()
                            .fill(fill)
                            .inner_margin(Margin::symmetric(20, 16)),
                    )
                    .show(ctx, |ui| {
                    ui.label(title_text(&app.req.title));
                    let muted = muted_of(ui);
                    if let Some(uri) = &app.req.uri {
                        ui.add_space(4.0);
                        ui.label(caption_text(uri, muted));
                    } else if let Some(name) = &app.req.filename {
                        ui.add_space(4.0);
                        ui.label(caption_text(name, muted));
                    } else if let Some(ct) = &app.req.content_type {
                        ui.add_space(4.0);
                        ui.label(caption_text(ct, muted));
                    }
                    ui.add_space(12.0);
                    search_field(ui, &mut app.query, "Search", ui.available_width());
                    ui.add_space(10.0);
                    let query = app.query.to_lowercase();
                    let apps: Vec<_> = app
                        .apps
                        .iter()
                        .filter(|a| {
                            query.is_empty()
                                || a.name.to_lowercase().contains(&query)
                                || a.id.to_lowercase().contains(&query)
                        })
                        .cloned()
                        .collect();
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if apps.is_empty() {
                                ui.add_space(24.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(caption_text("No Applications", muted_of(ui)));
                                });
                            }
                            for desktop in apps {
                                let selected =
                                    app.selected.as_deref() == Some(desktop.id.as_str());
                                let resp = app_row(ui, &mut self.icons, &desktop, selected);
                                if resp.clicked() {
                                    app.selected = Some(desktop.id.clone());
                                }
                                if resp.double_clicked() {
                                    app.selected = Some(desktop.id);
                                    app.accepted = true;
                                    app.done = true;
                                    close = true;
                                }
                            }
                        });
                });
                if escape {
                    app.accepted = false;
                    app.done = true;
                    close = true;
                } else if enter && app.selected.is_some() {
                    app.accepted = true;
                    app.done = true;
                    close = true;
                }
            }
        }
        if close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn app_row(
    ui: &mut egui::Ui,
    icons: &mut IconCache,
    app: &DesktopApp,
    selected: bool,
) -> egui::Response {
    let height = ROW_H + 8.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let vis = ui.visuals();
    let row = Rect::from_min_max(
        Pos2::new(rect.left() + 2.0, rect.top() + 1.0),
        Pos2::new(rect.right() - 2.0, rect.bottom() - 1.0),
    );
    if selected {
        ui.painter()
            .rect_filled(row, 8.0, vis.selection.bg_fill);
    } else if resp.hovered() {
        ui.painter()
            .rect_filled(row, 8.0, vis.widgets.hovered.bg_fill);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let icon = Rect::from_center_size(
        Pos2::new(row.left() + 22.0, row.center().y),
        Vec2::splat(28.0),
    );
    let names = if app.icon.is_empty() {
        vec!["application-x-executable".to_string()]
    } else {
        vec![app.icon.clone()]
    };
    icons.paint_at(ui, &names, icon);
    let name_pos = Pos2::new(icon.right() + 10.0, row.center().y - 7.0);
    ui.painter().text(
        name_pos,
        Align2::LEFT_CENTER,
        &app.name,
        FontId::proportional(13.0),
        vis.text_color(),
    );
    ui.painter().text(
        Pos2::new(icon.right() + 10.0, row.center().y + 8.0),
        Align2::LEFT_CENTER,
        &app.id,
        FontId::proportional(11.0),
        vis.weak_text_color(),
    );
    resp
}
