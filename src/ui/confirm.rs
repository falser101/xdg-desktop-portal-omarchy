use super::{cancelled, run_native};
use crate::dict::Choice;
use crate::paths::{face_image, real_name, whoami};
use crate::theme::OmarchyTheme;
use egui::RichText;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AccessRequest {
    pub title: String,
    pub subtitle: String,
    pub body: String,
    pub deny_label: String,
    pub grant_label: String,
    #[serde(default)]
    pub icon: Option<String>,
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AccessResult {
    pub granted: bool,
    pub choices: Vec<(String, String)>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AccountRequest {
    pub title: String,
    pub reason: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AccountResult {
    pub id: String,
    pub name: String,
    pub image: Option<PathBuf>,
}

pub fn run_access(req: AccessRequest, token: CancellationToken) -> Option<AccessResult> {
    run_prompt(PromptKind::Access(req), token).and_then(|r| match r {
        PromptResult::Access(r) => Some(r),
        _ => None,
    })
}

pub fn run_account(req: AccountRequest, token: CancellationToken) -> Option<AccountResult> {
    run_prompt(PromptKind::Account(req), token).and_then(|r| match r {
        PromptResult::Account(r) => Some(r),
        _ => None,
    })
}

pub fn run_wallpaper_confirm(uri: String, token: CancellationToken) -> bool {
    matches!(
        run_prompt(PromptKind::Wallpaper { uri }, token),
        Some(PromptResult::Granted)
    )
}

pub fn run_confirm(title: String, subtitle: String, accept: String, token: CancellationToken) -> bool {
    matches!(
        run_prompt(
            PromptKind::Confirm {
                title,
                subtitle,
                accept,
            },
            token,
        ),
        Some(PromptResult::Granted)
    )
}

enum PromptKind {
    Access(AccessRequest),
    Account(AccountRequest),
    Wallpaper { uri: String },
    Confirm {
        title: String,
        subtitle: String,
        accept: String,
    },
}

enum PromptResult {
    Access(AccessResult),
    Account(AccountResult),
    Granted,
}

struct PromptApp {
    kind: PromptKind,
    choice_values: Vec<String>,
    accepted: Option<bool>,
}

fn run_prompt(kind: PromptKind, token: CancellationToken) -> Option<PromptResult> {
    let title = match &kind {
        PromptKind::Access(r) => r.title.clone(),
        PromptKind::Account(r) => r.title.clone(),
        PromptKind::Wallpaper { .. } => "Set wallpaper".into(),
        PromptKind::Confirm { title, .. } => title.clone(),
    };
    let choice_values = match &kind {
        PromptKind::Access(r) => r.choices.iter().map(|c| c.selected.clone()).collect(),
        _ => Vec::new(),
    };
    let state = Arc::new(Mutex::new(PromptApp {
        kind,
        choice_values,
        accepted: None,
    }));
    let ui_state = Arc::clone(&state);
    let _ = run_native(title, [520.0, 280.0], PromptUi { state: ui_state, token });
    let app = state.lock().unwrap();
    match (app.accepted, &app.kind) {
        (Some(true), PromptKind::Access(req)) => Some(PromptResult::Access(AccessResult {
            granted: true,
            choices: req
                .choices
                .iter()
                .zip(app.choice_values.iter())
                .map(|(c, v)| (c.id.clone(), v.clone()))
                .collect(),
        })),
        (Some(false), PromptKind::Access(_)) => Some(PromptResult::Access(AccessResult {
            granted: false,
            choices: Vec::new(),
        })),
        (Some(true), PromptKind::Account(_)) => Some(PromptResult::Account(AccountResult {
            id: whoami(),
            name: real_name(),
            image: face_image(),
        })),
        (Some(true), PromptKind::Wallpaper { .. }) => Some(PromptResult::Granted),
        (Some(true), PromptKind::Confirm { .. }) => Some(PromptResult::Granted),
        _ => None,
    }
}

struct PromptUi {
    state: Arc<Mutex<PromptApp>>,
    token: CancellationToken,
}

impl eframe::App for PromptUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if cancelled(&self.token) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(120));
        let theme = OmarchyTheme::load();
        let mut close = false;
        let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        {
            let mut app = self.state.lock().unwrap();
            if app.accepted.is_some() {
                close = true;
            } else {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add_space(8.0);
                    match &app.kind {
                        PromptKind::Access(req) => {
                            ui.label(RichText::new(&req.title).size(18.0).strong());
                            if !req.subtitle.is_empty() {
                                ui.label(RichText::new(&req.subtitle).color(super::visuals::rgb(theme.muted)));
                            }
                            if !req.body.is_empty() {
                                ui.add_space(8.0);
                                ui.label(&req.body);
                            }
                        }
                        PromptKind::Account(req) => {
                            ui.label(RichText::new(&req.title).size(18.0).strong());
                            ui.add_space(8.0);
                            ui.label(format!("User: {}", whoami()));
                            ui.label(format!("Name: {}", real_name()));
                            if !req.reason.is_empty() {
                                ui.add_space(8.0);
                                ui.label(&req.reason);
                            }
                        }
                        PromptKind::Wallpaper { uri } => {
                            ui.label(RichText::new("Set Omarchy wallpaper?").size(18.0).strong());
                            ui.add_space(8.0);
                            ui.label(uri);
                        }
                        PromptKind::Confirm {
                            title,
                            subtitle,
                            ..
                        } => {
                            ui.label(RichText::new(title).size(18.0).strong());
                            if !subtitle.is_empty() {
                                ui.add_space(8.0);
                                ui.label(RichText::new(subtitle).color(super::visuals::rgb(theme.muted)));
                            }
                        }
                    }
                    let access_choices = if let PromptKind::Access(req) = &app.kind {
                        Some(
                            req.choices
                                .iter()
                                .enumerate()
                                .map(|(i, c)| {
                                    (
                                        i,
                                        c.label.clone(),
                                        c.options.clone(),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    };
                    if let Some(choices) = access_choices {
                        ui.add_space(8.0);
                        for (i, label, options) in choices {
                            if options.is_empty() {
                                let mut on = app.choice_values[i] == "true";
                                if ui.checkbox(&mut on, label).changed() {
                                    app.choice_values[i] =
                                        if on { "true" } else { "false" }.into();
                                }
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(&label);
                                    let current = app.choice_values[i].clone();
                                    egui::ComboBox::from_id_salt(format!("access-choice-{i}"))
                                        .selected_text(
                                            options
                                                .iter()
                                                .find(|(id, _)| id == &current)
                                                .map(|(_, l)| l.as_str())
                                                .unwrap_or(current.as_str()),
                                        )
                                        .show_ui(ui, |ui| {
                                            for (id, opt_label) in &options {
                                                if ui
                                                    .selectable_label(current == *id, opt_label)
                                                    .clicked()
                                                {
                                                    app.choice_values[i] = id.clone();
                                                }
                                            }
                                        });
                                });
                            }
                        }
                    }
                    ui.add_space(16.0);
                    let (deny, grant) = match &app.kind {
                        PromptKind::Access(req) => (req.deny_label.clone(), req.grant_label.clone()),
                        PromptKind::Confirm { accept, .. } => ("Cancel".into(), accept.clone()),
                        _ => ("Cancel".into(), "Allow".into()),
                    };
                    ui.horizontal(|ui| {
                        if ui.button(deny).clicked() {
                            app.accepted = Some(false);
                        }
                        if ui.button(RichText::new(grant).strong()).clicked() {
                            app.accepted = Some(true);
                        }
                    });
                });
                if escape {
                    app.accepted = Some(false);
                } else if enter {
                    app.accepted = Some(true);
                }
            }
        }
        if close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
