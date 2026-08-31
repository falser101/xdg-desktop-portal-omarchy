use super::{cancelled, run_native};
use crate::desktop::{load_apps, DesktopApp};
use crate::theme::OmarchyTheme;
use egui::{RichText, ScrollArea};
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
    let _ = run_native(title, [480.0, 520.0], AppUi { state: ui_state, token });
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
}

impl eframe::App for AppUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if cancelled(&self.token) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(120));
        let theme = OmarchyTheme::load();
        let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let mut close = false;
        {
            let mut app = self.state.lock().unwrap();
            if app.done {
                close = true;
            } else {
                egui::TopBottomPanel::bottom("act").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            app.accepted = false;
                            app.done = true;
                            close = true;
                        }
                        if ui.button(RichText::new("Open").strong()).clicked() {
                            app.accepted = true;
                            app.done = true;
                            close = true;
                        }
                    });
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label(RichText::new(&app.req.title).size(18.0).strong());
                    if let Some(uri) = &app.req.uri {
                        ui.label(RichText::new(uri).color(super::visuals::rgb(theme.muted)));
                    } else if let Some(name) = &app.req.filename {
                        ui.label(RichText::new(name).color(super::visuals::rgb(theme.muted)));
                    } else if let Some(ct) = &app.req.content_type {
                        ui.label(RichText::new(ct).color(super::visuals::rgb(theme.muted)));
                    }
                    if let Some(ct) = app.req.content_type.clone() {
                        ui.checkbox(
                            &mut app.remember,
                            format!("Set as default app to open {ct} files"),
                        );
                    }
                    ui.add(
                        egui::TextEdit::singleline(&mut app.query)
                            .hint_text("Search applications")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(6.0);
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
                    ScrollArea::vertical().show(ui, |ui| {
                        for desktop in apps {
                            let selected = app.selected.as_deref() == Some(desktop.id.as_str());
                            let resp = ui.selectable_label(selected, format!("{}  {}", desktop.name, desktop.id));
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
