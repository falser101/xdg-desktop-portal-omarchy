use super::PortalCtx;
use crate::dict::{self, Options};
use crate::picker::{self, PickerReply, PickerRequest};
use crate::request::with_request;
use crate::response::PortalResponse;
use crate::uri::file_uri;
use zbus::zvariant::{ObjectPath, SerializeDict, Type};

pub struct Screenshot(pub PortalCtx);

const TARGET_SCREEN: u32 = 1;
const TARGET_AREA: u32 = 4;
const TARGET_ACTIVE: u32 = 8;

#[derive(SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct ShotOut {
    uri: String,
}

#[derive(SerializeDict, Type)]
#[zvariant(signature = "dict")]
struct ColorOut {
    color: (f64, f64, f64),
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Screenshot")]
impl Screenshot {
    async fn screenshot(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        options: Options,
    ) -> PortalResponse<ShotOut> {
        tracing::info!(app_id, "Screenshot.Screenshot");
        let interactive = dict::bool_or(&options, "interactive", false);
        let requested_target = dict::as_u32(&options, "target");
        with_request(&self.0.connection, &handle, |token| async move {
            let target = if interactive {
                match picker::run(PickerRequest::Screenshot, token).await {
                    Some(PickerReply::Screenshot { target }) if target != 0 => target,
                    _ => return PortalResponse::Cancelled,
                }
            } else {
                requested_target.unwrap_or(TARGET_SCREEN)
            };
            match capture_png(target).await {
                Some(uri) => PortalResponse::Success(ShotOut { uri }),
                None => PortalResponse::Other,
            }
        })
        .await
    }

    async fn pick_color(
        &self,
        handle: ObjectPath<'_>,
        app_id: &str,
        _parent_window: &str,
        _options: Options,
    ) -> PortalResponse<ColorOut> {
        tracing::info!(app_id, "Screenshot.PickColor");
        with_request(&self.0.connection, &handle, |_token| async move {
            match pick_color().await {
                Some(color) => PortalResponse::Success(ColorOut { color }),
                None => PortalResponse::Cancelled,
            }
        })
        .await
    }

    #[zbus(property, name = "AvailableTargets")]
    fn available_targets(&self) -> u32 {
        TARGET_SCREEN | TARGET_AREA | TARGET_ACTIVE
    }

    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        3
    }
}

async fn capture_png(target: u32) -> Option<String> {
    let pictures = crate::paths::user_dir("PICTURES", "Pictures");
    let _ = std::fs::create_dir_all(&pictures);
    let name = format!(
        "Screenshot_{}.png",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let path = pictures.join(name);

    let mut cmd = tokio::process::Command::new("grim");
    match target {
        TARGET_AREA => {
            let geo = slurp_geo().await?;
            cmd.arg("-g").arg(geo);
        }
        TARGET_ACTIVE => {
            let geo = active_window_geo().await?;
            cmd.arg("-g").arg(geo);
        }
        _ => {}
    }
    cmd.arg(&path);
    let status = cmd.status().await.ok()?;
    if !status.success() || !path.is_file() {
        return None;
    }
    file_uri(&path)
}

async fn slurp_geo() -> Option<String> {
    let out = tokio::process::Command::new("slurp")
        .args(["-f", "%x,%y %wx%h"])
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let geo = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if geo.is_empty() {
        None
    } else {
        Some(geo)
    }
}

async fn active_window_geo() -> Option<String> {
    let out = tokio::process::Command::new("hyprctl")
        .args(["-j", "activewindow"])
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let at = v.get("at")?.as_array()?;
    let size = v.get("size")?.as_array()?;
    let x = at.first()?.as_i64()?;
    let y = at.get(1)?.as_i64()?;
    let w = size.first()?.as_i64()?;
    let h = size.get(1)?.as_i64()?;
    Some(format!("{x},{y} {w}x{h}"))
}

async fn pick_color() -> Option<(f64, f64, f64)> {
    let out = tokio::process::Command::new("hyprpicker")
        .args(["-n", "-f", "hex"])
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_hex_color(&String::from_utf8_lossy(&out.stdout))
}

pub(crate) fn parse_hex_color(s: &str) -> Option<(f64, f64, f64)> {
    let hex = s.trim().trim_start_matches('#');
    if hex.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
    Some((r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hash_hex() {
        let (r, g, b) = parse_hex_color("#ff8000").unwrap();
        assert!((r - 1.0).abs() < f64::EPSILON);
        assert!((g - 128.0 / 255.0).abs() < 1e-9);
        assert!(b.abs() < f64::EPSILON);
    }

    #[test]
    fn parses_bare_hex() {
        let (r, _, _) = parse_hex_color("00ff00\n").unwrap();
        assert!(r.abs() < f64::EPSILON);
    }
}
