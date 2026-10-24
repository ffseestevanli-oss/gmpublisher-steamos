import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
	title: "About"

	ColumnLayout {
		spacing: Kirigami.Units.largeSpacing

		Kirigami.Icon {
			Layout.alignment: Qt.AlignHCenter
			source: "io.github.ffseestevanli_oss.gmpublisher"
			implicitWidth: Kirigami.Units.iconSizes.huge
			implicitHeight: Kirigami.Units.iconSizes.huge
		}
		Kirigami.Heading {
			Layout.alignment: Qt.AlignHCenter
			text: "gmpublisher for Linux"
		}
		Controls.Label {
			Layout.fillWidth: true
			horizontalAlignment: Text.AlignHCenter
			wrapMode: Text.Wrap
			text: "gmpublisher was created by William Venner, and all credit for it goes to him. "
				+ "This edition is a Linux-first rewrite for SteamOS, Bazzite and other Linux desktops, "
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
