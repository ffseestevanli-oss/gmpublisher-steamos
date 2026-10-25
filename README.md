Here's the rewrite — repositioned around the two big claims up front (completely new UI, Linux-first), with the original-author credit kept prominent:

```markdown
# ⚙️ gmpublisher — Redesigned, Linux‑First

A from‑scratch redesign of [gmpublisher](https://github.com/WilliamVenner/gmpublisher), the Garry's Mod
Workshop tool. This project keeps the mission of the original — publishing and managing Workshop
content without `gmad.exe` or `gmpublish.exe` — but with a **completely new user interface** and a
**Linux‑first** focus: built to run great on SteamOS, the Steam Deck, Bazzite, and desktop Linux.

**All credit for the original gmpublisher goes to
[WilliamVenner](https://github.com/WilliamVenner) and its contributors.** This is an independent fork
with a rebuilt UI and a Linux‑first build pipeline; it does not claim authorship of the original
application, and upstream development and history live in the original repository.

## 📦 Downloads & Installation

Grab a build from this repository's **[Releases](../../releases)** page.
For the original application, see the [upstream repository](https://github.com/WilliamVenner/gmpublisher).

### SteamOS, Bazzite & other Linux — one command

Switch to **Desktop Mode** on the Steam Deck, open a terminal (Konsole), and run:

```sh
curl -fsSL https://raw.githubusercontent.com/ffseestevanli-oss/gmpublisher-steamos/master/scripts/install-steamos.sh | bash
```

That installs `gmpublisher.AppImage` to `~/Applications` and adds it to your app menu. No `sudo`,
no `steamos-readonly disable`. Run the same command again to update, or add `-- --uninstall` to remove it.

**Prefer to do it by hand?** Download `gmpublisher_linux_x86_64.AppImage` from
[Releases](../../releases) and run:

```sh
chmod +x gmpublisher_linux_x86_64.AppImage
./gmpublisher_linux_x86_64.AppImage
```

On Bazzite, right‑click the AppImage and open it with **Gear Lever** to add it to your menu.

### Good to know

- **Steam must be running and signed in** — gmpublisher talks to Steam through the Steamworks API.
- Use it in **Desktop Mode**; it isn't designed for Game Mode.
- If Garry's Mod isn't detected, point gmpublisher at its folder in settings (the one containing `GarrysMod/addons`).
- **Flatpak Steam:** game detection works, but connecting to a sandboxed Flatpak Steam client may not. SteamOS and Bazzite ship native Steam.
- FUSE error on launch? Run with `--appimage-extract-and-run`.

## 🎨 A brand-new interface

The UI has been rebuilt end to end for this fork — new layout, new flows, and a look that fits the
Steam Deck and Linux desktops. Everything gmpublisher does still lives here:

- **Publish and update** Workshop addons.
- **Browse, search, and extract** GMA files and installed addons.
- **Bulk download and extract** Workshop items and collections.
- **Animated GIF icons** for Workshop items.
- **Addon size analyzer** —
