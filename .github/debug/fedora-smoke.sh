#!/usr/bin/env bash
# Launch the AppImage on Fedora with different bundled libraries removed, and report how it renders.
set -u
APPIMAGE="$1"
chmod +x "$APPIMAGE"
"$APPIMAGE" --appimage-extract >/dev/null
mkdir -p off

run() {
	local name="$1" backend="$2"
	echo "::group::$name ($backend)"
	rm -f app.log shot.png
	if [ "$backend" = x11 ]; then
		xvfb-run -a -s "-screen 0 1280x800x24" sh -c '
			dbus-launch --exit-with-session ./squashfs-root/AppRun > app.log 2>&1 &
			sleep 25; import -window root shot.png; pkill -f squashfs-root/usr/bin/gmpublisher; sleep 1' 
	else
		export XDG_RUNTIME_DIR=$(mktemp -d); chmod 700 "$XDG_RUNTIME_DIR"
		weston --backend=headless --renderer=gl --socket=wl0 --width=1280 --height=800 --debug > weston.log 2>&1 &
		local wpid=$!
		sleep 3
		WAYLAND_DISPLAY=wl0 GDK_BACKEND=wayland dbus-launch --exit-with-session ./squashfs-root/AppRun > app.log 2>&1 &
		sleep 25
		(cd "$XDG_RUNTIME_DIR" && WAYLAND_DISPLAY=wl0 timeout 10 weston-screenshooter >/dev/null 2>&1; ls) ; cp "$XDG_RUNTIME_DIR"/wayland-screenshot*.png shot.png 2>/dev/null
		pkill -f squashfs-root/usr/bin/gmpublisher; kill $wpid; sleep 1
		grep -i -E "error|egl|fail" weston.log | head -5
	fi
	grep -v -E '^\s+"|Steam|S_API|dlopen|steamclient|with error|^\]|whitelist' app.log | head -30
	echo "::endgroup::"
	if [ -f shot.png ]; then
		echo "RESULT $name $backend: $(convert shot.png -format '%k' info:) colours"
		echo "THUMB $name $backend $(convert shot.png -resize 400x250 -colors 64 png:- | base64 -w0)"
	else
		echo "RESULT $name $backend: no screenshot"
	fi
}

for backend in x11 wayland; do
	run original "$backend"
	for group in libelf libdw libzstd libxcb- libepoxy libffi libxkbcommon libX libharfbuzz-icu libgraphite2; do
		mv squashfs-root/usr/lib/$group* off/
		run "without-$group" "$backend"
		mv off/* squashfs-root/usr/lib/
	done
	mv squashfs-root/usr/lib/{libelf,libdw,libzstd,libxcb-,libepoxy,libffi,libxkbcommon,libX}* off/
	run without-test3-set "$backend"
	mv off/* squashfs-root/usr/lib/
done
