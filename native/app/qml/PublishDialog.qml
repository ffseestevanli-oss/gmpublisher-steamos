import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.Dialog {
	id: dialog

	required property var project
	readonly property bool updating: linkField.text.trim() !== ""
	readonly property bool steamReady: applicationWindow().backend.connected
	property string previewPath: ""
	property bool badLink: false

	title: updating ? "Update Workshop item" : "Publish to the Workshop"
	padding: Kirigami.Units.largeSpacing
	preferredWidth: Kirigami.Units.gridUnit * 30

	onOpened: {
		previewPath = ""
		badLink = false
		changeNote.text = ""
		linkField.text = project.workshopId
		visibilityBox.currentIndex = 0
	}

	FileDialog {
		id: previewDialog
		title: "Choose a preview image"
		nameFilters: ["Images (*.png *.jpg *.jpeg *.gif)"]
		onAccepted: dialog.previewPath = applicationWindow().urlToPath(selectedFile)
	}

	ColumnLayout {
		spacing: Kirigami.Units.largeSpacing

		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Warning
			visible: !dialog.steamReady
			text: "Steam isn't connected. Start Steam and sign in to publish."
		}
		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Error
			visible: dialog.badLink
			text: "That isn't a Workshop item ID or link."
		}

		Kirigami.FormLayout {
			Layout.fillWidth: true

			Controls.TextField {
				id: linkField
				Kirigami.FormData.label: "Workshop item:"
				Layout.fillWidth: true
				placeholderText: "New item (or paste an ID or link to update one)"
				onTextEdited: dialog.badLink = false
			}
			Controls.ComboBox {
				id: visibilityBox
				Kirigami.FormData.label: "Visibility:"
				textRole: "text"
				valueRole: "value"
				model: [
					{ text: dialog.updating ? "Keep as it is" : "Private (default)", value: dialog.updating ? "" : "private" },
					{ text: "Private", value: "private" },
					{ text: "Friends only", value: "friends" },
					{ text: "Unlisted", value: "unlisted" },
					{ text: "Public", value: "public" }
				]
			}
			RowLayout {
				Kirigami.FormData.label: "Preview image:"

				Image {
					visible: dialog.previewPath !== ""
					source: dialog.previewPath === "" ? "" : applicationWindow().pathToUrl(dialog.previewPath)
					sourceSize.width: Kirigami.Units.gridUnit * 4
					sourceSize.height: Kirigami.Units.gridUnit * 4
					fillMode: Image.PreserveAspectFit
				}
				Controls.Label {
					visible: dialog.previewPath === ""
					text: dialog.updating ? "Keep the current image" : "gmpublisher's icon"
					opacity: 0.7
				}
				Controls.Button {
					text: "Choose…"
					icon.name: "insert-image"
					onClicked: previewDialog.open()
				}
			}
			Controls.TextArea {
				id: changeNote
				Kirigami.FormData.label: "Change notes:"
				Layout.fillWidth: true
				Layout.minimumHeight: Kirigami.Units.gridUnit * 4
				visible: dialog.updating
				wrapMode: TextEdit.Wrap
				placeholderText: "What changed in this update (optional)"
			}
		}

		Controls.Label {
			Layout.fillWidth: true
			wrapMode: Text.Wrap
			opacity: 0.7
			text: !dialog.updating
				? "Creates a new Workshop item titled “" + dialog.project.title + "”. You can change the title and description on its Workshop page later."
				: "Uploads the current files to this item. Its title and description stay as they are on the Workshop."
		}
	}

	customFooterActions: [
		Kirigami.Action {
			text: dialog.updating ? "Upload update" : "Publish"
			icon.name: "cloud-upload"
			enabled: dialog.steamReady
			onTriggered: {
				if (linkField.text.trim() !== dialog.project.workshopId && !dialog.project.linkWorkshopItem(linkField.text)) {
					dialog.badLink = true
					linkField.forceActiveFocus()
					return
				}
				dialog.project.publish(visibilityBox.currentValue, dialog.previewPath, changeNote.text)
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
