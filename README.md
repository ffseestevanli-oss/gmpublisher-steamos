# ⚙️ gmpublisher for Linux, SteamOS & Bazzite

> [!IMPORTANT]
> **All credit for gmpublisher goes to [William Venner](https://github.com/WilliamVenner) and the [gmpublisher contributors](https://github.com/WilliamVenner/gmpublisher/graphs/contributors).**
> This is an unofficial fork, and its maintainer takes **zero credit** for the app itself. It only adds packaging and small fixes so gmpublisher runs on Linux, especially immutable distros like SteamOS and Bazzite.
> The original project lives at **[WilliamVenner/gmpublisher](https://github.com/WilliamVenner/gmpublisher)**. Please support and star it there.
> Bugs that also happen on Windows/macOS or in the upstream build belong in the [upstream issue tracker](https://github.com/WilliamVenner/gmpublisher/issues).

<p align="center"><img src="https://github.com/WilliamVenner/gmpublisher/assets/14863743/8c514124-7f63-4d20-b6c6-25989c9bf528"/></p>

gmpublisher is a powerful Workshop publisher for Garry's Mod: publish and update Workshop items, extract and browse GMA files, bulk download addons, and more, without gmad or gmpublish.

## Why this fork?

On SteamOS (Steam Deck) and Bazzite the system is read-only, so you can't `pacman`/`dnf install` the WebKitGTK libraries the upstream Linux ZIP expects. This fork:

* **Ships a self-contained AppImage.** WebKitGTK and `libsteam_api.so` are bundled; nothing needs installing system-wide.
* **Finds Garry's Mod in more places.** Native Steam (`~/.steam`, `~/.local/share/Steam`), Flatpak Steam, Snap Steam, and every library folder, including SD cards and external drives like `/run/media/mmcblk0p1` on the Steam Deck.
* **Loads `libsteam_api.so` from next to the executable**, not the current working directory, so it runs from any folder or app launcher.
* **Avoids the blank-window WebKit bug** on Wayland / NVIDIA (common on Bazzite) by setting `WEBKIT_DISABLE_DMABUF_RENDERER=1` unless you've set it yourself.

## 📦 Installation (SteamOS, Bazzite, any Linux)

Switch to **Desktop Mode** on the Steam Deck, open a terminal (Konsole) and run:

```sh
curl -fsSL https://raw.githubusercontent.com/ffseestevanli-oss/gmpublisher-steamos/master/scripts/install-steamos.sh | bash
```

This puts `gmpublisher.AppImage` in `~/Applications` and adds it to your app menu. No `sudo`, no `steamos-readonly disable`. Run the same command to update, or add `-s -- --uninstall` after `bash` to remove it.

**Manual install:** download `gmpublisher_linux_x86_64.AppImage` from the [releases page](https://github.com/ffseestevanli-oss/gmpublisher-steamos/releases), then:

```sh
chmod +x gmpublisher_linux_x86_64.AppImage
./gmpublisher_linux_x86_64.AppImage
```

On Bazzite you can also open the AppImage with **Gear Lever** to integrate it into your menu.

### Things to know

* **Steam must be running** and logged in. gmpublisher talks to Steam through the Steamworks API to publish and download.
* Use it in **Desktop Mode**. It's a desktop app and is not designed for Game Mode.
* If Garry's Mod isn't found automatically, set its folder in gmpublisher's settings (the folder that contains `GarrysMod/addons`).
* **Flatpak Steam:** finding Garry's Mod works, but connecting to a sandboxed Flatpak Steam client from outside the sandbox may not. SteamOS and Bazzite both use native Steam, so this only matters on other distros.
* If the AppImage won't start with a FUSE error, run it with `--appimage-extract-and-run`.

## 🛠️ Building from source

The AppImage is built by [`.github/workflows/linux-appimage.yml`](.github/workflows/linux-appimage.yml) in an Ubuntu 22.04 container (Tauri 1 needs `webkit2gtk-4.0`). Every push builds one, and you can download it from the workflow run's artifacts. Pushing a version tag (e.g. `2.12.2`) attaches it to a GitHub release.

To build locally on an immutable distro, do it inside a Distrobox/Toolbox Ubuntu 22.04 container:

```sh
sudo apt install build-essential curl wget file libwebkit2gtk-4.0-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libssl-dev patchelf
npm install
LD_LIBRARY_PATH="$PWD/src-tauri/target/release" npm run tauri build -- --bundles appimage
```

## Features

* Doesn't depend on gmad.exe or gmpublish.exe
* Publish & update your Workshop items
* Extract, search and browse GMA files and installed addons
* Bulk download & extract Workshop items and collections
* Upload animated GIFs as your Workshop item's icon
* Analyze which addons are taking up the most disk space using the addon size analyzer treemap
* Supports legacy SteamPipe addons and old GMA versions
* Works without an Internet connection
* CLI interface

Translations, tutorials and Windows/macOS downloads are on the [original project page](https://github.com/WilliamVenner/gmpublisher).

## License & credits

gmpublisher is created by [William Venner](https://github.com/WilliamVenner) and contributors, and is licensed under the [GPL-3.0](LICENSE). This fork is distributed under the same license. All the app's design, code and features are their work.

## Technical Stuff

* The program makes heavy use of multithreading, and will work best on processors with a decent amount of cores.
* Made using [Rust](https://www.rust-lang.org/) (backend) and [Svelte](https://svelte.dev/) (frontend)
* This is not an Electron app; this is a [Tauri](https://github.com/tauri-apps/tauri) app. Big thanks to all the contributors to Tauri for their amazing work on finally killing Electron for good.
* gmpublisher uses the fantastic [steamworks-rs](https://crates.io/crates/steamworks) library for interfacing with the [Steamworks SDK](https://partner.steamgames.com/doc/api)
* The program is only about ~10 MB

## Media

![Screenshot](https://user-images.githubusercontent.com/14863743/115953601-5f1a7400-a4e4-11eb-831c-d6a924afbf33.png)

![Screenshot](https://user-images.githubusercontent.com/14863743/115953605-63469180-a4e4-11eb-9f96-90b992cbffc4.png)

![Screenshot](https://user-images.githubusercontent.com/14863743/115954341-5b88ec00-a4e8-11eb-8f27-c03d43df165a.png)

![Screenshot](https://user-images.githubusercontent.com/14863743/115953616-7c4f4280-a4e4-11eb-95c0-add80b1d41bd.png)

![Screenshot](https://user-images.githubusercontent.com/14863743/115953639-9db02e80-a4e4-11eb-935d-bad41cd30bde.png)

![Screenshot](https://user-images.githubusercontent.com/14863743/115958825-00afbe80-a501-11eb-81da-6d53a94eddbf.png)

![Screenshot](https://user-images.githubusercontent.com/14863743/115953801-845bb200-a4e5-11eb-8fc2-8b142f2be237.png)

![Screenshot](https://user-images.githubusercontent.com/14863743/115953820-99d0dc00-a4e5-11eb-93a4-36e8b2248e87.png)

![Screenshot](https://user-images.githubusercontent.com/14863743/115953827-a35a4400-a4e5-11eb-9691-48e520eb9bb1.png)

![Screenshot](https://user-images.githubusercontent.com/14863743/115953670-bb7d9380-a4e4-11eb-8f54-f43fcd153d90.png)

<p align="center"><img src="https://i.imgur.com/Un4akZe.gif"/></p>
