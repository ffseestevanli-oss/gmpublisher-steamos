#!/usr/bin/env bash
# Launch the AppImage on Fedora with different bundled libraries removed, and record how it renders.
set -u
APPIMAGE="$1"
chmod +x "$APPIMAGE"
"$APPIMAGE" --appimage-extract >/dev/null
mkdir -p off results
LIB=squashfs-root/usr/lib

run() {
	local name="$1" backend="$2"
	echo "::group::$name ($backend)"
	rm -f app.log shot.png shot2.png
	if [ "$backend" = x11 ]; then
		xvfb-run -a -s "-screen 0 1600x1000x24" sh -c '
			dbus-launch --exit-with-session ./squashfs-root/AppRun > app.log 2>&1 &
			sleep 20; import -window root shot.png
			W=$(xdotool search --name gmpublisher | head -1); xdotool windowsize "$W" 1500 950; sleep 4
			import -window root shot2.png
			pkill -f squashfs-root/usr/bin/gmpublisher; sleep 1'
	else
		export XDG_RUNTIME_DIR=$(mktemp -d); chmod 700 "$XDG_RUNTIME_DIR"
		weston --backend=headless --renderer=pixman --socket=wl0 --width=1280 --height=800 --debug > weston.log 2>&1 &
		local wpid=$!
		sleep 3
		WAYLAND_DISPLAY=wl0 GDK_BACKEND=wayland ./squashfs-root/AppRun > app.log 2>&1 &
		sleep 20
		(cd "$XDG_RUNTIME_DIR" && WAYLAND_DISPLAY=wl0 timeout 10 weston-screenshooter >/dev/null 2>&1); cp "$XDG_RUNTIME_DIR"/wayland-screenshot*.png shot.png 2>/dev/null
		pkill -f squashfs-root/usr/bin/gmpublisher; kill $wpid; sleep 1
		tail -5 weston.log >> app.log
	fi
	grep -v -E '^\s+"|Steam|S_API|dlopen|steamclient|with error|^\]|whitelist|D-Bus' app.log | head -30 | tee "results/$backend-$name.log"
	echo "::endgroup::"
	[ -f shot.png ] && convert shot.png -resize 800x500 "results/$backend-$name.png"
	[ -f shot2.png ] && convert shot2.png -resize 800x500 "results/$backend-$name-resized.png"
	echo "$backend $name: $([ -f shot.png ] && convert shot.png -format '%k' info: || echo none)" | tee -a results/summary.txt
}

# Remove bundled libraries matching the given prefixes, run, then restore them.
without() {
	local name="$1" backend="$2"; shift 2
	for p in "$@"; do mv $LIB/$p* off/ 2>/dev/null; done
	run "$name" "$backend"
	mv off/* $LIB/ 2>/dev/null
}

run original x11
without test3-set x11 libelf libdw libzstd libxcb- libepoxy libffi libxkbcommon libX
without graphite x11 libgraphite2
without pango-cairo x11 libpango libcairo libpixman
without png-brotli-bz2 x11 libpng16 libbrotli libbz2
without woff2 x11 libwoff2
without thai x11 libthai libdatrie
without xml x11 libxml2 libxslt
without glib x11 libglib libgobject libgio libgmodule libffi libpcre libmount libblkid libselinux
without gtk-gdk x11 libgtk libgdk libatk libatspi libepoxy
without fontstack x11 libgraphite2 libpango libcairo libpixman libpng16 libbrotli libbz2 libwoff2 libthai libdatrie
run original wayland
without test3-set wayland libelf libdw libzstd libxcb- libepoxy libffi libxkbcommon libX
for g in libelf libdw libzstd libxcb- libepoxy libffi libxkbcommon libX; do without "only-$g" wayland $g; done
