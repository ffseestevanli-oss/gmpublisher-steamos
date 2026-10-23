#!/usr/bin/env bash
# Installs gmpublisher for the current user only. No root, no system packages,
# so it works on SteamOS, Bazzite and other immutable distros.
#
#   curl -fsSL https://raw.githubusercontent.com/ffseestevanli-oss/gmpublisher-steamos/master/scripts/install-steamos.sh | bash
#
# Run it again to update. Pass --uninstall to remove.
set -euo pipefail

REPO="ffseestevanli-oss/gmpublisher-steamos"
APP_DIR="$HOME/Applications"
APPIMAGE="$APP_DIR/gmpublisher.AppImage"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
DESKTOP_FILE="$DATA_HOME/applications/gmpublisher.desktop"
ICON_FILE="$DATA_HOME/icons/hicolor/128x128/apps/gmpublisher.png"

if [ "${1:-}" = "--uninstall" ]; then
	rm -f "$APPIMAGE" "$DESKTOP_FILE" "$ICON_FILE"
	echo "gmpublisher removed. Your settings are kept in $DATA_HOME/gmpublisher and ~/.config/gmpublisher."
	exit 0
fi

echo "Finding the latest release of $REPO..."
URL=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
	| grep -o '"browser_download_url": *"[^"]*\.AppImage"' \
	| head -n1 | cut -d'"' -f4)

if [ -z "$URL" ]; then
	echo "No AppImage found in the latest release of $REPO." >&2
	exit 1
fi

mkdir -p "$APP_DIR" "$(dirname "$DESKTOP_FILE")" "$(dirname "$ICON_FILE")"

echo "Downloading $URL"
curl -fL --progress-bar -o "$APPIMAGE.part" "$URL"
chmod +x "$APPIMAGE.part"
mv "$APPIMAGE.part" "$APPIMAGE"

curl -fsSL -o "$ICON_FILE" "https://raw.githubusercontent.com/$REPO/master/src-tauri/icons/128x128.png" || true

cat > "$DESKTOP_FILE" <<DESKTOP
[Desktop Entry]
Name=gmpublisher
Comment=Garry's Mod Workshop publishing utility
Exec="$APPIMAGE" %F
Icon=gmpublisher
Type=Application
Categories=Utility;Game;
Terminal=false
DESKTOP

update-desktop-database "$(dirname "$DESKTOP_FILE")" >/dev/null 2>&1 || true

echo
echo "Installed to $APPIMAGE"
echo "Launch it from your application menu (Desktop Mode on the Steam Deck). Steam must be running."
