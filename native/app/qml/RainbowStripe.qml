import QtQuick
import org.kde.kirigami as Kirigami

// A thin rainbow that keeps scrolling sideways, as a border along the top of the window
Item {
	id: stripe

	implicitHeight: Kirigami.Units.smallSpacing
	clip: true

	// Two rainbows side by side, sliding left by one rainbow and starting over, so it never jumps
	Rectangle {
		id: band
		width: stripe.width * 2
		height: stripe.height
		gradient: Gradient {
			orientation: Gradient.Horizontal
			GradientStop { position: 0 / 14; color: "#ff3b5c" }
			GradientStop { position: 1 / 14; color: "#ff8a00" }
			GradientStop { position: 2 / 14; color: "#ffd400" }
			GradientStop { position: 3 / 14; color: "#2ee86b" }
			GradientStop { position: 4 / 14; color: "#20c5ff" }
			GradientStop { position: 5 / 14; color: "#6a5cff" }
			GradientStop { position: 6 / 14; color: "#c34bff" }
			GradientStop { position: 7 / 14; color: "#ff3b5c" }
			GradientStop { position: 8 / 14; color: "#ff8a00" }
			GradientStop { position: 9 / 14; color: "#ffd400" }
			GradientStop { position: 10 / 14; color: "#2ee86b" }
			GradientStop { position: 11 / 14; color: "#20c5ff" }
			GradientStop { position: 12 / 14; color: "#6a5cff" }
			GradientStop { position: 13 / 14; color: "#c34bff" }
			GradientStop { position: 14 / 14; color: "#ff3b5c" }
		}

		NumberAnimation on x {
			from: 0
			to: -stripe.width
			duration: 6000
			loops: Animation.Infinite
			running: stripe.visible && stripe.width > 0
		}
	}
}
