#!/usr/bin/env bash
# Installs gmpublisher-qt (the Linux-first rewrite) for the current user only (no root).
# It uses the Qt 6 and Kirigami libraries that KDE Plasma already ships on SteamOS and Bazzite.
#
# Run from inside the extracted release folder:  ./install.sh
# Pass --uninstall to remove.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ID="io.github.ffseestevanli_oss.gmpublisher"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
INSTALL_DIR="$DATA_HOME/gmpublisher-qt"
BIN="$HOME/.local/bin/gmpublisher-qt"
DESKTOP_FILE="$DATA_HOME/applications/$ID.desktop"
ICON_FILE="$DATA_HOME/icons/hicolor/128x128/apps/$ID.png"
MIME_FILE="$DATA_HOME/mime/packages/$ID.xml"
# Dolphin's right-click "Extract here" for .gma files
SERVICE_MENU="$DATA_HOME/kio/servicemenus/$ID.extract.desktop"
# ...and "Open as Garry's Mod addon" for folders
FOLDER_MENU="$DATA_HOME/kio/servicemenus/$ID.folder.desktop"
# Where earlier previews of this edition were installed
OLD_INSTALL_DIR="$DATA_HOME/gmpublisher-native"
OLD_BIN="$HOME/.local/bin/gmpublisher"
# The old Tauri gmpublisher AppImage (scripts/install-steamos.sh)
OLD_APPIMAGE_DESKTOP="$DATA_HOME/applications/gmpublisher.desktop"

refresh_caches() {
	update-desktop-database "$DATA_HOME/applications" >/dev/null 2>&1 || true
	update-mime-database "$DATA_HOME/mime" >/dev/null 2>&1 || true
}

remove_old_preview() {
	rm -rf "$OLD_INSTALL_DIR"
	# Only remove the old command if it pointed at the old preview, never the old AppImage
	if [ -L "$OLD_BIN" ] && [ "$(readlink "$OLD_BIN")" = "$OLD_INSTALL_DIR/gmpublisher" ]; then
		rm -f "$OLD_BIN"
	fi
}

if [ "${1:-}" = "--uninstall" ]; then
	rm -rf "$INSTALL_DIR" "$BIN" "$DESKTOP_FILE" "$ICON_FILE" "$MIME_FILE" "$SERVICE_MENU" "$FOLDER_MENU"
	remove_old_preview
	refresh_caches
	echo "gmpublisher-qt removed."
	exit 0
fi

kirigami_found=false
for dir in /usr/lib*/qt6/qml/org/kde/kirigami /usr/lib/*/qt6/qml/org/kde/kirigami; do
	[ -d "$dir" ] && kirigami_found=true
done
if [ "$kirigami_found" = false ]; then
	echo "Warning: Kirigami (Qt 6) wasn't found. It ships with KDE Plasma 6; on other desktops install your distro's kirigami / kf6-kirigami package." >&2
fi

remove_old_preview
mkdir -p "$INSTALL_DIR" "$(dirname "$BIN")" "$(dirname "$DESKTOP_FILE")" "$(dirname "$ICON_FILE")" "$(dirname "$MIME_FILE")" "$(dirname "$SERVICE_MENU")"
install -m755 "$HERE/gmpublisher-qt" "$INSTALL_DIR/gmpublisher-qt"
install -m644 "$HERE/libsteam_api.so" "$INSTALL_DIR/libsteam_api.so"
ln -sf "$INSTALL_DIR/gmpublisher-qt" "$BIN"
install -m644 "$HERE/$ID.png" "$ICON_FILE"
sed "s|^Exec=gmpublisher-qt|Exec=$INSTALL_DIR/gmpublisher-qt|" "$HERE/$ID.desktop" > "$DESKTOP_FILE"
install -m644 "$HERE/$ID.mime.xml" "$MIME_FILE"
# Plasma 6 only runs service menus that are marked executable
sed "s|^Exec=gmpublisher-qt|Exec=$INSTALL_DIR/gmpublisher-qt|" "$HERE/gmpublisher-servicemenu.desktop" > "$SERVICE_MENU"
sed "s|^Exec=gmpublisher-qt|Exec=$INSTALL_DIR/gmpublisher-qt|" "$HERE/gmpublisher-folder-servicemenu.desktop" > "$FOLDER_MENU"
chmod 755 "$SERVICE_MENU" "$FOLDER_MENU"
refresh_caches

echo "Installed. Launch gmpublisher-qt from your application menu, or run: $BIN"
echo ".gma files now open in gmpublisher-qt. In Dolphin, right-click a .gma to extract it, or a folder to open it as an addon."
echo "Run 'gmpublisher-qt --help' for the command-line tools."
if [ -f "$OLD_APPIMAGE_DESKTOP" ]; then
	echo
	echo "Note: the old gmpublisher AppImage is still installed and shows in your menu as plain \"gmpublisher\"."
	echo "The new app is the one called \"gmpublisher-qt\". To remove the old one: rm \"$OLD_APPIMAGE_DESKTOP\" and delete its AppImage."
fi
