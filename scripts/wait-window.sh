#!/usr/bin/env bash
set -euo pipefail
pattern=${1:-xdg-desktop-portal-omarchy}
for _ in $(seq 1 50); do
  if hyprctl clients -j | python3 -c "
import json,sys
pat='$pattern'.lower()
for c in json.load(sys.stdin):
    blob=' '.join([str(c.get('class') or ''), str(c.get('title') or ''), str(c.get('initialClass') or '')]).lower()
    if pat in blob:
        print(c.get('class','')+'|'+c.get('title','')+'|'+str(c.get('at'))+'|'+str(c.get('size')))
        sys.exit(0)
sys.exit(1)
"; then
    exit 0
  fi
  sleep 0.1
done
echo "window not found: $pattern" >&2
exit 1
