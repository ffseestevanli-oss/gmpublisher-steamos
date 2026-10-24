import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import io.github.gmpublisher

Kirigami.ApplicationWindow {
	id: root
	title: "gmpublisher"
	width: 800
	height: 600

	Backend {
		id: backend
		Component.onCompleted: start()
		onSteamStatusChanged: console.warn("steamStatus changed:", JSON.stringify(steamStatus))
	}

	Timer {
		interval: 5000; running: true; repeat: true
		onTriggered: console.warn("debug:", JSON.stringify(backend.steamStatus), JSON.stringify(backend.gmodDir), backend.subscribedItems, statusLabel.width, statusLabel.height, statusLabel.visible, statusLabel.color)
	}

	pageStack.initialPage: Kirigami.ScrollablePage {
		title: "gmpublisher (native preview)"

		ColumnLayout {
			spacing: Kirigami.Units.largeSpacing

			Kirigami.InlineMessage {
				Layout.fillWidth: true
				visible: true
				text: "Native Qt edition, early preview. All credit for gmpublisher goes to William Venner."
			}

			Kirigami.FormLayout {
				Layout.fillWidth: true

				Controls.Label {
					id: statusLabel
					Kirigami.FormData.label: "Steam:"
					text: backend.steamStatus
					Layout.maximumWidth: Kirigami.Units.gridUnit * 20
					wrapMode: Text.Wrap
				}
				Controls.Label {
					Kirigami.FormData.label: "Signed in as:"
					text: backend.persona || "–"
				}
				Controls.Label {
					Kirigami.FormData.label: "Subscribed Workshop items:"
					text: backend.subscribedItems >= 0 ? backend.subscribedItems : "–"
				}
				Controls.Label {
					Kirigami.FormData.label: "Garry's Mod:"
					Layout.fillWidth: true
					text: backend.gmodDir || "Not found yet"
					wrapMode: Text.WrapAnywhere
				}
			}
		}
	}
}
