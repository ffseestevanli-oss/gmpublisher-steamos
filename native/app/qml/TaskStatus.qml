import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Progress and the outcome of the last WorkshopTask job, for a page header
ColumnLayout {
	id: status
	required property var task
	spacing: 0

	Kirigami.InlineMessage {
		Layout.fillWidth: true
		type: Kirigami.MessageType.Error
		visible: status.task.error !== ""
		text: status.task.error
	}
	// A publish walks through Steam's upload stages; show where it is
	readonly property var steps: [
		{ label: "Prepare", stages: ["Starting", "Packing", "Copying the GMA", "Creating the Workshop item", "Waiting for Steam", "Preparing"] },
		{ label: "Files", stages: ["Preparing the files"] },
		{ label: "Upload", stages: ["Uploading"] },
		{ label: "Preview", stages: ["Uploading the preview image"] },
		{ label: "Finish", stages: ["Finishing"] }
	]
	readonly property int step: steps.findIndex(s => s.stages.includes(task.status))
	// Downloads and extractions use stages of their own
	readonly property bool publishing: task.busy && step >= 0 && task.status !== "Starting"

	Rectangle {
		Layout.fillWidth: true
		Layout.margins: Kirigami.Units.largeSpacing
		visible: status.task.busy
		implicitHeight: busyLayout.implicitHeight + Kirigami.Units.largeSpacing * 2
		radius: Kirigami.Units.largeSpacing
		color: "#17151f"

		ColumnLayout {
			id: busyLayout
			anchors.fill: parent
			anchors.margins: Kirigami.Units.largeSpacing
			spacing: Kirigami.Units.smallSpacing * 2

			RowLayout {
				Layout.fillWidth: true
				visible: status.publishing
				spacing: Kirigami.Units.smallSpacing

				Repeater {
					model: status.steps
					delegate: RowLayout {
						id: stepItem
						required property var modelData
						required property int index
						readonly property bool done: index < status.step
						readonly property bool current: index === status.step
						spacing: Kirigami.Units.smallSpacing

						Rectangle {
							implicitWidth: Kirigami.Units.gridUnit
							implicitHeight: implicitWidth
							radius: width / 2
							color: stepItem.done ? "#2ee86b" : stepItem.current ? "#ffd400" : "#2a2540"

							Controls.Label {
								anchors.centerIn: parent
								text: stepItem.done ? "✓" : String(stepItem.index + 1)
								color: stepItem.done || stepItem.current ? "#0f0e17" : "#a9a4c4"
								font.pixelSize: Kirigami.Theme.smallFont.pixelSize
								font.weight: Font.Bold
							}
						}
						Controls.Label {
							text: stepItem.modelData.label
							color: stepItem.done ? "#9fe8b8" : stepItem.current ? "#f4f2ff" : "#a9a4c4"
							font.weight: stepItem.current ? Font.Bold : Font.Normal
						}
						Rectangle {
							visible: stepItem.index < status.steps.length - 1
							implicitWidth: Kirigami.Units.gridUnit
							implicitHeight: 2
							color: "#3a3550"
						}
					}
				}
			}
			Controls.Label {
				Layout.fillWidth: true
				text: status.task.status + "…" + (status.task.progress >= 0 ? "  " + Math.round(status.task.progress * 100) + "%" : "")
				elide: Text.ElideRight
			}
			RainbowProgressBar {
				Layout.fillWidth: true
				indeterminate: status.task.progress < 0
				value: Math.max(0, status.task.progress)
			}
		}
	}
	Kirigami.InlineMessage {
		Layout.fillWidth: true
		type: Kirigami.MessageType.Positive
		visible: status.task.publishedId !== ""
		text: (status.task.created ? "Created Workshop item " + status.task.publishedId + ". New items start out private until you change their visibility."
				: "Saved your changes to Workshop item " + status.task.publishedId + ". Steam can take a few minutes to show them.")
			+ (status.task.needsLegalAgreement ? " Steam needs you to accept the Workshop legal agreement before others can see it." : "")
		actions: [
			Kirigami.Action {
				text: "Open in Steam"
				icon.name: "internet-web-browser"
				onTriggered: Qt.openUrlExternally("steam://url/CommunityFilePage/" + status.task.publishedId)
			},
			Kirigami.Action {
				text: "Legal agreement"
				icon.name: "documentinfo"
				visible: status.task.needsLegalAgreement
				onTriggered: Qt.openUrlExternally("https://steamcommunity.com/sharedfiles/workshoplegalagreement")
			}
		]
	}
	Kirigami.InlineMessage {
		Layout.fillWidth: true
		type: Kirigami.MessageType.Positive
		visible: status.task.savedTo !== ""
		text: (status.task.extracted ? "Extracted to " : "Saved ") + status.task.savedTo
		actions: [
			Kirigami.Action {
				text: status.task.extracted ? "Open folder" : "Show file"
				icon.name: "folder-open"
				onTriggered: Qt.openUrlExternally(applicationWindow().pathToUrl(status.task.extracted
					? status.task.savedTo : applicationWindow().parentDir(status.task.savedTo)))
			}
		]
	}
}
