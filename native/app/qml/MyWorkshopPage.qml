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
	readonly property var rainbow: applicationWindow().rainbow
	property string filter: ""

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
		RainbowProgressBar {
			Layout.fillWidth: true
			visible: page.backend.loadingMyItems
			indeterminate: true
		}
		RowLayout {
			Layout.fillWidth: true
			Layout.margins: Kirigami.Units.largeSpacing * 2
			Layout.bottomMargin: Kirigami.Units.largeSpacing
			spacing: Kirigami.Units.largeSpacing

			Kirigami.SearchField {
				Layout.fillWidth: true
				Layout.maximumWidth: Kirigami.Units.gridUnit * 18
				placeholderText: "Search your items…"
				onTextChanged: page.filter = text.toLowerCase()
			}
			Item {
				Layout.fillWidth: true
			}
			RainbowButton {
				text: "+ New item"
				enabled: page.backend.connected && !task.busy
				onClicked: editor.openFor(null)
			}
		}
	}

	GridView {
		id: grid
		model: page.items.filter(item => item.title.toLowerCase().includes(page.filter) || item.id.includes(page.filter))
		leftMargin: Kirigami.Units.largeSpacing * 2
		rightMargin: Kirigami.Units.largeSpacing * 2
		bottomMargin: Kirigami.Units.largeSpacing * 2
		keyNavigationEnabled: true
		currentIndex: -1

		// As many columns of cards as fit, at least a dozen grid units wide each
		readonly property real available: width - leftMargin - rightMargin
		readonly property int columns: Math.max(1, Math.floor(available / (Kirigami.Units.gridUnit * 12)))
		cellWidth: Math.floor(available / columns)
		cellHeight: Math.round((cellWidth - Kirigami.Units.largeSpacing * 3) * 9 / 16) + Kirigami.Units.gridUnit * 7

		delegate: Controls.ItemDelegate {
			id: card
			required property var modelData
			required property int index
			readonly property color accent: page.rainbow.color(index)
			width: GridView.view.cellWidth
			height: GridView.view.cellHeight
			padding: Kirigami.Units.largeSpacing
			// Each card is the whole cell minus a gap
			topInset: Kirigami.Units.smallSpacing
			bottomInset: Kirigami.Units.smallSpacing
			leftInset: Kirigami.Units.smallSpacing
			rightInset: Kirigami.Units.smallSpacing
			hoverEnabled: true

			Accessible.name: modelData.title

			background: Rectangle {
				radius: Kirigami.Units.largeSpacing * 1.5
				color: card.down ? page.rainbow.selected : page.rainbow.card
				border.width: card.hovered || card.visualFocus || card.GridView.isCurrentItem ? 2 : 0
				border.color: card.accent
			}

			contentItem: ColumnLayout {
				spacing: Kirigami.Units.smallSpacing

				// The preview image, or a splash of the card's colour until Steam sends it
				Rectangle {
					Layout.fillWidth: true
					Layout.preferredHeight: Math.round(width * 9 / 16)
					radius: Kirigami.Units.smallSpacing * 2
					gradient: Gradient {
						orientation: Gradient.Vertical
						GradientStop { position: 0; color: card.accent }
						GradientStop { position: 1; color: page.rainbow.card }
					}
					clip: true

					Image {
						id: thumb
						anchors.fill: parent
						source: card.modelData.previewUrl
						sourceSize.width: width
						fillMode: Image.PreserveAspectCrop
						asynchronous: true
					}
					Kirigami.Icon {
						anchors.centerIn: parent
						width: Kirigami.Units.iconSizes.large
						height: width
						visible: thumb.status !== Image.Ready
						source: "package-x-generic"
					}
					Controls.Label {
						anchors.top: parent.top
						anchors.right: parent.right
						anchors.margins: Kirigami.Units.smallSpacing * 2
						leftPadding: Kirigami.Units.smallSpacing * 2
						rightPadding: Kirigami.Units.smallSpacing * 2
						topPadding: Kirigami.Units.smallSpacing / 2
						bottomPadding: Kirigami.Units.smallSpacing / 2
						text: page.visibilityNames[card.modelData.visibility]
						color: card.modelData.visibility === "public" ? "#9fe8b8" : "#ffd27a"
						font.pixelSize: Kirigami.Theme.smallFont.pixelSize
						font.weight: Font.Bold
						background: Rectangle {
							radius: height / 2
							color: Qt.rgba(15 / 255, 14 / 255, 23 / 255, 0.8)
						}
					}
				}
				Controls.Label {
					Layout.fillWidth: true
					Layout.topMargin: Kirigami.Units.smallSpacing
					text: card.modelData.title
					font.weight: Font.DemiBold
					wrapMode: Text.Wrap
					maximumLineCount: 2
					elide: Text.ElideRight
				}
				Controls.Label {
					Layout.fillWidth: true
					text: applicationWindow().formatSize(card.modelData.size)
						+ "  ·  ▲ " + card.modelData.upvotes + "  ▼ " + card.modelData.downvotes
					color: page.rainbow.muted
					font: Kirigami.Theme.smallFont
					elide: Text.ElideRight
				}
				Item {
					Layout.fillHeight: true
				}
				RowLayout {
					Layout.fillWidth: true
					spacing: 0

					Controls.Label {
						Layout.fillWidth: true
						text: "Updated " + new Date(card.modelData.updated * 1000).toLocaleDateString(Qt.locale(), Locale.ShortFormat)
						color: page.rainbow.muted
						font: Kirigami.Theme.smallFont
						elide: Text.ElideRight
					}
					Controls.ToolButton {
						icon.name: "document-edit"
						text: "Edit…"
						display: Controls.AbstractButton.IconOnly
						enabled: !task.busy
						Controls.ToolTip.text: "Edit or update"
						Controls.ToolTip.visible: hovered
						onClicked: editor.openFor(card.modelData)
					}
					Controls.ToolButton {
						icon.name: "download"
						text: "Download…"
						display: Controls.AbstractButton.IconOnly
						enabled: !task.busy
						Controls.ToolTip.text: "Download or extract the GMA"
						Controls.ToolTip.visible: hovered
						onClicked: downloader.openFor(card.modelData.id, card.modelData.title)
					}
					Controls.ToolButton {
						icon.name: "internet-web-browser"
						text: "Open in Steam"
						display: Controls.AbstractButton.IconOnly
						Controls.ToolTip.text: text
						Controls.ToolTip.visible: hovered
						// Opens the item's page inside the Steam client
						onClicked: Qt.openUrlExternally("steam://url/CommunityFilePage/" + card.modelData.id)
					}
				}
			}

			onClicked: if (!task.busy) editor.openFor(modelData)
		}

		Kirigami.PlaceholderMessage {
			anchors.centerIn: parent
			width: parent.width - Kirigami.Units.gridUnit * 4
			visible: grid.count === 0 && !page.backend.loadingMyItems
			icon.name: "cloud-upload"
			text: !page.backend.connected ? "Waiting for Steam"
				: page.filter !== "" ? "Nothing matches your search"
				: "You haven't published any addons yet"
			explanation: !page.backend.connected
				? "Start Steam and sign in to see the addons you've published."
				: page.filter !== "" ? "" : "Choose New item to publish an addon folder or a GMA."
		}
	}
}
