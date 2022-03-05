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
		createTimer()
	}

	public func notifyAppFocusStatus() {
		if let previousApp = previousFocusedApp, let currentApp = currentFocusedApp {
			let appFocusState = AppFocusState(previousApp: previousApp, currentApp: currentApp)
			websocketManager.notify(message: Event(eventType: .AppFocusState, payload: appFocusState))
		}
	}

	func createTimer() {
		websocketManager.loop.scheduleRepeatedTask(initialDelay: .seconds(0), delay: .milliseconds(100)) { _ in
			self.updateState()
		}
	}

	func updateState() {
		var currentAppPID: Int32 = 0
		do {
			guard let window = try systemWideElement.attribute(.focusedUIElement) as UIElement? else { return }
			currentAppPID = try window.pid()
		} catch {
			consoleIO.writeMessage("Error: Could not read focused window: \(error)", to: .error)
			return
		}

		// Seemingly NSWorkspace.shared.frontmostApplication does NOT give us the correct Application,
		// presumably because we don't run the app in the main thread 🤷‍♂️
		let appForPid = NSWorkspace.shared.runningApplications.first { $0.processIdentifier == currentAppPID }

		let currentApp = AppInfo(
			bundleId: appForPid?.bundleIdentifier ?? "unknown",
			name: appForPid?.localizedName ?? "unknown",
			pid: currentAppPID,
			isFinishedLaunching: appForPid?.isFinishedLaunching ?? false
		)

		if currentFocusedApp?.pid == currentApp.pid {
			return
		} else {
			previousFocusedApp = currentFocusedApp
		}
		// only continue when app has changed
		currentFocusedApp = currentApp

		notifyAppFocusStatus()
	}
}
