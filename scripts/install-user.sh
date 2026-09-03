#!/usr/bin/env bash
# User-local install for development (no root, no Hyprland/xdph writes).
# Routing needs XDG_CURRENT_DESKTOP to include Omarchy (Omarchy:Hyprland).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$(realpath -m "$HOME/.local/libexec/xdg-desktop-portal-omarchy")"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"

cd "$ROOT"
cargo build --release --locked --bins
install -Dm755 "$ROOT/target/release/xdg-desktop-portal-omarchy" "$BIN"

install -Dm644 "$ROOT/data/omarchy.portal" \
  "$DATA/xdg-desktop-portal/portals/omarchy.portal"
install -Dm644 "$ROOT/data/omarchy-portals.conf" \
  "$DATA/xdg-desktop-portal/omarchy-portals.conf"
install -Dm644 "$ROOT/data/omarchy-portals.conf" \
  "$CONFIG/xdg-desktop-portal/omarchy-portals.conf"

install -Dm644 /dev/stdin \
  "$DATA/dbus-1/services/org.freedesktop.impl.portal.desktop.omarchy.service" <<EOF
[D-BUS Service]
Name=org.freedesktop.impl.portal.desktop.omarchy
Exec=$BIN
SystemdService=xdg-desktop-portal-omarchy.service
EOF

install -Dm644 /dev/stdin \
  "$CONFIG/systemd/user/xdg-desktop-portal-omarchy.service" <<EOF
[Unit]
Description=Portal service (Omarchy implementation)
PartOf=graphical-session.target
After=graphical-session.target

[Service]
Type=dbus
BusName=org.freedesktop.impl.portal.desktop.omarchy
ExecStart=$BIN
Restart=on-failure

[Install]
WantedBy=graphical-session.target
EOF

systemctl --user daemon-reload
systemctl --user try-restart xdg-desktop-portal-omarchy.service 2>/dev/null || true
systemctl --user restart xdg-desktop-portal.service 2>/dev/null || true

echo "Installed $BIN"
echo "D-Bus activates the backend. Do not enable the user unit."
echo "Routing needs XDG_CURRENT_DESKTOP to include Omarchy (Omarchy:Hyprland)."
echo "Reload Hyprland or re-login if that value is not set yet."
