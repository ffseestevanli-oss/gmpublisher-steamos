import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import io.github.gmpublisher

Kirigami.ScrollablePage {
	id: page
	property string folder

	title: project.title || "Addon"

	readonly property var plan: JSON.parse(project.planJson)
	readonly property var problems: JSON.parse(project.problemsJson)
	readonly property var tags: JSON.parse(project.tagsJson)
	readonly property bool canPack: problems.length === 0 && plan.files !== undefined && plan.files.length > 0 && plan.notAllowed.length === 0

	AddonProject {
		id: project
		Component.onCompleted: load(page.folder)
		onTitleChanged: problemsTimer.restart()
		onAddonTypeChanged: problemsTimer.restart()
		onTagsJsonChanged: problemsTimer.restart()
	}

	// Re-check while the user edits, without rescanning the folder on every keystroke
	Timer {
		id: problemsTimer
		interval: 400
		onTriggered: project.refresh()
	}

	FileDialog {
		id: packDialog
		title: "Save GMA as"
		fileMode: FileDialog.SaveFile
		nameFilters: ["Garry's Mod addons (*.gma)"]
		defaultSuffix: "gma"
		currentFolder: applicationWindow().pathToUrl(applicationWindow().parentDir(page.folder))
		onAccepted: project.pack(applicationWindow().urlToPath(selectedFile))
	}

	actions: [
		Kirigami.Action {
			text: "Save addon.json"
			icon.name: "document-save"
			onTriggered: project.save()
		},
		Kirigami.Action {
			text: "Pack GMA"
			icon.name: "package-x-generic"
			tooltip: "Pack into " + project.defaultOutput()
			enabled: page.canPack && !project.busy
			onTriggered: project.pack("")
		},
		Kirigami.Action {
			text: "Pack as…"
			icon.name: "document-save-as"
			enabled: page.canPack && !project.busy
			onTriggered: packDialog.open()
		},
		Kirigami.Action {
			text: "Publish to Workshop"
			icon.name: "cloud-upload"
			enabled: false
			tooltip: "Coming in the next update"
		},
		Kirigami.Action {
			text: "Open folder"
			icon.name: "folder-open"
			displayHint: Kirigami.DisplayHint.AlwaysHide
			onTriggered: Qt.openUrlExternally(applicationWindow().pathToUrl(page.folder))
		}
	]

	header: ColumnLayout {
		spacing: 0

		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Information
			visible: !project.hasAddonJson
			text: "This folder has no addon.json yet. Fill in the details and save to create one."
		}
		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Error
			visible: project.error !== ""
			text: project.error
		}
		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Positive
			visible: project.packedTo !== ""
			text: "Packed into " + project.packedTo
			actions: [
				Kirigami.Action {
					text: "Show file"
					icon.name: "document-open-folder"
					onTriggered: Qt.openUrlExternally(applicationWindow().pathToUrl(applicationWindow().parentDir(project.packedTo)))
				}
			]
		}
		Controls.ProgressBar {
			Layout.fillWidth: true
			visible: project.busy
			value: project.progress
		}
	}

	ColumnLayout {
		spacing: Kirigami.Units.largeSpacing

		Kirigami.FormLayout {
			Layout.fillWidth: true

			Controls.TextField {
				Kirigami.FormData.label: "Title:"
				Layout.fillWidth: true
				text: project.title
				onTextEdited: project.title = text
			}
			Controls.ComboBox {
				id: typeBox
				Kirigami.FormData.label: "Type:"
				model: JSON.parse(project.typesJson)
				currentIndex: model.indexOf(project.addonType)
				displayText: currentIndex < 0 ? "Choose a type" : currentText
				onActivated: project.addonType = currentText
			}
			Flow {
				Kirigami.FormData.label: "Tags (up to two):"
				Layout.fillWidth: true
				spacing: Kirigami.Units.smallSpacing

				Repeater {
					model: JSON.parse(project.allTagsJson)
					delegate: Controls.CheckBox {
						required property string modelData
						text: modelData
						checked: page.tags.includes(modelData)
						enabled: checked || page.tags.length < 2
						onToggled: {
							const tags = page.tags.filter(t => t !== modelData)
							if (checked)
								tags.push(modelData)
							project.tagsJson = JSON.stringify(tags)
						}
					}
				}
			}
			Controls.TextArea {
				Kirigami.FormData.label: "Ignore:"
				Layout.fillWidth: true
				Layout.minimumHeight: Kirigami.Units.gridUnit * 4
				placeholderText: "One pattern per line, e.g. *.md or lua/dev/*"
				text: project.ignoreText
				onTextChanged: if (text !== project.ignoreText) { project.ignoreText = text; problemsTimer.restart() }
			}
		}

		Repeater {
			model: page.problems
			delegate: Kirigami.InlineMessage {
				required property string modelData
				Layout.fillWidth: true
				type: Kirigami.MessageType.Warning
				visible: true
				text: modelData
			}
		}

		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Error
			visible: page.plan.error !== undefined
			text: page.plan.error || ""
		}

		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Error
			visible: page.plan.notAllowed !== undefined && page.plan.notAllowed.length > 0
			text: visible ? "Garry's Mod won't accept these files. Remove them or add them to Ignore:\n" + page.plan.notAllowed.join("\n") : ""
		}

		Kirigami.Heading {
			level: 3
			text: page.plan.files === undefined ? "Checking files…"
				: page.plan.files.length + " files to pack, " + applicationWindow().formatSize(page.plan.totalSize)
				+ (page.plan.ignored.length > 0 ? " (" + page.plan.ignored.length + " ignored)" : "")
		}

		Repeater {
			model: page.plan.files || []
			delegate: RowLayout {
				required property var modelData
				Layout.fillWidth: true

				Controls.Label {
					Layout.fillWidth: true
					text: modelData.path
					elide: Text.ElideMiddle
				}
				Controls.Label {
					text: applicationWindow().formatSize(modelData.size)
					opacity: 0.7
				}
			}
		}
	}
}
