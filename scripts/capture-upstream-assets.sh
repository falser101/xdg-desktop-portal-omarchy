#!/bin/bash
# Capture discussion / packaging demo stills for path B upstream proposal.
# Saves under docs/upstream/assets/ (git-friendly PNGs).
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
ASSETS="$ROOT/docs/upstream/assets"
mkdir -p "$ASSETS"
LOG=/tmp/omarchy-upstream-capture.log
: >"$LOG"

portal_call() {
  python3 "$ROOT/scripts/portal-call.py" "$@"
}

find_portal_geom() {
  hyprctl -j clients | python3 -c '
import json, sys
for c in json.load(sys.stdin):
    title = str(c.get("title") or "")
    klass = str(c.get("class") or "")
    if title == "Omarchy Portal" or klass == "xdg-desktop-portal-omarchy":
        x, y = c["at"]
        w, h = c["size"]
        print(c["address"], x, y, w, h, klass.replace(" ", "_"), title.replace(" ", "_"))
        break
'
}

wait_portal() {
  local tries=${1:-80}
  local win=""
  for _ in $(seq 1 "$tries"); do
    win=$(find_portal_geom || true)
    if [[ -n $win ]]; then
      printf '%s\n' "$win"
      return 0
    fi
    sleep 0.1
  done
  return 1
}

shot_geom() {
  local dest=$1 x=$2 y=$3 w=$4 h=$5
  # pad slightly so chrome is not clipped
  local px=$((x > 8 ? x - 8 : 0))
  local py=$((y > 8 ? y - 8 : 0))
  local pw=$((w + 16))
  local ph=$((h + 16))
  grim -g "${px},${py} ${pw}x${ph}" "$dest"
}

dismiss_portal() {
  # Omarchy's hyprctl Lua shim breaks `dispatch … address:0x…`; Escape + killing
  # the portal caller is enough to tear the Quickshell dialog down.
  wtype -k Escape 2>/dev/null || true
  sleep 0.2
  wtype -k Escape 2>/dev/null || true
  sleep 0.3
}

capture_portal_kind() {
  local kind=$1
  local dest=$ASSETS/${kind}.png
  local full=$ASSETS/${kind}-fullscreen.png
  echo "==> portal $kind" | tee -a "$LOG"
  dismiss_portal
  portal_call "$kind" --timeout 45000 >>"$LOG" 2>&1 &
  local pid=$!
  local win
  if ! win=$(wait_portal 100); then
    echo "FAIL: $kind dialog never appeared" | tee -a "$LOG"
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    return 1
  fi
  read -r addr x y w h klass title <<<"$win"
  echo "WINDOW $kind $klass $title ${w}x${h} @ ${x},${y}" | tee -a "$LOG"
  hyprctl dispatch focuswindow "address:$addr" >/dev/null || true
  # let live previews / layout settle
  sleep 0.8
  shot_geom "$dest" "$x" "$y" "$w" "$h"
  grim "$full"
  echo "SHOT $dest" | tee -a "$LOG"
  dismiss_portal
  wait "$pid" 2>/dev/null || true
}

xdph_window_list() {
  python3 - <<'PY'
import json, subprocess
clients = json.loads(subprocess.check_output(["hyprctl", "-j", "clients"], text=True))
parts = []
for i, c in enumerate(clients):
    title = str(c.get("title") or "")
    if title == "Omarchy Portal":
        continue
    addr = str(c.get("address") or "0")
    if addr.startswith("0x"):
        try:
            addr = str(int(addr, 16))
        except ValueError:
            pass
    klass = str(c.get("class") or "").replace("[", "").replace("]", "")
    title = title.replace("[", "").replace("]", "")
    parts.append(f"{i}[HC>]{klass}[HT>]{title}[HE>]{addr}[HA>]")
print("".join(parts))
PY
}

capture_share_picker() {
  local dest=$ASSETS/share-picker.png
  local full=$ASSETS/share-picker-fullscreen.png
  echo "==> share-picker" | tee -a "$LOG"
  dismiss_portal
  local list
  list=$(xdph_window_list)
  XDPH_WINDOW_SHARING_LIST="$list" /usr/bin/omarchy-share-picker --allow-token >>"$LOG" 2>&1 &
  local pid=$!
  local win
  if ! win=$(wait_portal 120); then
    echo "FAIL: share picker never appeared" | tee -a "$LOG"
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    return 1
  fi
  read -r addr x y w h klass title <<<"$win"
  echo "WINDOW share-picker $klass $title ${w}x${h} @ ${x},${y}" | tee -a "$LOG"
  hyprctl dispatch focuswindow "address:$addr" >/dev/null || true
  # live ScreencopyView needs a moment
  sleep 1.2
  # refresh geom in case it resized
  win=$(find_portal_geom || printf '%s\n' "$win")
  read -r addr x y w h klass title <<<"$win"
  shot_geom "$dest" "$x" "$y" "$w" "$h"
  grim "$full"
  echo "SHOT $dest" | tee -a "$LOG"
  dismiss_portal
  wait "$pid" 2>/dev/null || true
}

main() {
  echo "Assets -> $ASSETS" | tee -a "$LOG"
  capture_portal_kind open || true
  capture_portal_kind save || true
  capture_portal_kind screenshot || true
  capture_portal_kind access || true
  capture_portal_kind app-chooser || true
  capture_share_picker || true

  echo
  echo "Captured:"
  ls -lh "$ASSETS"/*.png 2>/dev/null || true
}

main "$@"
