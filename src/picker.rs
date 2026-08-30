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
    Wallpaper { uri: String },
    Confirm {
        title: String,
        subtitle: String,
        accept: String,
    },
    Screenshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PickerReply {
    Cancel,
    FileChooser(FileChooserResult),
    App { choice: String },
    Access(AccessResult),
    Account(AccountResult),
    Wallpaper { granted: bool },
    Share { selection: String },
    Confirm { accepted: bool },
    Screenshot { target: u32 },
}

pub fn run_blocking(req: PickerRequest) -> PickerReply {
    let token = CancellationToken::new();
    match req {
        PickerRequest::FileChooser(r) => crate::ui::run_file_chooser(r, token)
            .map(PickerReply::FileChooser)
            .unwrap_or(PickerReply::Cancel),
        PickerRequest::AppChooser(r) => crate::ui::run_app_chooser(r, token)
            .map(|choice| PickerReply::App { choice })
            .unwrap_or(PickerReply::Cancel),
        PickerRequest::Access(r) => crate::ui::run_access(r, token)
            .map(PickerReply::Access)
            .unwrap_or(PickerReply::Cancel),
        PickerRequest::Account(r) => crate::ui::run_account(r, token)
            .map(PickerReply::Account)
            .unwrap_or(PickerReply::Cancel),
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

async fn run_via_shell(req: PickerRequest, token: CancellationToken) -> Option<PickerReply> {
    let tmp = std::env::temp_dir().join(format!(
        "omarchy-portal-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).ok()?;
    let reply_file = tmp.join("reply.json");
    let done_file = tmp.join("done");

    let mut extra = serde_json::Map::new();
    match &req {
        PickerRequest::AppChooser(r) => {
            let apps: Vec<serde_json::Value> = crate::desktop::load_apps(&r.choices)
                .into_iter()
                .map(|a| {
                    serde_json::json!({
                        "id": crate::desktop::portal_app_id(&a.id),
                        "name": a.name,
                        "icon": a.icon,
                    })
                })
                .collect();
            extra.insert("apps".into(), serde_json::Value::Array(apps));
        }
        PickerRequest::Account(_) => {
            extra.insert("user".into(), crate::paths::whoami().into());
            extra.insert("realName".into(), crate::paths::real_name().into());
            if let Some(img) = crate::paths::face_image() {
                extra.insert(
                    "image".into(),
                    img.to_string_lossy().into_owned().into(),
                );
            }
        }
        PickerRequest::Wallpaper { uri } => {
            extra.insert("uri".into(), uri.clone().into());
        }
        PickerRequest::FileChooser(r) => {
            let places: Vec<serde_json::Value> = crate::paths::places()
                .into_iter()
                .map(|(label, path)| {
                    serde_json::json!({
                        "label": label,
                        "path": path.to_string_lossy(),
                    })
                })
                .collect();
            extra.insert("places".into(), serde_json::Value::Array(places));
            let filters: Vec<serde_json::Value> = r
                .filters
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "label": f.label,
                        "globs": f.globs(),
                        "portal": serde_json::to_value(f.to_portal()).unwrap_or(serde_json::Value::Null),
                    })
                })
                .collect();
            extra.insert("filters".into(), serde_json::Value::Array(filters));
            extra.insert(
                "filterIndex".into(),
                serde_json::json!(r.current_filter.unwrap_or(0)),
            );
        }
        _ => {}
    }

    let kind = match &req {
        PickerRequest::FileChooser(_) => "FileChooser",
        PickerRequest::AppChooser(_) => "AppChooser",
        PickerRequest::Access(_) => "Access",
        PickerRequest::Account(_) => "Account",
        PickerRequest::Wallpaper { .. } => "Wallpaper",
        PickerRequest::Confirm { .. } => "Confirm",
        PickerRequest::Screenshot => "Screenshot",
    };

    let payload = serde_json::json!({
        "kind": kind,
        "request": req,
        "extra": extra,
        "replyFile": reply_file.to_string_lossy(),
        "doneFile": done_file.to_string_lossy(),
    });

    let output = tokio::process::Command::new("omarchy-shell")
        .args(["shell", "summon", "omarchy-portal", &payload.to_string()])
        .output()
        .await
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || stdout.contains("unknown") {
        tracing::warn!(
            "omarchy-shell summon omarchy-portal failed: status={:?} stdout={stdout}",
            output.status.code()
        );
        return None;
    }

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                let _ = tokio::process::Command::new("omarchy-shell")
                    .args(["-q", "shell", "hide", "omarchy-portal"])
                    .status()
                    .await;
                let _ = std::fs::remove_dir_all(&tmp);
                return None;
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                if done_file.exists() {
                    let bytes = std::fs::read(&reply_file).ok()?;
                    let _ = std::fs::remove_dir_all(&tmp);
                    return serde_json::from_slice(&bytes).ok();
                }
            }
        }
    }
}

pub async fn run(req: PickerRequest, token: CancellationToken) -> Option<PickerReply> {
    if let Some(reply) = run_via_shell(req.clone(), token.clone()).await {
        return Some(reply);
    }
    tracing::warn!("shell portal dialog unavailable, falling back to embedded picker");
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
