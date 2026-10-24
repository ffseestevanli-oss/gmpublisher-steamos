import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
	id: page
	title: "Installed addons"

	readonly property var backend: applicationWindow().backend
	readonly property var addons: JSON.parse(backend.installedJson)
	property string filter: ""

	actions: [
		Kirigami.Action {
			text: "Refresh"
			icon.name: "view-refresh"
			enabled: !page.backend.scanning
			onTriggered: page.backend.rescan()
		}
	]

	header: Controls.Control {
		padding: Kirigami.Units.smallSpacing
		contentItem: Kirigami.SearchField {
			placeholderText: "Search installed addons…"
			onTextChanged: page.filter = text.toLowerCase()
		}
	}

	ListView {
		id: list
		model: page.addons.filter(addon => addon.title.toLowerCase().includes(page.filter) || String(addon.workshopId).includes(page.filter))

		delegate: Controls.ItemDelegate {
			id: item
			required property var modelData
			width: ListView.view.width

			contentItem: RowLayout {
				spacing: Kirigami.Units.largeSpacing

				Kirigami.Icon {
					source: "package-x-generic"
					implicitWidth: Kirigami.Units.iconSizes.medium
					implicitHeight: Kirigami.Units.iconSizes.medium
				}
				ColumnLayout {
					Layout.fillWidth: true
					spacing: 0

					Controls.Label {
						Layout.fillWidth: true
						text: item.modelData.title
						elide: Text.ElideRight
					}
					Controls.Label {
						Layout.fillWidth: true
						text: [
							item.modelData.workshopId ? "Workshop " + item.modelData.workshopId : item.modelData.source,
							applicationWindow().formatSize(item.modelData.size),
							item.modelData.files !== null ? item.modelData.files + " files" : "compressed"
						].join(" · ")
						elide: Text.ElideRight
						opacity: 0.7
						font: Kirigami.Theme.smallFont
					}
				}
			}

			onClicked: applicationWindow().openGma(modelData.path)
		}

		Kirigami.PlaceholderMessage {
			anchors.centerIn: parent
			width: parent.width - Kirigami.Units.gridUnit * 4
			visible: list.count === 0
			icon.name: page.backend.scanning ? "view-refresh" : "package-x-generic"
			text: page.backend.scanning ? "Looking for addons…"
				: page.filter !== "" ? "Nothing matches your search"
				: page.backend.gmodDir === "" ? "Garry's Mod wasn't found"
				: "No addons installed"
			explanation: !page.backend.scanning && page.filter === "" && page.backend.gmodDir === ""
				? "Install Garry's Mod through Steam, or start Steam so gmpublisher can ask it where the game is."
				: ""
		}
	}
}
