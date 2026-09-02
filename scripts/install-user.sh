#!/usr/bin/env bash
# Install the portal backend for this user (no root required).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$(realpath -m "$HOME/.local/libexec/xdg-desktop-portal-omarchy")"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"

cd "$ROOT"
cargo build --release
install -Dm755 "$ROOT/target/release/xdg-desktop-portal-omarchy" "$BIN"

install -Dm644 "$ROOT/data/omarchy.portal" \
  "$DATA/xdg-desktop-portal/portals/omarchy.portal"

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

install -Dm644 "$ROOT/data/omarchy-portals.conf" \
  "$CONFIG/xdg-desktop-portal/hyprland-portals.conf"

XDPH="$CONFIG/hypr/xdph.conf"
mkdir -p "$CONFIG/hypr"
cat >"$XDPH" <<EOF
screencopy {
    allow_token_by_default = false
}
EOF

HYPR="$CONFIG/hypr/hyprland.lua"
if [[ -f $HYPR ]] && ! grep -Fq 'xdg-desktop-portal-omarchy' "$HYPR"; then
  cat >>"$HYPR" <<'EOF'

-- Portal dialogs (egui): float + opaque
o.window("xdg-desktop-portal-omarchy", { tag = "-default-opacity +floating-window", opacity = "1 1" })
EOF
fi

systemctl --user daemon-reload
systemctl --user enable --now xdg-desktop-portal-omarchy.service
systemctl --user restart xdg-desktop-portal.service xdg-desktop-portal-omarchy.service xdg-desktop-portal-hyprland.service 2>/dev/null || true

echo "Installed $BIN"
