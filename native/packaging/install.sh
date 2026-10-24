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

if [ "${1:-}" = "--uninstall" ]; then
	rm -rf "$INSTALL_DIR" "$BIN" "$DESKTOP_FILE" "$ICON_FILE"
	echo "gmpublisher removed."
	exit 0
fi

if ! ls /usr/lib*/qt6/qml/org/kde/kirigami /usr/lib/*/qt6/qml/org/kde/kirigami >/dev/null 2>&1; then
	echo "Warning: Kirigami (Qt 6) wasn't found. It ships with KDE Plasma 6; on other desktops install your distro's kirigami / kf6-kirigami package." >&2
fi

mkdir -p "$INSTALL_DIR" "$(dirname "$BIN")" "$(dirname "$DESKTOP_FILE")" "$(dirname "$ICON_FILE")"
install -m755 "$HERE/gmpublisher" "$INSTALL_DIR/gmpublisher"
install -m644 "$HERE/libsteam_api.so" "$INSTALL_DIR/libsteam_api.so"
ln -sf "$INSTALL_DIR/gmpublisher" "$BIN"
install -m644 "$HERE/$ID.png" "$ICON_FILE"
sed "s|^Exec=.*|Exec=$INSTALL_DIR/gmpublisher|" "$HERE/$ID.desktop" > "$DESKTOP_FILE"
update-desktop-database "$(dirname "$DESKTOP_FILE")" >/dev/null 2>&1 || true

echo "Installed. Launch gmpublisher from your application menu, or run: $BIN"
