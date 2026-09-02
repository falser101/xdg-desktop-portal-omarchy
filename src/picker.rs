//! Dialogs run in a child process so Hyprland `killactive` cannot SIGKILL the
//! portal daemon, and so eframe/winit own a real main thread.

use crate::ui::{
    AccessRequest, AccessResult, AccountRequest, AccountResult, AppChooserRequest,
    FileChooserRequest, FileChooserResult,
};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PickerRequest {
    FileChooser(FileChooserRequest),
    AppChooser(AppChooserRequest),
    Access(AccessRequest),
    Account(AccountRequest),
    Background {
        title: String,
        subtitle: String,
        body: String,
    },
    Wallpaper { uri: String },
    Confirm {
        title: String,
        subtitle: String,
        accept: String,
    },
    DynamicLauncher {
        main_text: String,
        subtitle: String,
        name: String,
        #[serde(default)]
        icon_path: String,
        #[serde(default)]
        target: String,
        #[serde(default = "default_true")]
        editable_name: bool,
    },
    Screenshot,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PickerReply {
    Cancel,
    FileChooser(FileChooserResult),
    App {
        choice: String,
        #[serde(default)]
        remember: bool,
    },
    Access(AccessResult),
    Account(AccountResult),
    /// 0 Forbid, 1 Allow, 2 Allow once
    Background { result: u32 },
    Wallpaper { granted: bool },
    Confirm { accepted: bool },
    DynamicLauncher {
        accepted: bool,
        #[serde(default)]
        name: String,
    },
    Screenshot { target: u32 },
}

pub fn run_blocking(req: PickerRequest) -> PickerReply {
    let token = CancellationToken::new();
    match req {
        PickerRequest::FileChooser(r) => crate::ui::run_file_chooser(r, token)
            .map(PickerReply::FileChooser)
            .unwrap_or(PickerReply::Cancel),
        PickerRequest::AppChooser(r) => crate::ui::run_app_chooser(r, token)
            .map(|(choice, remember)| PickerReply::App { choice, remember })
            .unwrap_or(PickerReply::Cancel),
        PickerRequest::Access(r) => crate::ui::run_access(r, token)
            .map(PickerReply::Access)
            .unwrap_or(PickerReply::Cancel),
        PickerRequest::Account(r) => crate::ui::run_account(r, token)
            .map(PickerReply::Account)
            .unwrap_or(PickerReply::Cancel),
        PickerRequest::Background {
            title,
            subtitle,
            body,
        } => PickerReply::Background {
            result: crate::ui::run_background(title, subtitle, body, token).unwrap_or(0),
        },
        PickerRequest::Wallpaper { uri } => PickerReply::Wallpaper {
            granted: crate::ui::run_wallpaper_confirm(uri, token),
        },
        PickerRequest::Confirm {
            title,
            subtitle,
            accept,
        } => PickerReply::Confirm {
            accepted: crate::ui::run_confirm(title, subtitle, accept, token),
        },
        PickerRequest::DynamicLauncher {
            main_text,
            subtitle,
            name,
            ..
        } => {
            let accepted = crate::ui::run_confirm(
                main_text,
                subtitle,
                "Create".into(),
                token,
            );
            PickerReply::DynamicLauncher {
                accepted,
                name,
            }
        }
        PickerRequest::Screenshot => {
            if crate::ui::run_confirm(
                "Take a screenshot?".into(),
                "Choose a capture target after confirming.".into(),
                "Capture".into(),
                token,
            ) {
                PickerReply::Screenshot { target: 1 }
            } else {
                PickerReply::Cancel
            }
        }
    }
}

pub fn child_main() -> anyhow::Result<()> {
    let req: PickerRequest = serde_json::from_reader(std::io::stdin().lock())?;
    let reply = run_blocking(req);
    serde_json::to_writer(std::io::stdout().lock(), &reply)?;
    Ok(())
}

pub async fn run(req: PickerRequest, token: CancellationToken) -> Option<PickerReply> {
    let exe = std::env::current_exe().ok()?;
    let payload = serde_json::to_vec(&req).ok()?;
    let mut child = tokio::process::Command::new(exe)
        .arg("--picker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .ok()?;

    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(&payload).await.is_err() {
            let _ = child.start_kill();
            return None;
        }
        drop(stdin);
    }

    let mut stdout = child.stdout.take()?;
    let mut buf = Vec::new();
    tokio::select! {
        _ = token.cancelled() => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return None;
        }
        read = tokio::io::AsyncReadExt::read_to_end(&mut stdout, &mut buf) => {
            let _ = read.ok()?;
            let _ = child.wait().await;
        }
    }

    if buf.is_empty() {
        return None;
    }
    serde_json::from_slice(&buf).ok()
}
