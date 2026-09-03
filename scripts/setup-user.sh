#!/usr/bin/env bash
# Reload portal services after install. Does not write session or Hyprland files.
# xdg-desktop-portal picks this backend from omarchy-portals.conf when
# XDG_CURRENT_DESKTOP includes Omarchy (Omarchy:Hyprland).
set -euo pipefail

systemctl --user daemon-reload 2>/dev/null || true
systemctl --user try-restart xdg-desktop-portal-omarchy.service 2>/dev/null || true
systemctl --user restart xdg-desktop-portal.service 2>/dev/null || true

desktop="${XDG_CURRENT_DESKTOP:-}"
echo "Reloaded xdg-desktop-portal."
if [[ $desktop == *Omarchy* || $desktop == *omarchy* ]]; then
  echo "XDG_CURRENT_DESKTOP=$desktop — omarchy-portals.conf should apply."
else
  echo "XDG_CURRENT_DESKTOP=$desktop"
  echo "Set it to Omarchy:Hyprland (Omarchy session default) and re-login, or reload Hyprland."
fi
