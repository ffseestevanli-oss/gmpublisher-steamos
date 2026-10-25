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
	Kirigami.InlineMessage {
		Layout.fillWidth: true
		type: Kirigami.MessageType.Information
		visible: status.task.busy
		text: status.task.status + "…"
	}
	Controls.ProgressBar {
		Layout.fillWidth: true
		visible: status.task.busy
		indeterminate: status.task.progress < 0
		value: Math.max(0, status.task.progress)
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
