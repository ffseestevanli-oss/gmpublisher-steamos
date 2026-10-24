#!/usr/bin/env bash
# Installs the native Qt edition of gmpublisher for the current user only (no root).
# It uses the Qt 6 and Kirigami libraries that KDE Plasma already ships on SteamOS and Bazzite.
#
# Run from inside the extracted release folder:  ./install.sh
# Pass --uninstall to remove.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ID="io.github.ffseestevanli_oss.gmpublisher"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
INSTALL_DIR="$DATA_HOME/gmpublisher-native"
BIN="$HOME/.local/bin/gmpublisher"
DESKTOP_FILE="$DATA_HOME/applications/$ID.desktop"
ICON_FILE="$DATA_HOME/icons/hicolor/128x128/apps/$ID.png"
MIME_FILE="$DATA_HOME/mime/packages/$ID.xml"
# Dolphin's right-click "Extract here" for .gma files
SERVICE_MENU="$DATA_HOME/kio/servicemenus/$ID.extract.desktop"
# ...and "Open as Garry's Mod addon" for folders
FOLDER_MENU="$DATA_HOME/kio/servicemenus/$ID.folder.desktop"

refresh_caches() {
	update-desktop-database "$DATA_HOME/applications" >/dev/null 2>&1 || true
	update-mime-database "$DATA_HOME/mime" >/dev/null 2>&1 || true
}

if [ "${1:-}" = "--uninstall" ]; then
	rm -rf "$INSTALL_DIR" "$BIN" "$DESKTOP_FILE" "$ICON_FILE" "$MIME_FILE" "$SERVICE_MENU" "$FOLDER_MENU"
	refresh_caches
	echo "gmpublisher removed."
	exit 0
fi

kirigami_found=false
for dir in /usr/lib*/qt6/qml/org/kde/kirigami /usr/lib/*/qt6/qml/org/kde/kirigami; do
	[ -d "$dir" ] && kirigami_found=true
done
if [ "$kirigami_found" = false ]; then
	echo "Warning: Kirigami (Qt 6) wasn't found. It ships with KDE Plasma 6; on other desktops install your distro's kirigami / kf6-kirigami package." >&2
fi

mkdir -p "$INSTALL_DIR" "$(dirname "$BIN")" "$(dirname "$DESKTOP_FILE")" "$(dirname "$ICON_FILE")" "$(dirname "$MIME_FILE")" "$(dirname "$SERVICE_MENU")"
install -m755 "$HERE/gmpublisher" "$INSTALL_DIR/gmpublisher"
install -m644 "$HERE/libsteam_api.so" "$INSTALL_DIR/libsteam_api.so"
ln -sf "$INSTALL_DIR/gmpublisher" "$BIN"
install -m644 "$HERE/$ID.png" "$ICON_FILE"
sed "s|^Exec=gmpublisher|Exec=$INSTALL_DIR/gmpublisher|" "$HERE/$ID.desktop" > "$DESKTOP_FILE"
install -m644 "$HERE/$ID.mime.xml" "$MIME_FILE"
# Plasma 6 only runs service menus that are marked executable
sed "s|^Exec=gmpublisher|Exec=$INSTALL_DIR/gmpublisher|" "$HERE/gmpublisher-servicemenu.desktop" > "$SERVICE_MENU"
sed "s|^Exec=gmpublisher|Exec=$INSTALL_DIR/gmpublisher|" "$HERE/gmpublisher-folder-servicemenu.desktop" > "$FOLDER_MENU"
chmod 755 "$SERVICE_MENU" "$FOLDER_MENU"
refresh_caches

echo "Installed. Launch gmpublisher from your application menu, or run: $BIN"
echo ".gma files now open in gmpublisher. In Dolphin, right-click a .gma to extract it, or a folder to open it as an addon."
echo "Run 'gmpublisher --help' for the command-line tools."
