import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
	id: page
	title: "My addons"

	readonly property var backend: applicationWindow().backend

	function openProject(folder) {
		applicationWindow().openProject(folder)
	}

	FolderDialog {
		id: addDialog
		title: "Choose an addon folder"
		onAccepted: {
			const folder = applicationWindow().urlToPath(selectedFolder)
			page.backend.addProject(folder)
			page.openProject(folder)
		}
	}

	actions: [
		Kirigami.Action {
			text: "Add addon folder…"
			icon.name: "list-add"
			onTriggered: addDialog.open()
		}
	]

	ListView {
		id: list
		model: JSON.parse(page.backend.projectsJson)

		delegate: Controls.ItemDelegate {
			id: item
			required property string modelData
			width: ListView.view.width

			contentItem: RowLayout {
				spacing: Kirigami.Units.largeSpacing

				Kirigami.Icon {
					source: "folder-development"
					implicitWidth: Kirigami.Units.iconSizes.medium
					implicitHeight: Kirigami.Units.iconSizes.medium
				}
				ColumnLayout {
					Layout.fillWidth: true
					spacing: 0

					Controls.Label {
						Layout.fillWidth: true
						text: item.modelData.substring(item.modelData.lastIndexOf("/") + 1)
						elide: Text.ElideRight
					}
					Controls.Label {
						Layout.fillWidth: true
						text: item.modelData
						elide: Text.ElideMiddle
						opacity: 0.7
						font: Kirigami.Theme.smallFont
					}
				}
				Controls.ToolButton {
					icon.name: "list-remove"
					text: "Forget"
					display: Controls.AbstractButton.IconOnly
					Controls.ToolTip.text: "Remove from this list (the folder isn't touched)"
					Controls.ToolTip.visible: hovered
					onClicked: page.backend.removeProject(item.modelData)
				}
			}

			onClicked: page.openProject(modelData)
		}

		Kirigami.PlaceholderMessage {
			anchors.centerIn: parent
			width: parent.width - Kirigami.Units.gridUnit * 4
			visible: list.count === 0
			icon.name: "folder-development"
			text: "No addon folders yet"
			explanation: "Add the folder you develop an addon in. gmpublisher edits its addon.json, checks its files against Garry's Mod's rules and packs it into a GMA."
			helpfulAction: Kirigami.Action {
				text: "Add addon folder…"
				icon.name: "list-add"
				onTriggered: addDialog.open()
			}
		}
	}
}
