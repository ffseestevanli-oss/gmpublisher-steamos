import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

// The main action on a page: a rainbow pill with dark text
Controls.Button {
	id: button

	implicitHeight: Math.max(Kirigami.Units.gridUnit * 2.25, implicitContentHeight + topPadding + bottomPadding)
	leftPadding: Kirigami.Units.largeSpacing * 2
	rightPadding: Kirigami.Units.largeSpacing * 2

	contentItem: Controls.Label {
		text: button.text
		color: "#0f0e17"
		font.weight: Font.Bold
		horizontalAlignment: Text.AlignHCenter
		verticalAlignment: Text.AlignVCenter
	}
	background: Rectangle {
		radius: height / 2
		gradient: RainbowGradient {}
		opacity: button.enabled ? (button.down ? 0.8 : 1) : 0.4
		border.width: button.visualFocus ? 2 : 0
		border.color: "#ffffff"
	}
}
