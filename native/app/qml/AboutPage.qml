import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
	title: "About"

	ColumnLayout {
		spacing: Kirigami.Units.largeSpacing

		RowLayout {
			Layout.alignment: Qt.AlignHCenter
			spacing: Kirigami.Units.largeSpacing * 2

			Image {
				source: "../assets/brand/gmpublisher-icon.png"
				sourceSize.width: Kirigami.Units.iconSizes.huge
				sourceSize.height: Kirigami.Units.iconSizes.huge
			}
			Image {
				source: "../assets/brand/steventechno-logo.png"
				sourceSize.width: Kirigami.Units.iconSizes.huge
				sourceSize.height: Kirigami.Units.iconSizes.huge
			}
		}
		Kirigami.Heading {
			Layout.alignment: Qt.AlignHCenter
			text: "gmpublisher"
			font.weight: Font.Black
		}
		Controls.Label {
			Layout.alignment: Qt.AlignHCenter
			textFormat: Text.StyledText
			text: applicationWindow().rainbowText("STEVENTECHNO EDITION")
			font.weight: Font.Bold
			font.letterSpacing: 2
		}
		Controls.Label {
			Layout.fillWidth: true
			horizontalAlignment: Text.AlignHCenter
			wrapMode: Text.Wrap
			text: "gmpublisher was created by William Venner, and all credit for it goes to him. "
				+ "The Steventechno Edition is a Linux-first rewrite for SteamOS, Bazzite and other Linux desktops, "
				+ "built with Rust and KDE's Kirigami."
		}
		Controls.Label {
			Layout.fillWidth: true
			horizontalAlignment: Text.AlignHCenter
			wrapMode: Text.Wrap
			textFormat: Text.StyledText
			text: "<a href=\"https://github.com/WilliamVenner/gmpublisher\">Original gmpublisher</a> · "
				+ "<a href=\"https://github.com/ffseestevanli-oss/gmpublisher-steamos\">This edition</a> · GPL-3.0"
			onLinkActivated: link => Qt.openUrlExternally(link)
		}
	}
}
