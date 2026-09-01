#!/usr/bin/env bash
# System-wide install for packaging / PREFIX=/usr (no $HOME writes).
# Usage:
#   ./scripts/install-system.sh                  # build + install to /usr (needs root)
#   DESTDIR=/tmp/pkg PREFIX=/usr ./scripts/install-system.sh
#   ./scripts/install-system.sh --skip-build     # install already-built release binaries
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PREFIX="${PREFIX:-/usr}"
DESTDIR="${DESTDIR:-}"
SKIP_BUILD=false
for a in "$@"; do
  case "$a" in
  --skip-build) SKIP_BUILD=true ;;
  --help | -h)
    sed -n '2,8p' "$0"
    exit 0
    ;;
  esac
done

# Arch / FHS: portal backends live under /usr/lib
LIBDIR="${LIBDIR:-$PREFIX/lib}"
BINDIR="${BINDIR:-$PREFIX/bin}"
DATADIR="${DATADIR:-$PREFIX/share}"
UNITDIR="${UNITDIR:-$PREFIX/lib/systemd/user}"

cd "$ROOT"
if [[ $SKIP_BUILD != true ]]; then
  cargo build --release --locked --bins
fi

install -Dm755 "$ROOT/target/release/xdg-desktop-portal-omarchy" \
  "${DESTDIR}${LIBDIR}/xdg-desktop-portal-omarchy"
install -Dm755 "$ROOT/target/release/omarchy-portal-capture" \
  "${DESTDIR}${LIBDIR}/omarchy-portal-capture"
install -Dm755 "$ROOT/scripts/omarchy-share-picker" \
  "${DESTDIR}${BINDIR}/omarchy-share-picker"
install -Dm755 "$ROOT/scripts/setup-user.sh" \
  "${DESTDIR}${BINDIR}/xdg-desktop-portal-omarchy-setup"

install -Dm644 "$ROOT/data/omarchy.portal" \
  "${DESTDIR}${DATADIR}/xdg-desktop-portal/portals/omarchy.portal"
install -Dm644 "$ROOT/data/omarchy-portals.conf" \
  "${DESTDIR}${DATADIR}/xdg-desktop-portal/omarchy-portals.conf"

# Keep unit + D-Bus Exec in sync with LIBDIR
install -Dm644 /dev/stdin \
  "${DESTDIR}${UNITDIR}/xdg-desktop-portal-omarchy.service" <<EOF
[Unit]
Description=Portal service (Omarchy implementation)
PartOf=graphical-session.target
After=graphical-session.target

[Service]
Type=dbus
BusName=org.freedesktop.impl.portal.desktop.omarchy
ExecStart=${LIBDIR}/xdg-desktop-portal-omarchy
Restart=on-failure

[Install]
WantedBy=graphical-session.target
EOF

install -Dm644 /dev/stdin \
  "${DESTDIR}${DATADIR}/dbus-1/services/org.freedesktop.impl.portal.desktop.omarchy.service" <<EOF
[D-BUS Service]
Name=org.freedesktop.impl.portal.desktop.omarchy
Exec=${LIBDIR}/xdg-desktop-portal-omarchy
SystemdService=xdg-desktop-portal-omarchy.service
EOF

install -d "${DESTDIR}${DATADIR}/xdg-desktop-portal-omarchy"
cp -a "$ROOT/shell/omarchy.portal" \
  "${DESTDIR}${DATADIR}/xdg-desktop-portal-omarchy/"

install -Dm644 "$ROOT/LICENSE" \
  "${DESTDIR}${DATADIR}/licenses/xdg-desktop-portal-omarchy/LICENSE"

if [[ -z $DESTDIR && $(id -u) -eq 0 ]]; then
  echo "Installed system files under ${PREFIX}"
  echo "Each user should run: xdg-desktop-portal-omarchy-setup"
elif [[ -z $DESTDIR ]]; then
  echo "Installed under ${PREFIX} (DESTDIR empty)."
  echo "If this was not root, ensure you have write permission."
  echo "Each user should run: xdg-desktop-portal-omarchy-setup"
else
  echo "Staged under DESTDIR=${DESTDIR} PREFIX=${PREFIX}"
fi
