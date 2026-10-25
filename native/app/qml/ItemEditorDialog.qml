import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Creates a Workshop item, or edits one of the user's items (see workshop_task.rs)
Kirigami.Dialog {
	id: dialog

	required property var task
	// One entry of Backend.myItemsJson, or null for a new item
	property var item: null
	readonly property bool creating: item === null
	readonly property bool steamReady: applicationWindow().backend.connected
	readonly property var types: JSON.parse(task.addonTypes())
	readonly property var allTags: JSON.parse(task.addonTags())

	// What the item has now, to send only what changed
	property string originalType: ""
	property var originalTags: []

	property string previewPath: ""
	property string addonType: ""
	property var tags: []
	property string contentPath: ""
	property string problem: ""

	title: creating ? "New Workshop item" : "Edit “" + item.title + "”"
	padding: Kirigami.Units.largeSpacing
	preferredWidth: Kirigami.Units.gridUnit * 34

	function openFor(workshopItem) {
		item = workshopItem
		const itemTags = workshopItem ? workshopItem.tags.map(t => t.toLowerCase()) : []
		originalType = itemTags.find(t => types.includes(t)) || ""
		originalTags = itemTags.filter(t => allTags.includes(t))
		addonType = originalType
		tags = originalTags.slice()
		titleField.text = workshopItem ? workshopItem.title : ""
		descriptionField.text = workshopItem ? workshopItem.description : ""
		visibilityBox.currentIndex = 0
		contentBox.currentIndex = 0
		contentPath = ""
		previewPath = ""
		changeNote.text = ""
		descriptionTabs.currentIndex = 0
		problem = ""
		open()
	}

	readonly property string contentKind: contentBox.currentValue || "keep"

	// Fill in what an addon folder or GMA says about itself
	function useContent(path) {
		contentPath = path
		const info = task.readAddon(path)
		if (info === "")
			return
		const addon = JSON.parse(info)
		if (types.includes(addon.type))
			addonType = addon.type
		tags = (addon.tags || []).filter(t => allTags.includes(t)).slice(0, 2)
		if (creating && titleField.text.trim() === "")
			titleField.text = addon.title || ""
		if (creating && descriptionField.text.trim() === "")
			descriptionField.text = addon.description || ""
	}

	// Steam's BBCode as rich text, close enough to judge the layout before publishing
	function bbcodeToHtml(code) {
		let html = code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
		const simple = { b: "b", i: "i", u: "u", strike: "s", h1: "h2", h2: "h3", h3: "h4", code: "pre", quote: "blockquote" }
		for (const tag in simple)
			html = html.replace(new RegExp("\\[" + tag + "\\]([\\s\\S]*?)\\[/" + tag + "\\]", "gi"), "<" + simple[tag] + ">$1</" + simple[tag] + ">")
		html = html
			.replace(/\[url=([^\]]+)\]([\s\S]*?)\[\/url\]/gi, "<a href=\"$1\">$2</a>")
			.replace(/\[url\]([\s\S]*?)\[\/url\]/gi, "<a href=\"$1\">$1</a>")
			.replace(/\[img\][\s\S]*?\[\/img\]/gi, "<i>[image]</i>")
			.replace(/\[spoiler\]([\s\S]*?)\[\/spoiler\]/gi, "<span style=\"background-color:#3a3550\">$1</span>")
			.replace(/\[hr\]\s*\[\/hr\]|\[hr\]/gi, "<hr>")
			.replace(/\[(o?)list\]([\s\S]*?)\[\/\1list\]/gi, (_, ordered, items) => {
				const tag = ordered ? "ol" : "ul"
				return "<" + tag + ">" + items.split(/\[\*\]/).slice(1).map(item => "<li>" + item.trim() + "</li>").join("") + "</" + tag + ">"
			})
		// Line breaks, except the ones right next to block elements
		return html.replace(/\n*(<\/?(?:h\d|ul|ol|li|pre|blockquote|hr)>)\n*/g, "$1").replace(/\n/g, "<br>")
	}

	function sameTags(a, b) {
		return a.length === b.length && a.every(t => b.includes(t))
	}

	function submit() {
		if (titleField.text.trim() === "")
			return problem = "Give the addon a title."
		if (addonType === "" && (creating || contentKind !== "keep"))
			return problem = "Choose the addon's type."
		if (contentKind !== "keep" && contentPath === "")
			return problem = contentKind === "folder" ? "Choose the addon folder to upload." : "Choose the GMA to upload."

		const request = {
			id: creating ? "" : item.id,
			gmaTitle: titleField.text.trim(),
			visibility: visibilityBox.currentValue,
			preview: previewPath,
			content: contentKind,
			contentPath: contentPath,
			changeNote: changeNote.text
		}
		if (creating || titleField.text.trim() !== item.title)
			request.title = titleField.text.trim()
		if (creating || descriptionField.text !== item.description)
			request.description = descriptionField.text
		// Packing a folder always writes the type and tags into the GMA, so send them along too
		if (addonType !== "" && (creating || contentKind === "folder" || addonType !== originalType || !sameTags(tags, originalTags))) {
			request.type = addonType
			request.tags = tags
		}
		task.submit(JSON.stringify(request))
		close()
	}

	FileDialog {
		id: previewDialog
		title: "Choose a preview image"
		nameFilters: ["Images (*.png *.jpg *.jpeg *.gif)"]
		onAccepted: dialog.previewPath = applicationWindow().urlToPath(selectedFile)
	}
	FolderDialog {
		id: folderDialog
		title: "Choose the addon folder"
		onAccepted: dialog.useContent(applicationWindow().urlToPath(selectedFolder))
	}
	FileDialog {
		id: gmaDialog
		title: "Choose the GMA to upload"
		nameFilters: ["Garry's Mod addons (*.gma *.bin)", "All files (*)"]
		onAccepted: dialog.useContent(applicationWindow().urlToPath(selectedFile))
	}

	ColumnLayout {
		spacing: Kirigami.Units.largeSpacing

		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Warning
			visible: !dialog.steamReady
			text: "Steam isn't connected. Start Steam and sign in to publish."
		}
		Kirigami.InlineMessage {
			Layout.fillWidth: true
			type: Kirigami.MessageType.Error
			visible: dialog.problem !== ""
			text: dialog.problem
		}

		Kirigami.FormLayout {
			Layout.fillWidth: true

			Controls.ComboBox {
				id: contentBox
				Kirigami.FormData.label: "Files:"
				Layout.fillWidth: true
				textRole: "text"
				valueRole: "value"
				model: (dialog.creating ? [] : [{ text: "Keep the files that are on the Workshop", value: "keep" }]).concat([
					{ text: "Pack and upload an addon folder", value: "folder" },
					{ text: "Upload a GMA file", value: "gma" }
				])
				onActivated: {
					dialog.contentPath = ""
					dialog.problem = ""
				}
			}
			RowLayout {
				Kirigami.FormData.label: dialog.contentKind === "folder" ? "Addon folder:" : "GMA file:"
				Layout.fillWidth: true
				visible: dialog.contentKind !== "keep"

				Controls.Label {
					Layout.fillWidth: true
					text: dialog.contentPath || "None chosen"
					elide: Text.ElideMiddle
					opacity: dialog.contentPath === "" ? 0.7 : 1
				}
				Controls.Button {
					text: "Choose…"
					icon.name: dialog.contentKind === "folder" ? "folder-open" : "document-open"
					onClicked: (dialog.contentKind === "folder" ? folderDialog : gmaDialog).open()
				}
			}
			Controls.TextField {
				id: titleField
				Kirigami.FormData.label: "Title:"
				Layout.fillWidth: true
				maximumLength: 128
			}
			Controls.ComboBox {
				Kirigami.FormData.label: "Type:"
				model: dialog.types
				currentIndex: model.indexOf(dialog.addonType)
				displayText: currentIndex < 0 ? "Choose a type" : currentText
				onActivated: dialog.addonType = currentText
			}
			Flow {
				Kirigami.FormData.label: "Tags (up to two):"
				Layout.fillWidth: true
				spacing: Kirigami.Units.smallSpacing

				Repeater {
					model: dialog.allTags
					delegate: Controls.CheckBox {
						required property string modelData
						text: modelData
						checked: dialog.tags.includes(modelData)
						enabled: checked || dialog.tags.length < 2
						onToggled: {
							const tags = dialog.tags.filter(t => t !== modelData)
							if (checked)
								tags.push(modelData)
							dialog.tags = tags
						}
					}
				}
			}
			Controls.ComboBox {
				id: visibilityBox
				Kirigami.FormData.label: "Visibility:"
				textRole: "text"
				valueRole: "value"
				model: [
					{ text: dialog.creating ? "Private (default)" : "Keep as it is", value: dialog.creating ? "private" : "" },
					{ text: "Private", value: "private" },
					{ text: "Friends only", value: "friends" },
					{ text: "Unlisted", value: "unlisted" },
					{ text: "Public", value: "public" }
				]
			}
			// What the Workshop shows now, next to the image that will replace it
			RowLayout {
				Kirigami.FormData.label: "Preview image:"
				spacing: Kirigami.Units.largeSpacing

				ColumnLayout {
					visible: !dialog.creating
					spacing: Kirigami.Units.smallSpacing

					PreviewImage {
						source: dialog.creating ? "" : dialog.item.previewUrl
						dimmed: dialog.previewPath !== ""
					}
					Controls.Label {
						text: "On the Workshop now"
						color: applicationWindow().rainbow.muted
						font: Kirigami.Theme.smallFont
					}
				}
				Kirigami.Icon {
					visible: !dialog.creating && dialog.previewPath !== ""
					source: "go-next"
					implicitWidth: Kirigami.Units.iconSizes.medium
					implicitHeight: implicitWidth
					color: "#ffd400"
				}
				ColumnLayout {
					visible: dialog.creating || dialog.previewPath !== ""
					spacing: Kirigami.Units.smallSpacing

					PreviewImage {
						source: dialog.previewPath !== "" ? applicationWindow().pathToUrl(dialog.previewPath) : ""
						highlighted: dialog.previewPath !== ""
						placeholder: "gmpublisher's icon"
					}
					Controls.Label {
						Layout.maximumWidth: Kirigami.Units.gridUnit * 8
						text: dialog.previewPath !== "" ? dialog.previewPath.substring(dialog.previewPath.lastIndexOf("/") + 1) : "No image chosen"
						color: applicationWindow().rainbow.muted
						font: Kirigami.Theme.smallFont
						elide: Text.ElideMiddle
					}
				}
				ColumnLayout {
					Layout.alignment: Qt.AlignVCenter
					spacing: Kirigami.Units.smallSpacing

					Controls.Button {
						text: dialog.previewPath === "" ? "Choose…" : "Change…"
						icon.name: "insert-image"
						onClicked: previewDialog.open()
					}
					Controls.Button {
						visible: dialog.previewPath !== ""
						text: "Undo"
						icon.name: "edit-undo"
						onClicked: dialog.previewPath = ""
					}
				}
			}
			ColumnLayout {
				Kirigami.FormData.label: "Description:"
				Layout.fillWidth: true
				spacing: 0

				Controls.TabBar {
					id: descriptionTabs
					Controls.TabButton {
						text: "Write"
						width: implicitWidth
					}
					Controls.TabButton {
						text: "Preview"
						width: implicitWidth
					}
				}
				Controls.TextArea {
					id: descriptionField
					Layout.fillWidth: true
					Layout.minimumHeight: Kirigami.Units.gridUnit * 6
					visible: descriptionTabs.currentIndex === 0
					wrapMode: TextEdit.Wrap
					placeholderText: "Shown on the Workshop page. Steam BBCode works here."
				}
				// Roughly how Steam will show the BBCode
				Controls.Frame {
					Layout.fillWidth: true
					Layout.minimumHeight: Kirigami.Units.gridUnit * 6
					visible: descriptionTabs.currentIndex === 1

					Controls.Label {
						width: parent.width
						textFormat: Text.RichText
						wrapMode: Text.Wrap
						text: descriptionField.text.trim() === "" ? "<i>Nothing to preview yet.</i>" : dialog.bbcodeToHtml(descriptionField.text)
						onLinkActivated: link => Qt.openUrlExternally(link)
					}
				}
			}
			Controls.TextArea {
				id: changeNote
				Kirigami.FormData.label: "Change notes:"
				Layout.fillWidth: true
				Layout.minimumHeight: Kirigami.Units.gridUnit * 3
				visible: !dialog.creating
				wrapMode: TextEdit.Wrap
				placeholderText: "What changed in this update (optional)"
			}
		}

		Controls.Label {
			Layout.fillWidth: true
			wrapMode: Text.Wrap
			opacity: 0.7
			text: dialog.creating
				? "Creates a new item on the Workshop. It stays private until you choose another visibility."
				: dialog.contentKind === "keep"
					? "Only the details you change are sent to Steam. The addon's files stay as they are."
					: "Replaces the addon's files on the Workshop. Players get the new version the next time Steam updates it."
		}
	}

	customFooterActions: [
		Kirigami.Action {
			text: dialog.creating ? "Publish" : "Save changes"
			icon.name: "cloud-upload"
			enabled: dialog.steamReady && !dialog.task.busy
			onTriggered: dialog.submit()
		},
		Kirigami.Action {
			text: "Cancel"
			icon.name: "dialog-cancel"
			onTriggered: dialog.close()
		}
	]
}
