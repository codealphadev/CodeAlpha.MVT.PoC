import AXSwift
import Cocoa

class GlobalAXState {
	let consoleIO = ConsoleIO()

	var observerGlobalFocus: Observer?

	var currentFocusedApp: AppInfo?
	var previousFocusedApp: AppInfo?

	var anonymousClientService: AXClientXPCProtocol?

	init() {
		updateState()
		createTimer()
	}

	public func setXPCService(_ service: AXClientXPCProtocol?) {
		anonymousClientService = service
	}

	func createTimer() {
		_ = Timer.scheduledTimer(
			timeInterval: 0.1,
			target: self,
			selector: #selector(updateState),
			userInfo: nil,
			repeats: true
		)
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
			if let unwrappedAnonymousClientService = anonymousClientService {
				unwrappedAnonymousClientService.notifyAppFocusChange(application, currentApp) { _ in
					self.consoleIO.writeMessage("PreviousApp: \(String(describing: application.name)) CurrentApp: \(String(describing: currentApp.name))")
				}
			}
		}

		// Add observer for app in global focus -- not needed right now but might be useful later
		guard let unwrappedCurrentUIApp = currentUIApp else { return }

		observerGlobalFocus = unwrappedCurrentUIApp.createObserver { (_: Observer, _: UIElement, _: AXNotification, _: [String: AnyObject]?) in
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
