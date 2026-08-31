#!/usr/bin/env bash
# Install the Omarchy portal backend for this user (no root required).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${XDG_DATA_HOME:-$HOME/.local/share}/../libexec/xdg-desktop-portal-omarchy"
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

PLUGIN_DST="$CONFIG/omarchy/plugins/omarchy-portal"
mkdir -p "$PLUGIN_DST"
cp -a "$ROOT/shell/omarchy.portal/." "$PLUGIN_DST/"

install -Dm755 "$ROOT/scripts/omarchy-share-picker" "$HOME/.local/bin/omarchy-share-picker"

python3 - <<'PY'
import json, os
path = os.path.expanduser("~/.config/omarchy/shell.json")
try:
    cfg = json.load(open(path))
except FileNotFoundError:
    raise SystemExit(0)
plugins = cfg.get("plugins") or []
ids = []
for p in plugins:
    ids.append(p.get("id") if isinstance(p, dict) else p)
if "omarchy-portal" not in ids:
    plugins.append({"id": "omarchy-portal"})
    cfg["plugins"] = plugins
    with open(path, "w") as f:
        json.dump(cfg, f, indent=2)
        f.write("\n")
PY

XDPH="$CONFIG/hypr/xdph.conf"
mkdir -p "$CONFIG/hypr"
if [[ -f $XDPH ]]; then
  if grep -q 'custom_picker_binary' "$XDPH"; then
    sed -i "s|custom_picker_binary *=.*|    custom_picker_binary = $HOME/.local/bin/omarchy-share-picker|" "$XDPH"
  else
    printf '\n    custom_picker_binary = %s\n' "$HOME/.local/bin/omarchy-share-picker" >>"$XDPH"
  fi
else
  cat >"$XDPH" <<EOF
screencopy {
    allow_token_by_default = true
    custom_picker_binary = $HOME/.local/bin/omarchy-share-picker
}
EOF
fi

HYPR="$CONFIG/hypr/hyprland.lua"
if [[ -f $HYPR ]] && ! grep -Fq 'Omarchy Portal' "$HYPR"; then
  cat >>"$HYPR" <<'EOF'

-- Portal dialogs: opaque popup surface so dark-theme text stays readable
o.window({ class = "^org.quickshell$", title = "^Omarchy Portal$" }, { tag = "-default-opacity", opacity = "1 1" })
o.window("xdg-desktop-portal-omarchy", { tag = "-default-opacity +floating-window", opacity = "1 1" })
EOF
fi

systemctl --user daemon-reload
systemctl --user enable --now xdg-desktop-portal-omarchy.service
systemctl --user restart xdg-desktop-portal.service xdg-desktop-portal-omarchy.service xdg-desktop-portal-hyprland.service 2>/dev/null || true
omarchy-shell -q shell rescanPlugins || true

echo "Installed $BIN"
echo "Shell plugin: $PLUGIN_DST"
echo "Share picker: $HOME/.local/bin/omarchy-share-picker"
