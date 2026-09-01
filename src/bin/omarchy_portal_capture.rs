//! Capture a Hyprland window via `hyprland_toplevel_export_v1` (same protocol
//! path as hyprland-preview-share-picker). Avoids `grim -g` screen-region crops
//! that mix up overlapping windows.
//!
//! Usage:
//!   omarchy-portal-capture window <hyprctl-address> <out.png> [--size N]
//!
//! Address is `hyprctl clients -j` `.address` (0x…) or the decimal form from
//! `XDPH_WINDOW_SHARING_LIST`. The protocol uses the low 32 bits of the handle.

use std::path::PathBuf;

use anyhow::{bail, Context};
use hyprland_preview_share_picker_lib::frame::FrameManager;
use hyprland_preview_share_picker_lib::image::{Image, ImageKind};
use wayland_client::Connection;

fn main() {
    if let Err(err) = run() {
        eprintln!("omarchy-portal-capture: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(|s| s.as_str()) == Some("--help") || args.is_empty() {
        eprintln!(
            "Usage: omarchy-portal-capture window <address> <out.png> [--size N]\n\
             Capture one Hyprland toplevel via hyprland_toplevel_export_v1."
        );
        return Ok(());
    }

    let cmd = args.remove(0);
    match cmd.as_str() {
        "window" => {
            if args.len() < 2 {
                bail!("window requires <address> <out.png>");
            }
            let address = parse_address(&args[0])
                .with_context(|| format!("bad address {}", args[0]))?;
            let out = PathBuf::from(&args[1]);
            let mut size = 480u32;
            let mut i = 2;
            while i < args.len() {
                if args[i] == "--size" {
                    i += 1;
                    size = args
                        .get(i)
                        .ok_or_else(|| anyhow::anyhow!("--size needs a value"))?
                        .parse()
                        .context("--size")?;
                } else {
                    bail!("unknown argument {}", args[i]);
                }
                i += 1;
            }
            capture_window(address, &out, size)
        }
        other => bail!("unknown command {other} (only `window` is supported)"),
    }
}

fn parse_address(raw: &str) -> anyhow::Result<u64> {
    let a = raw.trim().to_ascii_lowercase();
    if a.is_empty() || a == "0" {
        bail!("empty address");
    }
    if let Some(hex) = a.strip_prefix("0x") {
        return u64::from_str_radix(hex, 16).context("hex address");
    }
    if a.chars().all(|c| c.is_ascii_digit()) {
        return a.parse::<u64>().context("decimal address");
    }
    u64::from_str_radix(&a, 16).context("hex address")
}

fn capture_window(address: u64, out: &PathBuf, fit: u32) -> anyhow::Result<()> {
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("mkdir {}", parent.display()))?;
        }
    }

    let connection = Connection::connect_to_env().context("connect to Wayland")?;
    let mut manager = FrameManager::new(&connection).context("FrameManager (toplevel export)")?;
    let buffer = manager
        .capture_frame(address)
        .with_context(|| format!("capture_frame({address:#x})"))?;
    let mut image = Image::new(buffer)
        .map_err(|e| anyhow::anyhow!("image from buffer: {e}"))?
        .into_rgb()
        .map_err(|e| anyhow::anyhow!("into_rgb: {e}"))?;
    if fit > 0 {
        image.resize_to_fit(fit);
    }

    match image.buffer {
        ImageKind::Rgb(rgb) => rgb
            .save(out)
            .with_context(|| format!("write {}", out.display()))?,
        ImageKind::Xrgb(_) => bail!("expected rgb image after into_rgb"),
    }
    manager.destroy();
    Ok(())
}
