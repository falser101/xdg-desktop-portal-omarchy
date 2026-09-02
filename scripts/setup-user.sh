#!/usr/bin/env bash
# Wire a system-installed xdg-desktop-portal-omarchy into this user session.
set -euo pipefail

CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
PORTALS_SRC="/usr/share/xdg-desktop-portal/omarchy-portals.conf"

if [[ ! -f $PORTALS_SRC ]]; then
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
  [[ -f $ROOT/data/omarchy-portals.conf ]] && PORTALS_SRC="$ROOT/data/omarchy-portals.conf"
fi

if [[ -f $PORTALS_SRC ]]; then
  install -Dm644 "$PORTALS_SRC" "$CONFIG/xdg-desktop-portal/hyprland-portals.conf"
  echo "Wrote $CONFIG/xdg-desktop-portal/hyprland-portals.conf"
fi

mkdir -p "$CONFIG/hypr"
cat >"$CONFIG/hypr/xdph.conf" <<'EOF'
screencopy {
    allow_token_by_default = false
    custom_picker_binary = omarchy-share-picker
}
EOF
echo "Wrote $CONFIG/hypr/xdph.conf"

HYPR="$CONFIG/hypr/hyprland.lua"
if [[ -f $HYPR ]] && ! grep -Fq 'xdg-desktop-portal-omarchy' "$HYPR"; then
  cat >>"$HYPR" <<'EOF'

-- Portal dialogs (egui): float + opaque
o.window("xdg-desktop-portal-omarchy", { tag = "-default-opacity +floating-window", opacity = "1 1" })
EOF
  echo "Appended portal window rules to $HYPR"
fi

systemctl --user daemon-reload 2>/dev/null || true
systemctl --user enable --now xdg-desktop-portal-omarchy.service 2>/dev/null || true
systemctl --user restart xdg-desktop-portal.service xdg-desktop-portal-omarchy.service xdg-desktop-portal-hyprland.service 2>/dev/null || true

echo "User setup done. Dialogs: built-in egui picker."
