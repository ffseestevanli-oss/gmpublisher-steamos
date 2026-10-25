import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import io.github.gmpublisher

Kirigami.ScrollablePage {
	id: page
	title: "My Workshop"

	readonly property var backend: applicationWindow().backend
	readonly property var items: JSON.parse(backend.myItemsJson)
	readonly property var visibilityNames: ({ "public": "Public", "friends": "Friends only", "unlisted": "Unlisted", "private": "Private" })

	Component.onCompleted: if (backend.connected) backend.refreshMyItems()
	Connections {
		target: page.backend
		function onConnectedChanged() {
			if (page.backend.connected)
				page.backend.refreshMyItems()
		}
	}

	WorkshopTask {
		id: task
		// Show the new title, preview and so on once Steam has them
		onBusyChanged: if (!busy && publishedId !== "") page.backend.refreshMyItems()
	}
	ItemEditorDialog {
		id: editor
		task: task
	}
	DownloadDialog {
		id: downloader
		task: task
	}

	actions: [
		Kirigami.Action {
			text: "New item…"
			icon.name: "list-add"
			enabled: page.backend.connected && !task.busy
			onTriggered: editor.openFor(null)
		},
		Kirigami.Action {
			text: "Download…"
			icon.name: "download"
			tooltip: "Download any Workshop addon by its ID or link"
			enabled: page.backend.connected && !task.busy
			onTriggered: downloader.openFor("", "")
		},
		Kirigami.Action {
			text: "Refresh"
			icon.name: "view-refresh"
			enabled: page.backend.connected && !page.backend.loadingMyItems
			onTriggered: page.backend.refreshMyItems()
		}
	]

	header: ColumnLayout {
		spacing: 0
		TaskStatus {
			Layout.fillWidth: true
			task: task
		}
		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Error
			visible: page.backend.myItemsError !== ""
			text: page.backend.myItemsError
		}
		Controls.ProgressBar {
			Layout.fillWidth: true
			visible: page.backend.loadingMyItems
			indeterminate: true
		}
	}

	ListView {
		id: list
		model: page.items

		delegate: Controls.ItemDelegate {
			id: item
			required property var modelData
			width: ListView.view.width

			contentItem: RowLayout {
				spacing: Kirigami.Units.largeSpacing

				Image {
					Layout.preferredWidth: Kirigami.Units.gridUnit * 3
					Layout.preferredHeight: Kirigami.Units.gridUnit * 3
					source: item.modelData.previewUrl
					sourceSize.width: Kirigami.Units.gridUnit * 3
					sourceSize.height: Kirigami.Units.gridUnit * 3
					fillMode: Image.PreserveAspectCrop
					asynchronous: true

					Kirigami.Icon {
						anchors.fill: parent
						visible: parent.status !== Image.Ready
						source: "package-x-generic"
					}
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
							page.visibilityNames[item.modelData.visibility],
							applicationWindow().formatSize(item.modelData.size),
							"\u{1F44D} " + item.modelData.upvotes + "  \u{1F44E} " + item.modelData.downvotes,
							"updated " + new Date(item.modelData.updated * 1000).toLocaleDateString(Qt.locale(), Locale.ShortFormat)
						].join(" · ")
						elide: Text.ElideRight
						opacity: 0.7
						font: Kirigami.Theme.smallFont
					}
				}
				Controls.ToolButton {
					icon.name: "document-edit"
					text: "Edit…"
					display: Controls.AbstractButton.IconOnly
					enabled: !task.busy
					Controls.ToolTip.text: "Edit or update"
					Controls.ToolTip.visible: hovered
					onClicked: editor.openFor(item.modelData)
				}
				Controls.ToolButton {
					icon.name: "download"
					text: "Download…"
					display: Controls.AbstractButton.IconOnly
					enabled: !task.busy
					Controls.ToolTip.text: "Download or extract the GMA"
					Controls.ToolTip.visible: hovered
					onClicked: downloader.openFor(item.modelData.id, item.modelData.title)
				}
				Controls.ToolButton {
					icon.name: "internet-web-browser"
					text: "Open in Steam"
					display: Controls.AbstractButton.IconOnly
					Controls.ToolTip.text: text
					Controls.ToolTip.visible: hovered
					// Opens the item's page inside the Steam client
					onClicked: Qt.openUrlExternally("steam://url/CommunityFilePage/" + item.modelData.id)
				}
			}

			onClicked: if (!task.busy) editor.openFor(modelData)
		}

		Kirigami.PlaceholderMessage {
			anchors.centerIn: parent
			width: parent.width - Kirigami.Units.gridUnit * 4
			visible: list.count === 0 && !page.backend.loadingMyItems
			icon.name: "cloud-upload"
			text: !page.backend.connected ? "Waiting for Steam" : "You haven't published any addons yet"
			explanation: !page.backend.connected
				? "Start Steam and sign in to see the addons you've published."
				: "Choose New item to publish an addon folder or a GMA."
		}
	}
}
