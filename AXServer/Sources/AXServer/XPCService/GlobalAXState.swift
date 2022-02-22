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
				consoleIO.writeMessage("Error: Could not read focused window???", to: .error)
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
			consoleIO.writeMessage("DEBUG #1 -- PID 1: \(String(describing: currentFocusedApp?.pid)), PID 2: \(currentApp.pid) ", to: .error)
			return
		} else {
			consoleIO.writeMessage("Previous previous app: \(String(describing: previousFocusedApp?.name)) (\(String(describing: previousFocusedApp?.bundleId))) \(String(describing: previousFocusedApp?.pid)) \(String(describing: previousFocusedApp?.isFinishedLaunching)) ", to: .error)
			consoleIO.writeMessage("Previous app: \(String(describing: currentFocusedApp?.name)) (\(String(describing: currentFocusedApp?.bundleId))) \(String(describing: currentFocusedApp?.pid)) \(String(describing: currentFocusedApp?.isFinishedLaunching)) ", to: .error)
			consoleIO.writeMessage("Current app: \(currentApp.name) (\(currentApp.bundleId)) \(currentApp.pid) \(currentApp.isFinishedLaunching) ", to: .error)
			previousFocusedApp = currentFocusedApp
		}
		// only continue when app has changed

		currentFocusedApp = currentApp

		consoleIO.writeMessage("DEBUG #2", to: .error)
		if let application = previousFocusedApp {
			consoleIO.writeMessage("DEBUG #3", to: .error)
			if let unwrappedAnonymousClientService = anonymousClientService {
				consoleIO.writeMessage("DEBUG #4", to: .error)

				let t1 = Testen()
				t1.text = "t1"
				let t2 = Testen()
				t2.text = "t2"
				unwrappedAnonymousClientService.notifyAppFocusChange2(t1, t2) { _ in
					// unwrappedAnonymousClientService.notifyAppFocusChange(application, currentApp) { _ in
					// self.consoleIO.writeMessage("localizedName: \(String(describing: currentApp.name))", to: .error)
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
