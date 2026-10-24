import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import io.github.gmpublisher

Kirigami.ApplicationWindow {
	id: root
	title: "gmpublisher-qt"
	width: Kirigami.Units.gridUnit * 56
	height: Kirigami.Units.gridUnit * 38
	minimumWidth: Kirigami.Units.gridUnit * 22

	property alias backend: backend

	Backend {
		id: backend
		Component.onCompleted: {
			start()
			if (openAtStart !== "")
				root.openGma(openAtStart)
			if (openFolderAtStart !== "") {
				addProject(openFolderAtStart)
				root.showPage(projectsPage)
				root.openProject(openFolderAtStart)
			}
		}
	}

	function formatSize(bytes) {
		const units = ["B", "KiB", "MiB", "GiB"]
		let i = 0
		while (bytes >= 1024 && i < units.length - 1) {
			bytes /= 1024
			i++
		}
		return (i === 0 ? bytes : bytes.toFixed(1)) + " " + units[i]
	}
	function urlToPath(url) {
		return decodeURIComponent(url.toString().replace(/^file:\/\//, ""))
	}
	function pathToUrl(path) {
		return "file://" + path.split("/").map(encodeURIComponent).join("/")
	}
	function parentDir(path) {
		return path.substring(0, path.lastIndexOf("/")) || "/"
	}
	function openGma(path) {
		pageStack.push(gmaPage, { path: path })
	}
	function openProject(folder) {
		pageStack.push(projectPage, { folder: folder })
	}
	function showPage(component) {
		pageStack.clear()
		pageStack.push(component)
	}

	globalDrawer: Kirigami.GlobalDrawer {
		isMenu: false
		modal: !root.wideScreen
		handleVisible: modal
		width: Kirigami.Units.gridUnit * 14
		title: "gmpublisher-qt"
		titleIcon: "io.github.ffseestevanli_oss.gmpublisher"

		actions: [
			Kirigami.Action {
				text: "Installed addons"
				icon.name: "package-x-generic"
				onTriggered: root.showPage(installedPage)
			},
			Kirigami.Action {
				text: "My addons"
				icon.name: "folder-development"
				onTriggered: root.showPage(projectsPage)
			},
			Kirigami.Action {
				text: "My Workshop"
				icon.name: "cloud-upload"
				onTriggered: root.showPage(myWorkshopPage)
			},
			Kirigami.Action {
				text: "Open GMA…"
				icon.name: "document-open"
				onTriggered: openDialog.open()
			},
			Kirigami.Action {
				text: "About"
				icon.name: "help-about"
				onTriggered: root.showPage(aboutPage)
			}
		]

		content: [
			Item {
				Layout.fillHeight: true
			},
			Kirigami.Separator {
				Layout.fillWidth: true
			},
			RowLayout {
				Layout.fillWidth: true
				Layout.margins: Kirigami.Units.largeSpacing
				spacing: Kirigami.Units.smallSpacing

				Kirigami.Icon {
					source: backend.connected ? "network-connect" : "network-disconnect"
					implicitWidth: Kirigami.Units.iconSizes.small
					implicitHeight: Kirigami.Units.iconSizes.small
				}
				Controls.Label {
					Layout.fillWidth: true
					text: backend.connected ? backend.persona : backend.steamStatus
					wrapMode: Text.Wrap
					font: Kirigami.Theme.smallFont
				}
			}
		]
	}

	FileDialog {
		id: openDialog
		title: "Open a Garry's Mod addon"
		nameFilters: ["Garry's Mod addons (*.gma *.bin)", "All files (*)"]
		onAccepted: root.openGma(root.urlToPath(selectedFile))
	}

	Component {
		id: installedPage
		InstalledPage {}
	}
	Component {
		id: gmaPage
		GmaPage {}
	}
	Component {
		id: projectsPage
		ProjectsPage {}
	}
	Component {
		id: projectPage
		ProjectPage {}
	}
	Component {
		id: myWorkshopPage
		MyWorkshopPage {}
	}
	Component {
		id: aboutPage
		AboutPage {}
	}

	pageStack.initialPage: installedPage
	pageStack.globalToolBar.style: Kirigami.ApplicationHeaderStyle.ToolBar
}
