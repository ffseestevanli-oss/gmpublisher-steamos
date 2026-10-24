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
					Kirigami.FormData.label: "Steam:"
					text: backend.steamStatus
					color: backend.connected ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.neutralTextColor
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
					text: backend.gmodDir || "Not found yet"
					wrapMode: Text.WrapAnywhere
				}
			}
		}
	}
}
