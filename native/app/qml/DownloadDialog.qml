import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Downloads any Workshop addon and saves its GMA, or extracts it, into a local folder
Kirigami.Dialog {
	id: dialog

	required property var task
	readonly property bool steamReady: applicationWindow().backend.connected
	property string destination: ""
	property bool badLink: false

	title: "Download from the Workshop"
	padding: Kirigami.Units.largeSpacing
	preferredWidth: Kirigami.Units.gridUnit * 30

	// Opens with an item filled in, or empty to paste one
	function openFor(itemId, itemTitle) {
		linkField.text = itemId || ""
		itemLabel.text = itemTitle || ""
		badLink = false
		if (destination === "")
			destination = task.defaultDownloadDir()
		open()
	}

	Controls.ButtonGroup {
		buttons: [saveGma, extractFiles]
	}

	FolderDialog {
		id: folderDialog
		title: "Save into"
		currentFolder: dialog.destination === "" ? "" : applicationWindow().pathToUrl(dialog.destination)
		onAccepted: dialog.destination = applicationWindow().urlToPath(selectedFolder)
	}

	ColumnLayout {
		spacing: Kirigami.Units.largeSpacing

		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Warning
			visible: !dialog.steamReady
			text: "Steam isn't connected. Start Steam and sign in to download."
		}
		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Error
			visible: dialog.badLink
			text: "That isn't a Workshop item ID or link."
		}

		Kirigami.FormLayout {
			Layout.fillWidth: true

			Controls.Label {
				id: itemLabel
				Kirigami.FormData.label: "Addon:"
				Layout.fillWidth: true
				visible: text !== ""
				elide: Text.ElideRight
			}
			Controls.TextField {
				id: linkField
				Kirigami.FormData.label: "Workshop item:"
				Layout.fillWidth: true
				visible: itemLabel.text === ""
				placeholderText: "Paste an ID or a Workshop link"
				onTextEdited: dialog.badLink = false
			}
			Controls.RadioButton {
				id: saveGma
				Kirigami.FormData.label: "Get:"
				text: "The .gma file"
				checked: true
			}
			Controls.RadioButton {
				id: extractFiles
				text: "The addon's files, extracted into a new folder"
			}
			RowLayout {
				Kirigami.FormData.label: "Save into:"
				Layout.fillWidth: true

				Controls.Label {
					Layout.fillWidth: true
					text: dialog.destination
					elide: Text.ElideMiddle
				}
				Controls.Button {
					text: "Choose…"
					icon.name: "folder-open"
					onClicked: folderDialog.open()
				}
			}
		}

		Controls.Label {
			Layout.fillWidth: true
			wrapMode: Text.Wrap
			opacity: 0.7
			text: "Steam downloads the addon first, if it isn't installed and up to date already. Nothing in the folder is overwritten."
		}
	}

	customFooterActions: [
		Kirigami.Action {
			text: "Download"
			icon.name: "download"
			enabled: dialog.steamReady && !dialog.task.busy
			onTriggered: {
				if (!dialog.task.isWorkshopId(linkField.text)) {
					dialog.badLink = true
					linkField.forceActiveFocus()
					return
				}
				dialog.task.download(linkField.text, dialog.destination, extractFiles.checked)
				dialog.close()
			}
		},
		Kirigami.Action {
			text: "Cancel"
			icon.name: "dialog-cancel"
			onTriggered: dialog.close()
		}
	]
}
