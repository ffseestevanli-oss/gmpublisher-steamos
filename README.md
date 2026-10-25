🛠️ gmpublisher Steventechno Edition v1.0.0

🚀 The first release of a Linux-first rewrite of gmpublisher, made for SteamOS, Bazzite, and other KDE Plasma desktops. Built with Rust and KDE's Kirigami.

💙 gmpublisher was created by William Venner, and all credit for it goes to him. This edition is based on the original gmpublisher and is released under the same GPL-3.0 license.

👤 Original author: https://github.com/WilliamVenner

🔗 Original project: https://github.com/WilliamVenner/gmpublisher

━━━━━━━━━━━━━━━━━━━━

✨ WHAT IT DOES

📂 Browse your installed Garry's Mod addons and Workshop downloads.

👀 Open, preview, and extract .gma files.

📦 Pack addon folders into .gma files.

🚀 Publish, update, and edit your Steam Workshop items, starting from My Workshop.

🌐 Download and extract any Workshop addon.

🎮 Steam must be running and signed in to use the Workshop features.

━━━━━━━━━━━━━━━━━━━━

📥 WHICH DOWNLOAD?

🟢 gmpublisher-qt-x86_64.AppImage

⭐ RECOMMENDED FOR STEAMOS AND BAZZITE

A single file that uses the Qt 6 and Kirigami libraries already available on your Plasma desktop. No traditional installation required.

🟣 gmpublisher-qt-x86_64.flatpak

📦 For users who prefer Flatpak. It brings its own KDE runtime from Flathub.

🔵 gmpublisher-qt_linux_x86_64.tar.gz

🖥️ For users who want a per-user installation with Dolphin right-click menus for .gma files and addon folders.

━━━━━━━━━━━━━━━━━━━━

🟢 APPIMAGE INSTALLATION

In Desktop Mode:

1️⃣ Download the AppImage.

2️⃣ Right-click the file in Dolphin and select Properties → Permissions.

3️⃣ Enable Is executable.

4️⃣ Double-click the file to launch it.

⌨️ Alternatively, open a terminal and run:

chmod +x gmpublisher-qt-x86_64.AppImage

./gmpublisher-qt-x86_64.AppImage

━━━━━━━━━━━━━━━━━━━━

🟣 FLATPAK INSTALLATION

📦 Install the downloaded Flatpak and launch the application with:

flatpak install --user gmpublisher-qt-x86_64.flatpak

flatpak run io.github.ffseestevanli_oss.gmpublisher

🔧 If Flatpak asks where to get the KDE runtime, add Flathub first:

flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

⚠️ The Flatpak reaches Steam through its s…

━━━━━━━━━━━━━━━━━━━━

ℹ️ Note: The original text ends mid-sentence at “The Flatpak reaches Steam through its s”, so the remaining instructions are incomplete.
