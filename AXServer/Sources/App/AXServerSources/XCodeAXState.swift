import AXSwift
import Cocoa

class XCodeAXState {
	let xCodeBundleId = "com.apple.dt.Xcode"

	let consoleIO = ConsoleIO()

	var xCodeApp: AXSwift.Application?
	var observerContent: AXSwift.Observer?

	var xcodeEditorFocusStatus = false
	var xcodeAppFocusStatus = false

	var lastFocusedXCodeEditorUIElement: UIElement?
	var xcodeEditorContent: String?

	let websocketManager: WebsocketManager

	init(_ wsManager: WebsocketManager) {
		websocketManager = wsManager
		timerCheckFocusXCodeEditor()
		timerFetchXCodeApplication()
		timerObserveXCodeEditorContentChanges()
	}

	public func isXCodeEditorInFocus() -> Bool {
		// 1. XCode must be in focus
		if !isXCodeAppInFocus() {
			return false
		}

		guard let unwrappedFocusedWindow = try? systemWideElement.attribute(.focusedUIElement) as UIElement? else {
			return false
		}

		// 2. Focus must be on a Text AREA AX element of XCode
		var focusFieldIsTextArea = false
		do {
			let uiElementType = try unwrappedFocusedWindow.role()

			if uiElementType == .textArea {
				focusFieldIsTextArea = true
				lastFocusedXCodeEditorUIElement = unwrappedFocusedWindow
			} else {}
		} catch {
			consoleIO.writeMessage("Error: Could not read type of UIElement [\(unwrappedFocusedWindow)]: \(error)", to: .error)
			return false
		}

		return focusFieldIsTextArea
	}

	public func isXCodeAppInFocus() -> Bool {
		guard let unwrappedFocusedWindow = try? systemWideElement.attribute(.focusedUIElement) as UIElement? else {
			return false
		}

		// 1. An instance of XCode must be running
		guard let app = xCodeApp else {
			return false
		}

		// 2. XCode must be in focus
		var focusAppIsXCode = false
		do {
			focusAppIsXCode = try comparePIDs(unwrappedFocusedWindow.pid(), app.pid())
		} catch {
			consoleIO.writeMessage("Error: Could not read PID of app [\(unwrappedFocusedWindow)]: \(error)", to: .error)
			return false
		}

		return focusAppIsXCode
	}

	public func isXCodeAppRunning() -> Bool {
		// 1. An instance of XCode must be running
		let xCodeApplication = Application.allForBundleID(xCodeBundleId)

		if xCodeApplication.count == 0 {
			xCodeApp = nil
			return false
		} else {
			xCodeApp = xCodeApplication[0]
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

	func timerObserveXCodeEditorContentChanges() {
		websocketManager.loop.scheduleRepeatedTask(initialDelay: .seconds(0), delay: .milliseconds(100)) { _ in
			self.observeEditorContentChangesHacked()
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
	func observeEditorContentChanges() throws {
		guard let unwrappedApp = xCodeApp else { return }
		var updated = false

		// 1. Create an observer for the XCode editor
		observerContent = unwrappedApp.createObserver { (_: Observer, element: UIElement, event: AXNotification, _: [String: AnyObject]?) in
			self.consoleIO.writeMessage("Observer: \(event.rawValue)", to: .error)
			// 2. Logic for handling "valueChanged" event in the XCode editor
			if event == .valueChanged {
				// Focus must be on a text area AX UIElement of XCode
				do {
					if !(try element.attributeIsSupported(.role)) {
						return
					}
					let uiElementType = try element.role()
					if uiElementType == .textArea {
						// // Group simultaneous events together with
						if !updated {
							updated = true

							self.notifyEditorContent()

							// Set this code to run after the current run loop, which is dispatching all notifications.
							DispatchQueue.main.async {
								updated = false
							}
						}
					}
				} catch {
					self.consoleIO.writeMessage("Error: Could not read type of UIElement [\(element)]: \(error)", to: .error)
					return
				}
			}
		}

		// 3. Register notification "valueChanged" at observer
		try observerContent!.addNotification(.valueChanged, forElement: unwrappedApp)
	}

	func timerFetchXCodeApplication() {
		websocketManager.loop.scheduleRepeatedTask(initialDelay: .seconds(0), delay: .milliseconds(100)) { _ in
			self.fetchXCodeApplication()
		}
	}

	func fetchXCodeApplication() {
		// Fetch the XCode application _again_ to later compare it with the previous one
		let xCodeApplication = Application.allForBundleID(xCodeBundleId)

		// Check if the XCode application is still running, stop here already if not
		if xCodeApplication.count == 0 {
			xCodeApp = nil
			return
		}

		// only continue when app has changed
		if xCodeApp == xCodeApplication.first! {
			return
		} else {
			xCodeApp = xCodeApplication.first!
			do {
				try observeEditorContentChanges()
			} catch {
				consoleIO.writeMessage("Error: Could not observe content: \(error)", to: .error)
				xCodeApp = nil
			}
		}
	}

	func timerCheckFocusXCodeEditor() {
		websocketManager.loop.scheduleRepeatedTask(initialDelay: .seconds(0), delay: .milliseconds(100)) { _ in
			self.checkFocusXCodeEditor()
		}
	}

	func checkFocusXCodeEditor() {
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
}
