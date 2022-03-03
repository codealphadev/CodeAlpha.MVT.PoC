import AXSwift
import Cocoa

class XCodeAXState {
	let xCodeBundleId = "com.apple.dt.Xcode"

	let consoleIO = ConsoleIO()

	var xCodeAXApp: AXSwift.Application?

	var xcodeEditorFocusStatus = false
	var xcodeAppFocusStatus = false

	var lastFocusedXCodeEditorUIElement: UIElement?
	var xcodeEditorContent: String?

	let websocketManager: WebsocketManager

	init(_ wsManager: WebsocketManager) {
		websocketManager = wsManager
		scheduleRepeatedTask_isXCodeAppRunning()
		scheduleRepeatedTask_checkFocusXCodeEditor()
		scheduleRepeatedTask_observeXCodeEditorContentChanges()
	}

	func scheduleRepeatedTask_isXCodeAppRunning() {
		websocketManager.loop.scheduleRepeatedTask(initialDelay: .seconds(0), delay: .milliseconds(100)) { _ in
			_ = self.isXCodeAppRunning()
		}
	}

	func scheduleRepeatedTask_checkFocusXCodeEditor() {
		websocketManager.loop.scheduleRepeatedTask(initialDelay: .seconds(0), delay: .milliseconds(100)) { _ in
			self.notifyXCodeFocusChange()
		}
	}

	func scheduleRepeatedTask_observeXCodeEditorContentChanges() {
		websocketManager.loop.scheduleRepeatedTask(initialDelay: .seconds(0), delay: .milliseconds(100)) { _ in
			self.observeEditorContentChangesHacked()
		}
	}

	public func isXCodeEditorInFocus() -> Bool {
		guard let window = try? systemWideElement.attribute(.focusedUIElement) as UIElement? else {
			return false
		}

		// 1. XCode must be in focus
		if !isXCodeAppInFocus() {
			return false
		}

		// 2. Focus must be on a Text AREA AX element of XCode
		if let uiElementType = try? window.role() {
			if uiElementType == .textArea {
				lastFocusedXCodeEditorUIElement = window
				return true
			}
		}

		// Case: XCode in focus but not on a Text AREA AX element
		return false
	}

	public func isXCodeAppInFocus() -> Bool {
		guard let window = try? systemWideElement.attribute(.focusedUIElement) as UIElement? else {
			return false
		}

		// 1. An instance of XCode must be running
		if !isXCodeAppRunning() {
			return false
		}

		// 2. Compare the PID of the focused window to the PID of the XCode app
		if let windowPID = try? window.pid() {
			if let xCodePID = try? xCodeAXApp?.pid() {
				if windowPID == xCodePID {
					return true
				}
			}
		}

		return false
	}

	public func isXCodeAppRunning() -> Bool {
		// 1. An instance of XCode must be running
		let xCodeApplication = Application.allForBundleID(xCodeBundleId)

		if xCodeApplication.count == 0 {
			xCodeAXApp = nil
			return false
		} else {
			xCodeAXApp = xCodeApplication[0]
			return true
		}
	}

	public func notifyEditorContent() {
		// Logic: If XCode is still running and the editor UI element is known, return its value
		if !isXCodeAppRunning() {
			return
		}

		if let unwrappedXCodeEditorUIElement = lastFocusedXCodeEditorUIElement {
			if let unwrappedContent: String = try? unwrappedXCodeEditorUIElement.attribute(.value) {
				websocketManager.notify(message: XCodeEditorContent(content: unwrappedContent))
			}
		}
	}

	public func notifyXCodeFocusStatus() {
		let appFocusStatus = XCodeFocusStatusChange(focusElementChange: .app, isInFocus: isXCodeAppInFocus())
		let editorFocusStatus = XCodeFocusStatusChange(focusElementChange: .editor, isInFocus: isXCodeEditorInFocus())

		websocketManager.notify(message: XCodeFocusStatus(AppStatus: appFocusStatus, EditorStatus: editorFocusStatus))
	}

	public func notifyXCodeFocusChange() {
		let editorInFocus = isXCodeEditorInFocus()
		let appInFocus = isXCodeAppInFocus()

		// only send notification if focus has changed
		if xcodeEditorFocusStatus != editorInFocus {
			xcodeEditorFocusStatus = editorInFocus

			let xCodeFocusStatusChange = XCodeFocusStatusChange(focusElementChange: .editor, isInFocus: editorInFocus)
			websocketManager.notify(message: xCodeFocusStatusChange)
		}

		if xcodeAppFocusStatus != appInFocus {
			xcodeAppFocusStatus = appInFocus

			let xCodeFocusStatusChange = XCodeFocusStatusChange(focusElementChange: .app, isInFocus: editorInFocus)
			websocketManager.notify(message: xCodeFocusStatusChange)
		}
	}

	public func updateEditorContent(_ newContent: String) {
		// Logic: If XCode is still running and the editor UI element is known, update its value
		if !isXCodeAppRunning() {
			return
		}

		guard let unwrappedXCodeEditorUIElement = lastFocusedXCodeEditorUIElement else { return }

		do {
			try unwrappedXCodeEditorUIElement.setAttribute(.value, value: newContent)
		} catch {
			consoleIO.writeMessage("Error: Could not set value of UIElement [\(unwrappedXCodeEditorUIElement)]: \(error)", to: .error)
			return
		}
	}

	func comparePIDs(_ pidWindow: Int32, _ pidOtherWindow: Int32) -> Bool {
		if pidWindow == pidOtherWindow {
			return true
		} else {
			return false
		}
	}

	// This implementation is a hack to get around the fact that the AXSwift
	// Observer is publishing its events on the main thread which is not used
	// because of the Vapor.
	func observeEditorContentChangesHacked() {
		if !isXCodeEditorInFocus() {
			return
		}

		guard let editorUIElement = lastFocusedXCodeEditorUIElement else {
			return
		}

		if let currentContent: String = try? editorUIElement.attribute(.value) {
			if currentContent != xcodeEditorContent {
				xcodeEditorContent = currentContent
				notifyEditorContent()
			}
		}
	}

	// This method is the correct way to observe changes in the content of a text area.
	// Unfortunately this publishes the changes to the RunLoop of the NSApplication, which is
	// not being listened to by the websocket server.
	//
	// This method is currently not being used. Git history reveals implementation. Removed here to avoid confusion.
	func observeEditorContentChanges() {}
}
