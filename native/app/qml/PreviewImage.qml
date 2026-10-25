import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

// A Workshop preview image in a rounded frame. The one about to be uploaded gets a rainbow border.
Rectangle {
	id: frame

	property alias source: image.source
	property bool highlighted: false
	property bool dimmed: false
	property string placeholder: ""

	implicitWidth: Kirigami.Units.gridUnit * 7
	implicitHeight: implicitWidth
	radius: Kirigami.Units.largeSpacing
	gradient: highlighted ? rainbow : null
	color: "#1a1826"
	opacity: dimmed ? 0.55 : 1

	RainbowGradient {
		id: rainbow
	}

	Rectangle {
		anchors.fill: parent
		anchors.margins: frame.highlighted ? 3 : 0
		radius: frame.radius - anchors.margins
		color: "#1a1826"
		clip: true

		Image {
			id: image
			anchors.fill: parent
			sourceSize.width: width
			sourceSize.height: height
			fillMode: Image.PreserveAspectCrop
			asynchronous: true
		}
		Controls.Label {
			anchors.centerIn: parent
			width: parent.width - Kirigami.Units.largeSpacing * 2
			visible: image.status !== Image.Ready
			horizontalAlignment: Text.AlignHCenter
			wrapMode: Text.Wrap
			text: image.status === Image.Loading ? "Loading…" : frame.placeholder
			color: "#a9a4c4"
			font: Kirigami.Theme.smallFont
		}
	}
}
