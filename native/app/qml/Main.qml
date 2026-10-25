import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import io.github.gmpublisher

Kirigami.ApplicationWindow {
	id: root
	title: "gmpublisher Steventechno Edition"
	width: Kirigami.Units.gridUnit * 56
	height: Kirigami.Units.gridUnit * 38
	minimumWidth: Kirigami.Units.gridUnit * 22

	property alias backend: backend
	property alias rainbow: rainbow
	// Which drawer entry the page stack is showing
	property string section: "workshop"

	// The Rainbow Road colours, for the parts the colour scheme doesn't cover
	QtObject {
		id: rainbow
		readonly property color ground: "#0f0e17"
		readonly property color sidebar: "#17151f"
		readonly property color card: "#1a1826"
		readonly property color raised: "#221f30"
		readonly property color selected: "#2a2540"
		readonly property color border: "#3a3550"
		readonly property color text: "#f4f2ff"
		readonly property color muted: "#a9a4c4"
		readonly property color onRainbow: "#0f0e17"
		// The rainbow, plus lighter shades of its blue end that stay readable as text on the dark ground
		readonly property var colors: ["#ff3b5c", "#ff8a00", "#ffd400", "#2ee86b", "#20c5ff", "#6a5cff", "#c34bff"]
		readonly property var textColors: ["#ff5c77", "#ff9a2e", "#ffd400", "#2ee86b", "#20c5ff", "#8f85ff", "#d07bff"]

		function color(index) {
			return colors[((index % colors.length) + colors.length) % colors.length]
		}
	}

	Backend {
		id: backend
		Component.onCompleted: {
			start()
			if (openAtStart !== "")
				root.openGma(openAtStart)
			if (openFolderAtStart !== "") {
				addProject(openFolderAtStart)
				root.showPage(projectsPage, "projects")
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
	// Rich text that colours each letter along the rainbow
	function rainbowText(text) {
		const letters = Array.from(text)
		const last = Math.max(1, letters.length - 1)
		return letters.map((letter, i) => letter === " " ? " "
			: "<font color=\"" + rainbow.textColors[Math.round(i / last * (rainbow.textColors.length - 1))] + "\">"
				+ letter.replace("&", "&amp;").replace("<", "&lt;") + "</font>").join("")
	}
	function openGma(path) {
		pageStack.push(gmaPage, { path: path })
	}
	function openProject(folder) {
		pageStack.push(projectPage, { folder: folder })
	}
	function showPage(component, name) {
		section = name
		pageStack.clear()
		pageStack.push(component)
	}

	// A scrolling rainbow border along the top of the window
	header: RainbowStripe {
		width: root.width
	}

	globalDrawer: Kirigami.GlobalDrawer {
		id: drawer
		isMenu: false
		modal: !root.wideScreen
		handleVisible: modal
		width: Kirigami.Units.gridUnit * 14

		background: Rectangle {
			color: rainbow.sidebar
		}

		header: ColumnLayout {
			spacing: Kirigami.Units.largeSpacing

			RowLayout {
				Layout.topMargin: Kirigami.Units.largeSpacing * 2
				Layout.leftMargin: Kirigami.Units.largeSpacing * 2
				Layout.rightMargin: Kirigami.Units.largeSpacing * 2
				spacing: Kirigami.Units.largeSpacing

				Image {
					source: "../assets/brand/gmpublisher-icon.png"
					sourceSize.width: Kirigami.Units.gridUnit * 2.5
					sourceSize.height: Kirigami.Units.gridUnit * 2.5
					smooth: true
				}
				Kirigami.Heading {
					Layout.fillWidth: true
					text: "gmpublisher"
					level: 2
					font.weight: Font.Black
					elide: Text.ElideRight
				}
			}

			// The Steventechno Edition badge
			Rectangle {
				Layout.fillWidth: true
				Layout.leftMargin: Kirigami.Units.largeSpacing * 2
				Layout.rightMargin: Kirigami.Units.largeSpacing * 2
				Layout.bottomMargin: Kirigami.Units.largeSpacing
				implicitHeight: badge.implicitHeight + Kirigami.Units.smallSpacing * 2
				radius: height / 2
				color: rainbow.raised

				RowLayout {
					id: badge
					anchors.fill: parent
					anchors.leftMargin: Kirigami.Units.smallSpacing
					anchors.rightMargin: Kirigami.Units.largeSpacing
					spacing: Kirigami.Units.smallSpacing

					Image {
						source: "../assets/brand/steventechno-logo.png"
						sourceSize.width: Kirigami.Units.gridUnit * 2
						sourceSize.height: Kirigami.Units.gridUnit * 2
						smooth: true
					}
					Controls.Label {
						Layout.fillWidth: true
						textFormat: Text.StyledText
						text: root.rainbowText("STEVENTECHNO EDITION")
						font.weight: Font.Bold
						font.pixelSize: Kirigami.Theme.smallFont.pixelSize
						font.letterSpacing: 1.5
						wrapMode: Text.Wrap
					}
				}
			}
		}

		actions: [
			Kirigami.Action {
				text: "My Workshop"
				icon.name: "cloud-upload"
				checked: root.section === "workshop"
				onTriggered: root.showPage(myWorkshopPage, "workshop")
			},
			Kirigami.Action {
				text: "Installed addons"
				icon.name: "package-x-generic"
				checked: root.section === "installed"
				onTriggered: root.showPage(installedPage, "installed")
			},
			Kirigami.Action {
				text: "Open GMA…"
				icon.name: "document-open"
				onTriggered: openDialog.open()
			},
			Kirigami.Action {
				text: "My addons"
				icon.name: "folder-development"
				checked: root.section === "projects"
				onTriggered: root.showPage(projectsPage, "projects")
			},
			Kirigami.Action {
				text: "About"
				icon.name: "help-about"
				checked: root.section === "about"
				onTriggered: root.showPage(aboutPage, "about")
			}
		]

		content: [
			Item {
				Layout.fillHeight: true
			},
			// Who's signed in to Steam
			Rectangle {
				Layout.fillWidth: true
				Layout.leftMargin: Kirigami.Units.largeSpacing
				Layout.rightMargin: Kirigami.Units.largeSpacing
				implicitHeight: account.implicitHeight + Kirigami.Units.largeSpacing * 2
				radius: Kirigami.Units.smallSpacing * 3
				color: rainbow.card

				RowLayout {
					id: account
					anchors.fill: parent
					anchors.margins: Kirigami.Units.largeSpacing
					spacing: Kirigami.Units.largeSpacing

					// The Steam avatar in a rainbow frame
					Rectangle {
						implicitWidth: Kirigami.Units.gridUnit * 2.25
						implicitHeight: implicitWidth
						radius: Kirigami.Units.smallSpacing
						gradient: Gradient {
							orientation: Gradient.Horizontal
							GradientStop { position: 0; color: backend.connected ? "#ff3b5c" : rainbow.border }
							GradientStop { position: 0.5; color: backend.connected ? "#ffd400" : rainbow.border }
							GradientStop { position: 1; color: backend.connected ? "#20c5ff" : rainbow.border }
						}

						Image {
							anchors.fill: parent
							anchors.margins: 2
							visible: status === Image.Ready
							source: backend.avatarPath !== "" ? root.pathToUrl(backend.avatarPath) : ""
							sourceSize.width: width
							sourceSize.height: height
							fillMode: Image.PreserveAspectCrop
							cache: false
							smooth: true
						}
						Kirigami.Icon {
							anchors.centerIn: parent
							width: Kirigami.Units.iconSizes.small
							height: width
							visible: !backend.connected
							source: "network-disconnect"
						}
					}
					ColumnLayout {
						Layout.fillWidth: true
						spacing: 0

						Controls.Label {
							Layout.fillWidth: true
							text: backend.connected ? backend.personaName : "Not signed in"
							font.weight: Font.DemiBold
							elide: Text.ElideRight
						}
						Controls.Label {
							Layout.fillWidth: true
							text: backend.connected ? "Steam connected" : backend.steamStatus
							color: backend.connected ? "#9fe8b8" : rainbow.muted
							wrapMode: Text.Wrap
							font: Kirigami.Theme.smallFont
						}
					}
				}
			},
			// Credit where it's due
			Controls.Label {
				Layout.fillWidth: true
				Layout.margins: Kirigami.Units.largeSpacing
				Layout.leftMargin: Kirigami.Units.largeSpacing * 1.5
				wrapMode: Text.Wrap
				color: rainbow.muted
				font: Kirigami.Theme.smallFont
				textFormat: Text.StyledText
				linkColor: "#20c5ff"
				text: "Based on <a href=\"https://github.com/WilliamVenner/gmpublisher\">gmpublisher</a> by William Venner"
				onLinkActivated: link => Qt.openUrlExternally(link)

				HoverHandler {
					cursorShape: parent.hoveredLink !== "" ? Qt.PointingHandCursor : Qt.ArrowCursor
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

	pageStack.initialPage: myWorkshopPage
	pageStack.globalToolBar.style: Kirigami.ApplicationHeaderStyle.ToolBar
}
