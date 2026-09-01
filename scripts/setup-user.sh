#!/usr/bin/env bash
# Wire a system-installed xdg-desktop-portal-omarchy into the current user session.
# Safe to re-run. Does not need root.
set -euo pipefail

DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"

SHARE_SRC="/usr/share/xdg-desktop-portal-omarchy/omarchy.portal"
PORTALS_SRC="/usr/share/xdg-desktop-portal/omarchy-portals.conf"
PICKER_BIN="/usr/bin/omarchy-share-picker"
CAPTURE_HINT="/usr/lib/omarchy-portal-capture"

# Prefer packaged share tree; fall back to repo layout for developers.
if [[ ! -d $SHARE_SRC ]]; then
  ROOT="$(cd "$(dirname "$0")/.." && pwd)"
  if [[ -d $ROOT/shell/omarchy.portal ]]; then
    SHARE_SRC="$ROOT/shell/omarchy.portal"
  fi
fi
if [[ ! -f $PORTALS_SRC ]]; then
  ROOT="${ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
  [[ -f $ROOT/data/omarchy-portals.conf ]] && PORTALS_SRC="$ROOT/data/omarchy-portals.conf"
fi
if [[ ! -x $PICKER_BIN ]]; then
  PICKER_BIN="$HOME/.local/bin/omarchy-share-picker"
fi

if [[ ! -d $SHARE_SRC ]]; then
  echo "error: Omarchy portal QML not found at /usr/share/xdg-desktop-portal-omarchy/omarchy.portal" >&2
  echo "Install the package (or run from a git checkout) first." >&2
  exit 1
fi

PLUGIN_DST="$CONFIG/omarchy/plugins/omarchy-portal"
mkdir -p "$PLUGIN_DST"
cp -a "$SHARE_SRC"/. "$PLUGIN_DST/"

# Ensure shell.json enables the plugin
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
    print("Added omarchy-portal to ~/.config/omarchy/shell.json")
else:
    print("omarchy-portal already listed in shell.json")
PY

# Portal routing for Hyprland sessions (Omarchy uses XDG_CURRENT_DESKTOP=Hyprland)
if [[ -f $PORTALS_SRC ]]; then
  install -Dm644 "$PORTALS_SRC" "$CONFIG/xdg-desktop-portal/hyprland-portals.conf"
  echo "Wrote $CONFIG/xdg-desktop-portal/hyprland-portals.conf"
fi

# xdph share picker
XDPH="$CONFIG/hypr/xdph.conf"
mkdir -p "$CONFIG/hypr"
cat >"$XDPH" <<EOF
screencopy {
    allow_token_by_default = false
    custom_picker_binary = $PICKER_BIN
}
EOF
echo "Wrote $XDPH → custom_picker_binary = $PICKER_BIN"

# Hyprland window rules for portal dialogs (lua config)
HYPR="$CONFIG/hypr/hyprland.lua"
if [[ -f $HYPR ]] && ! grep -Fq 'Omarchy Portal' "$HYPR"; then
  cat >>"$HYPR" <<'EOF'

-- Portal dialogs: opaque popup surface so dark-theme text stays readable
o.window({ class = "^org.quickshell$", title = "^(Omarchy Portal|User Information Requested|Background Activity|Launcher Requested)$" }, { float = true, center = true, tag = "-default-opacity", opacity = "1 1" })
o.window("xdg-desktop-portal-omarchy", { tag = "-default-opacity +floating-window", opacity = "1 1" })
EOF
  echo "Appended portal window rules to $HYPR"
fi

systemctl --user daemon-reload 2>/dev/null || true
systemctl --user enable --now xdg-desktop-portal-omarchy.service 2>/dev/null || true
systemctl --user restart xdg-desktop-portal.service xdg-desktop-portal-omarchy.service xdg-desktop-portal-hyprland.service 2>/dev/null || true
command -v omarchy-shell >/dev/null && omarchy-shell -q shell rescanPlugins 2>/dev/null || true
command -v omarchy >/dev/null && omarchy restart shell 2>/dev/null || true

echo
echo "User setup done."
echo "  Plugin:  $PLUGIN_DST"
echo "  Capture: $CAPTURE_HINT (used by share picker)"
echo "Test: open a file dialog or share screen from a browser/OBS."
