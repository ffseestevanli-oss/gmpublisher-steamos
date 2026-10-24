#!/usr/bin/env bash
# Installs gmpublisher inside an Ubuntu 22.04 Distrobox, for the current user only.
# No root and no changes to the read-only system, so it works on Bazzite, SteamOS and Fedora Atomic.
#
# gmpublisher is built on Tauri 1 / WebKitGTK 4.0, which only Ubuntu 22.04-era systems still ship.
# Running it in a matching container avoids the library clashes a bundled AppImage hits on newer systems.
#
#   curl -fsSL https://raw.githubusercontent.com/ffseestevanli-oss/gmpublisher-steamos/master/scripts/install-distrobox.sh | bash
#
# Run it again to update. Pass --uninstall to remove.
# Set GMPUBLISHER_TARBALL=/path/to/gmpublisher_linux_x86_64.tar.gz to install a local build instead of the latest release.
set -euo pipefail

REPO="ffseestevanli-oss/gmpublisher-steamos"
BOX="gmpublisher"
IMAGE="docker.io/library/ubuntu:22.04"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
INSTALL_DIR="$DATA_HOME/gmpublisher-app"
DESKTOP_FILE="$DATA_HOME/applications/gmpublisher.desktop"
ICON_FILE="$DATA_HOME/icons/hicolor/128x128/apps/gmpublisher.png"
LAUNCHER="$HOME/.local/bin/gmpublisher"

if [ "${1:-}" = "--uninstall" ]; then
	distrobox rm -f "$BOX" 2>/dev/null || true
	rm -rf "$INSTALL_DIR" "$DESKTOP_FILE" "$ICON_FILE" "$LAUNCHER"
	echo "gmpublisher removed. Your settings are kept in $DATA_HOME/gmpublisher and ~/.config/gmpublisher."
	exit 0
fi

if ! command -v distrobox >/dev/null; then
	echo "distrobox is not installed. It ships with Bazzite and SteamOS 3.5+; on other systems see https://distrobox.it" >&2
	exit 1
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

if [ -n "${GMPUBLISHER_TARBALL:-}" ]; then
	cp "$GMPUBLISHER_TARBALL" "$TMP/gmpublisher.tar.gz"
else
	echo "Finding the latest release of $REPO..."
	URL=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
		| grep -o '"browser_download_url": *"[^"]*linux_x86_64\.tar\.gz"' \
		| head -n1 | cut -d'"' -f4)
	if [ -z "$URL" ]; then
		echo "No Linux build found in the latest release of $REPO." >&2
		exit 1
	fi
	echo "Downloading $URL"
	curl -fL --progress-bar -o "$TMP/gmpublisher.tar.gz" "$URL"
fi

rm -rf "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR" "$(dirname "$DESKTOP_FILE")" "$(dirname "$ICON_FILE")" "$(dirname "$LAUNCHER")"
tar -xzf "$TMP/gmpublisher.tar.gz" -C "$INSTALL_DIR"

if ! distrobox list 2>/dev/null | grep -qw "$BOX"; then
	echo "Creating the $BOX container (first run downloads Ubuntu 22.04, about 30 MB)..."
	distrobox create --yes --name "$BOX" --image "$IMAGE"
fi

echo "Installing gmpublisher's libraries inside the container (this can take a few minutes the first time)..."
distrobox enter "$BOX" -- sudo sh -c 'export DEBIAN_FRONTEND=noninteractive
	apt-get update -qq
	apt-get install -y -qq libwebkit2gtk-4.0-37 libgtk-3-0 libayatana-appindicator3-1 librsvg2-common \
		gstreamer1.0-plugins-good xdg-utils dbus-x11 fonts-noto-core > /dev/null'

cat > "$LAUNCHER" <<LAUNCH
#!/bin/sh
exec distrobox enter $BOX -- "$INSTALL_DIR/gmpublisher" "\$@"
LAUNCH
chmod +x "$LAUNCHER"

cp "$INSTALL_DIR/gmpublisher.png" "$ICON_FILE" 2>/dev/null || true

cat > "$DESKTOP_FILE" <<DESKTOP
[Desktop Entry]
Name=gmpublisher
Comment=Garry's Mod Workshop publishing utility
Exec=$LAUNCHER
Icon=gmpublisher
Type=Application
Categories=Utility;Game;
Terminal=false
DESKTOP

update-desktop-database "$(dirname "$DESKTOP_FILE")" >/dev/null 2>&1 || true

echo
echo "Installed. Launch gmpublisher from your application menu, or run: $LAUNCHER"
echo "Steam must be running."
