#!/usr/bin/env bash
# Remove a previous per-user install from ./scripts/install-user.sh so it
# does not shadow a system (/usr) package.
set -euo pipefail

CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"

systemctl --user disable --now xdg-desktop-portal-omarchy.service 2>/dev/null || true

rm -fv \
  "$HOME/.local/libexec/xdg-desktop-portal-omarchy" \
  "$HOME/.local/libexec/omarchy-portal-capture" \
  "$HOME/.local/bin/omarchy-share-picker" \
  "$DATA/xdg-desktop-portal/portals/omarchy.portal" \
  "$DATA/dbus-1/services/org.freedesktop.impl.portal.desktop.omarchy.service" \
  "$CONFIG/systemd/user/xdg-desktop-portal-omarchy.service"

rmdir "$HOME/.local/libexec" 2>/dev/null || true
rmdir "$DATA/xdg-desktop-portal/portals" 2>/dev/null || true
rmdir "$DATA/xdg-desktop-portal" 2>/dev/null || true
rmdir "$DATA/dbus-1/services" 2>/dev/null || true
rmdir "$DATA/dbus-1" 2>/dev/null || true

systemctl --user daemon-reload 2>/dev/null || true

cat <<EOF
Removed user-level portal binaries, D-Bus service, and systemd unit.

Left in place (session config / UI — re-run setup after system install):
  $CONFIG/xdg-desktop-portal/hyprland-portals.conf
  $CONFIG/hypr/xdph.conf
  $CONFIG/omarchy/plugins/omarchy-portal/

Next:
  sudo ./scripts/install-system.sh
  xdg-desktop-portal-omarchy-setup
EOF
