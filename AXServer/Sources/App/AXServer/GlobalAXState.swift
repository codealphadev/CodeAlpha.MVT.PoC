import AXSwift
import Cocoa

class GlobalAXState {
	let consoleIO = ConsoleIO()

	var observerGlobalFocus: Observer?

	var currentFocusedApp: AppInfo?
	var previousFocusedApp: AppInfo?

	let websocketManager: WebsocketManager

	init(_ wsManager: WebsocketManager) {
		websocketManager = wsManager
		updateState()
		createTimer()
	}

	public func notifyAppFocusStatus() {
		if let previousApp = previousFocusedApp, let currentApp = currentFocusedApp {
			let appFocusState = AppFocusState(previousApp: previousApp, currentApp: currentApp)
			websocketManager.notify(message: appFocusState)
		}
	}

	func createTimer() {
		websocketManager.loop.scheduleRepeatedTask(initialDelay: .seconds(0), delay: .milliseconds(100)) { _ in
			self.updateState()
		}
	}

	@objc func updateState() {
		var focusedWindow: UIElement?
		var currentAppPID: Int32 = 0

		do {
			focusedWindow = try systemWideElement.attribute(.focusedUIElement) as UIElement?
			guard let window = focusedWindow else {
				consoleIO.writeMessage("Error: Could not read UIElement of focused window", to: .error)
				return
			}
			currentAppPID = try window.pid()

		} catch {
			consoleIO.writeMessage("Error: Could not read focused window: \(error)", to: .error)
			return
		}

		let currentUIApp = Application(forProcessID: currentAppPID)
		let currentApp = AppInfo(
			bundleId: NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "unknown",
			name: NSWorkspace.shared.frontmostApplication?.localizedName ?? "unknown",
			pid: currentAppPID,
			isFinishedLaunching: NSWorkspace.shared.frontmostApplication?.isFinishedLaunching ?? false
		)

		if currentFocusedApp?.pid == currentApp.pid {
			return
		} else {
			previousFocusedApp = currentFocusedApp
		}
		// only continue when app has changed
		currentFocusedApp = currentApp

		if let application = previousFocusedApp {
			let appFocusState = AppFocusState(previousApp: application, currentApp: currentApp)
			websocketManager.notify(message: appFocusState)
		}

		// Add observer for app in global focus -- not needed right now but might be useful later
		guard let unwrappedCurrentUIApp = currentUIApp else { return }

		observerGlobalFocus = unwrappedCurrentUIApp.createObserver { (_: Observer, _: UIElement, _: AXNotification, _: [String: AnyObject]?) in
			// Closure for when one of the events listed below happens - not implemented yet
		}

		do {
			try observerGlobalFocus!.addNotification(.windowCreated, forElement: unwrappedCurrentUIApp)
			try observerGlobalFocus!.addNotification(.mainWindowChanged, forElement: unwrappedCurrentUIApp)
			try observerGlobalFocus!.addNotification(.moved, forElement: unwrappedCurrentUIApp)
			try observerGlobalFocus!.addNotification(.focusedWindowChanged, forElement: unwrappedCurrentUIApp)
		} catch {
			consoleIO.writeMessage("Error: Could not add notifications: \(error)", to: .error)
		}
	}
}
