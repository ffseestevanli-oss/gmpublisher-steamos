import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import io.github.gmpublisher

Kirigami.ScrollablePage {
	id: page
	property string path

	title: doc.loaded ? doc.title : "Opening…"

	GmaDocument {
		id: doc
		Component.onCompleted: open(page.path)
	}

	FolderDialog {
		id: extractDialog
		title: "Extract to"
		currentFolder: applicationWindow().pathToUrl(applicationWindow().parentDir(page.path))
		onAccepted: doc.extract(applicationWindow().urlToPath(selectedFolder))
	}

	actions: [
		Kirigami.Action {
			text: "Extract"
			icon.name: "archive-extract"
			tooltip: "Extract next to the file, into " + doc.defaultExtractDir()
			enabled: doc.loaded && !doc.busy
			onTriggered: doc.extract("")
		},
		Kirigami.Action {
			text: "Extract to…"
			icon.name: "document-save-as"
			enabled: doc.loaded && !doc.busy
			onTriggered: extractDialog.open()
		},
		Kirigami.Action {
			text: "Show file"
			icon.name: "document-open-folder"
			displayHint: Kirigami.DisplayHint.AlwaysHide
			onTriggered: Qt.openUrlExternally(applicationWindow().pathToUrl(applicationWindow().parentDir(page.path)))
		}
	]

	header: ColumnLayout {
		spacing: 0

		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Error
			visible: doc.error !== ""
			text: doc.error
		}
		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Warning
			visible: doc.rejectedCount > 0
			text: doc.rejectedCount + " file(s) in this GMA have unsafe paths and will be skipped."
		}
		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Positive
			visible: doc.extractedTo !== ""
			text: "Extracted to " + doc.extractedTo
			actions: [
				Kirigami.Action {
					text: "Open folder"
					icon.name: "folder-open"
					onTriggered: Qt.openUrlExternally(applicationWindow().pathToUrl(doc.extractedTo))
				}
			]
		}
		Controls.ProgressBar {
			Layout.fillWidth: true
			visible: doc.busy
			indeterminate: !doc.loaded
			value: doc.progress
		}
	}

	ListView {
		id: list
		model: doc.loaded ? JSON.parse(doc.entriesJson) : []

		header: Kirigami.FormLayout {
			width: list.width
			visible: doc.loaded

			Controls.Label {
				Kirigami.FormData.label: "File:"
				Layout.fillWidth: true
				text: page.path
				wrapMode: Text.WrapAnywhere
			}
			Controls.Label {
				Kirigami.FormData.label: "Type:"
				text: doc.addonType || "Not set"
			}
			Controls.Label {
				Kirigami.FormData.label: "Tags:"
				text: doc.tags || "None"
			}
			Controls.Label {
				Kirigami.FormData.label: "Contents:"
				text: list.count + " files, " + applicationWindow().formatSize(doc.contentSize)
			}
			Item {
				Kirigami.FormData.isSection: true
			}
		}

		delegate: Controls.ItemDelegate {
			id: entry
			required property var modelData
			width: ListView.view.width

			contentItem: RowLayout {
				Controls.Label {
					Layout.fillWidth: true
					text: entry.modelData.path
					elide: Text.ElideMiddle
				}
				Controls.Label {
					text: applicationWindow().formatSize(entry.modelData.size)
					opacity: 0.7
				}
			}
		}
	}
}
