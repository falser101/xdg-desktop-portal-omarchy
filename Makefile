PREFIX ?= /usr
DESTDIR ?=
CARGO ?= cargo

.PHONY: build release install install-user uninstall-user install-system setup-user clean aur-srcinfo

build release:
	$(CARGO) build --release --locked --bins

install: install-system

install-system: release
	DESTDIR="$(DESTDIR)" PREFIX="$(PREFIX)" ./scripts/install-system.sh --skip-build

install-user: release
	./scripts/install-user.sh

setup-user:
	./scripts/setup-user.sh

clean:
	$(CARGO) clean

# Regenerate AUR .SRCINFO (requires makepkg on Arch)
aur-srcinfo:
	cd aur/xdg-desktop-portal-omarchy-git && makepkg --printsrcinfo > .SRCINFO
