# ⚙️ gmpublisher — SteamOS/Linux Fork

This project is a fork of [gmpublisher](https://github.com/WilliamVenner/gmpublisher), adapted with a focus on compatibility with **SteamOS and Linux**.

**All original application development and credit belong to [WilliamVenner](https://github.com/WilliamVenner) and the original contributors.** This fork is intended to make gmpublisher more accessible to SteamOS/Linux users. I do not claim authorship of the original application.

## 📦 Downloads & Installation

Download the available builds from this fork's **[Releases](../../releases)** page.

For the original application, visit the [upstream gmpublisher repository](https://github.com/WilliamVenner/gmpublisher).

### SteamOS, Bazzite & other Linux (recommended: Distrobox)

Switch to **Desktop Mode** on the Steam Deck, open a terminal (Konsole) and run:

```sh
curl -fsSL https://raw.githubusercontent.com/ffseestevanli-oss/gmpublisher-steamos/master/scripts/install-distrobox.sh | bash
```

This creates a small Ubuntu 22.04 [Distrobox](https://distrobox.it) container (Distrobox comes preinstalled on Bazzite and SteamOS 3.5+), installs gmpublisher into your home folder, and adds it to your app menu. No `sudo`, no `steamos-readonly disable`. Run the same command to update, or add `-s -- --uninstall` after `bash` to remove it.

Why a container? gmpublisher is built on Tauri 1, which needs WebKitGTK 4.0. Modern systems no longer ship it, and bundling it (as the AppImage does) clashes with newer graphics drivers. That shows up as a blank window or missing text. Inside the container, every library matches.

**AppImage (experimental):** `gmpublisher_linux_x86_64.AppImage` is also on the [releases page](../../releases). It works on some systems, but on Bazzite it can show a blank window. If it does, use the Distrobox installer above.

#### Things to know

- **Steam must be running** and logged in. gmpublisher talks to Steam through the Steamworks API.
- Use it in **Desktop Mode**; it isn't designed for Game Mode.
- If Garry's Mod isn't found automatically, set its folder in gmpublisher's settings (the folder that contains `GarrysMod/addons`).
- **Flatpak Steam:** finding Garry's Mod works, but connecting to a sandboxed Flatpak Steam client may not. SteamOS and Bazzite use native Steam.
- If the AppImage won't start with a FUSE error, run it with `--appimage-extract-and-run`.
- If you see nothing at all after launching the Distrobox install, run `~/.local/bin/gmpublisher` in a terminal to see its output.

## 🐧 What this fork changes

On SteamOS and Bazzite the system is read-only, so the WebKitGTK libraries the upstream Linux ZIP expects can't be installed. This fork:

- **Ships a Distrobox installer** that runs gmpublisher in a matching Ubuntu 22.04 container, plus an experimental self-contained AppImage.
- **Finds Garry's Mod in more places:** native Steam (`~/.steam`, `~/.local/share/Steam`), Flatpak Steam, Snap Steam, and every library folder, including SD cards and external drives such as `/run/media/mmcblk0p1` on the Steam Deck.
- **Loads `libsteam_api.so` from next to the executable**, not the current working directory, so it runs from any folder or launcher.
- **Avoids the blank-window WebKit bug** on Wayland / NVIDIA by setting `WEBKIT_DISABLE_DMABUF_RENDERER=1` unless you've set it yourself.

### Building from source

The AppImage is built by [`.github/workflows/linux-appimage.yml`](.github/workflows/linux-appimage.yml) in an Ubuntu 22.04 container (Tauri 1 needs `webkit2gtk-4.0`). Every push builds one (download it from the workflow run's artifacts), and pushing a version tag (e.g. `2.12.2-steamos.1`) attaches it to a GitHub release.

To build locally on an immutable distro, use a Distrobox/Toolbox Ubuntu 22.04 container:

```sh
sudo apt install build-essential curl wget file libwebkit2gtk-4.0-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libssl-dev patchelf
npm install
LD_LIBRARY_PATH="$PWD/src-tauri/target/release" npm run tauri build -- --bundles appimage
```

## ✨ Features

gmpublisher provides a comprehensive set of tools for managing Garry's Mod Workshop content:

- **Publish and update** Garry's Mod Workshop addons.
- **Browse, search, and extract** GMA files and installed addons.
- **Bulk download and extract** Workshop items and collections.
- **Animated GIF icons** for Workshop items.
- **Addon size analyzer** with a treemap visualization to identify disk space usage.
- **Legacy compatibility** for SteamPipe addons and older GMA formats.
- **Offline functionality** for supported operations.
- **Command-line interface** for terminal-based workflows.
- No dependency on `gmad.exe` or `gmpublish.exe`.

Feature availability may depend on the build and platform.

## 📚 Tutorials & Resources

- [Fastest Way to Upload a Garry's Mod Addon to Workshop — DanFMN](https://youtu.be/_syLXTFXmgM)
- [GMPublisher Guide in Russian — DarkFated](https://youtu.be/ldjHm85AnYk)
- [Original gmpublisher source code](https://github.com/WilliamVenner/gmpublisher)
- [Original gmpublisher releases](https://github.com/WilliamVenner/gmpublisher/releases)

## 🛠️ Technical Details

The original application is built using:

- **Rust** — backend
- **Svelte** — frontend
- **Tauri** — desktop application framework
- **steamworks-rs** — Steamworks integration

The original application is lightweight, with a reported size of approximately 10 MB.

## 🙏 Credits & Attribution

This project builds upon the work of the original gmpublisher developer and its contributors.

- **Original project:** [WilliamVenner/gmpublisher](https://github.com/WilliamVenner/gmpublisher)
- **Original author:** [WilliamVenner](https://github.com/WilliamVenner)
- **License:** See the original repository's license and any applicable notices.

This fork is an independent effort focused on SteamOS/Linux compatibility. Please refer to the original repository for upstream development, original documentation, and project history.
