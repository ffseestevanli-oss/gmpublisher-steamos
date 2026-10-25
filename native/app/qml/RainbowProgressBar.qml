import QtQuick
import QtQuick.Controls as Controls
import org.kde.kirigami as Kirigami

// A progress bar filled with the rainbow. Indeterminate progress sweeps a short rainbow back and forth.
Controls.ProgressBar {
	id: bar

	implicitHeight: Kirigami.Units.smallSpacing * 1.5

	background: Rectangle {
		implicitWidth: Kirigami.Units.gridUnit * 10
		radius: height / 2
		color: "#2a2540"
	}
	contentItem: Item {
		id: track
		clip: true

		property real sweep: 0
		SequentialAnimation on sweep {
			running: bar.indeterminate && bar.visible
			loops: Animation.Infinite
			NumberAnimation { from: 0; to: 1; duration: 900; easing.type: Easing.InOutQuad }
			NumberAnimation { from: 1; to: 0; duration: 900; easing.type: Easing.InOutQuad }
		}

		Rectangle {
			id: fill
			height: parent.height
			radius: height / 2
			gradient: RainbowGradient {}
			width: bar.indeterminate ? parent.width / 3 : parent.width * bar.visualPosition
			x: bar.indeterminate ? track.sweep * (track.width - width) : 0
		}
	}
}
