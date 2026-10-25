<p align="center"><img src="https://raw.githubusercontent.com/ffseestevanli-oss/gmpublisher-steamos/master/native/app/assets/brand/gmpublisher-rainbow.svg" width="128" alt="gmpublisher Steventechno Edition icon"></p>

# gmpublisher Steventechno Edition

A Linux-first rewrite of gmpublisher for SteamOS, Bazzite and other KDE Plasma desktops, built with Rust and KDE's Kirigami.

**gmpublisher was created by [William Venner](https://github.com/WilliamVenner), and all credit for it goes to him.** This edition is based on [the original gmpublisher](https://github.com/WilliamVenner/gmpublisher) and is released under the same GPL-3.0 license.

## What it does

- Browse your installed Garry's Mod addons and Workshop downloads
- Open, preview and extract `.gma` files
- Pack addon folders into `.gma` files
- Publish, update and edit your Steam Workshop items, starting from **My Workshop**
- Download and extract any Workshop addon

Steam has to be running and signed in for the Workshop features.

## Which download?

| File | Use it when |
| --- | --- |
| `gmpublisher-qt-x86_64.AppImage` | **Recommended for SteamOS and Bazzite.** One file that uses the Qt 6 and Kirigami your Plasma desktop already has. |
| `gmpublisher-qt-x86_64.flatpak` | You prefer Flatpak. It brings its own KDE runtime from Flathub. |
| `gmpublisher-qt_linux_x86_64.tar.gz` | You want a per-user install with Dolphin right-click menus for `.gma` files and addon folders. |

### AppImage

In Desktop Mode, download it, then right-click it in Dolphin, choose **Properties → Permissions** and tick **Is executable**, and double-click it. Or from a terminal:

```sh
chmod +x gmpublisher-qt-x86_64.AppImage
./gmpublisher-qt-x86_64.AppImage
```

### Flatpak

```sh
flatpak install --user gmpublisher-qt-x86_64.flatpak
flatpak run io.github.ffseestevanli_oss.gmpublisher
```

If Flatpak asks where to get the KDE runtime, add Flathub first: `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.

The Flatpak's connection to Steam runs through its sandbox and is the least tested part of this release. If it keeps saying it's waiting for Steam, use the AppImage for Workshop work and please open an issue.

### Tarball

```sh
tar -xzf gmpublisher-qt_linux_x86_64.tar.gz
./gmpublisher-qt/install.sh
```

Run `./gmpublisher-qt/install.sh --uninstall` to remove it again.
