#!/usr/bin/env bash
set -euo pipefail
kind=${1:-open}
keys=${2-}  # e.g. Escape or Return
shots=/tmp/omarchy-portal-shots
mkdir -p "$shots"
log=/tmp/omarchy-portal-call.log
: >"$log"

python3 /home/falser/Projects/xdg-desktop-portal-omarchy/scripts/portal-call.py "$kind" --timeout 45000 >"$log" 2>&1 &
pid=$!

win=""
for i in $(seq 1 80); do
  win=$(hyprctl -j clients | python3 -c '
import json,sys
for c in json.load(sys.stdin):
    if c.get("class")=="xdg-desktop-portal-omarchy":
        print(c["address"], c["at"][0], c["at"][1], c["size"][0], c["size"][1], c.get("title","").replace(" ","_"))
        break
' || true)
  if [[ -n $win ]]; then
    break
  fi
  sleep 0.1
done

if [[ -z $win ]]; then
  echo "FAIL: window never appeared"
  wait "$pid" || true
  cat "$log"
  exit 1
fi

read -r addr x y w h title <<<"$win"
echo "WINDOW $addr ${w}x${h} @ ${x},${y} title=$title"

# Focus by address (Hyprland lua wrapper dislikes class: filters)
hyprctl dispatch focuswindow "address:$addr" >/dev/null || true
sleep 0.2

shot="$shots/${kind}-dialog.png"
if grim -g "${x},${y} ${w}x${h}" "$shot" 2>/dev/null; then
  echo "SHOT $shot"
else
  grim "$shot"
  echo "SHOT_FULL $shot"
fi
grim "$shots/${kind}-fullscreen.png"

if [[ -n $keys ]]; then
  sleep 0.2
  # shellcheck disable=SC2086
  wtype $keys || true
fi

wait "$pid" || true
echo "RESULT $(tail -n 1 "$log")"
cat "$log"
