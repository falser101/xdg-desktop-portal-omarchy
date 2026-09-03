#!/usr/bin/env bash
# Remove a previous per-user install from ./scripts/install-user.sh.
set -euo pipefail

CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"

systemctl --user disable --now xdg-desktop-portal-omarchy.service 2>/dev/null || true

rm -fv \
  "$HOME/.local/libexec/xdg-desktop-portal-omarchy" \
  "$DATA/xdg-desktop-portal/portals/omarchy.portal" \
  "$DATA/xdg-desktop-portal/omarchy-portals.conf" \
  "$DATA/dbus-1/services/org.freedesktop.impl.portal.desktop.omarchy.service" \
  "$CONFIG/systemd/user/xdg-desktop-portal-omarchy.service" \
  "$CONFIG/xdg-desktop-portal/omarchy-portals.conf"

rm -rf "$CONFIG/omarchy/plugins/omarchy-portal"

rmdir "$HOME/.local/libexec" 2>/dev/null || true
rmdir "$DATA/xdg-desktop-portal/portals" 2>/dev/null || true
rmdir "$DATA/xdg-desktop-portal" 2>/dev/null || true
rmdir "$DATA/dbus-1/services" 2>/dev/null || true
rmdir "$DATA/dbus-1" 2>/dev/null || true

# Drop the old setup-user.sh copy of omarchy-portals.conf if it is still the
# shipped file under the Hyprland name. Leave it if the user edited it.
legacy="$CONFIG/xdg-desktop-portal/hyprland-portals.conf"
shipped=/usr/share/xdg-desktop-portal/omarchy-portals.conf
if [[ -f $legacy ]]; then
  if [[ -f $shipped ]] && cmp -s "$legacy" "$shipped"; then
    rm -fv "$legacy"
  elif grep -Fq 'default=omarchy;hyprland;gtk' "$legacy" && grep -Fq 'Screenshot=omarchy' "$legacy"; then
    rm -fv "$legacy"
  fi
fi

python3 - <<'PY'
import json, os
path = os.path.expanduser("~/.config/omarchy/shell.json")
try:
    cfg = json.load(open(path))
except FileNotFoundError:
    raise SystemExit(0)
plugins = cfg.get("plugins") or []
new = [p for p in plugins if (p.get("id") if isinstance(p, dict) else p) != "omarchy-portal"]
if new != plugins:
    cfg["plugins"] = new
    with open(path, "w") as f:
        json.dump(cfg, f, indent=2)
        f.write("\n")
PY

systemctl --user daemon-reload 2>/dev/null || true
systemctl --user restart xdg-desktop-portal.service 2>/dev/null || true
echo "Removed user-level portal install."
