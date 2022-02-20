import AXSwift
import Cocoa

class GlobalAXState {
	let consoleIO = ConsoleIO()

	var observerGlobalFocus: Observer?
	var appGlobalFocus: Application?

	var currentFocusedAppPid: Int32?
	var previousFocusedAppPid: Int32?

	var anonymousClientService: AXClientXPCProtocol?

	init() {
		do {
			try updateState()
		} catch {
			consoleIO.writeMessage("Error: Could not update Global State", to: .error)
		}
		createTimer()
	}

	public func setXPCService(_ service: AXClientXPCProtocol?) {
		anonymousClientService = service
	}

	public func focusAppPID() -> Int32? {
		do {
			let pid = try appGlobalFocus?.pid()
			return pid
		} catch {
			consoleIO.writeMessage("Error: Could not read PID of app: \(error)", to: .error)
			return nil
		}
	}

	public func getCurrentFocusedAppPid() -> Int32? {
		return currentFocusedAppPid
	}

	public func getPreviousFocusedAppPid() -> Int32? {
		return previousFocusedAppPid
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

	@objc func updateState() throws {
		guard let focusedWindow = try systemWideElement.attribute(.focusedUIElement) as UIElement? else { return }
		guard let app = Application(forProcessID: try focusedWindow.pid()) else { return }

		// only continue when app has changed
		if appGlobalFocus == app {
			return
		} else {
			appGlobalFocus = app
		}

		previousFocusedAppPid = currentFocusedAppPid
		currentFocusedAppPid = try app.pid()

		if let application = NSWorkspace.shared.frontmostApplication {
			if let unwrappedAnonymousClientService = anonymousClientService {
				unwrappedAnonymousClientService.notifyAppFocusChange("\(String(describing: application.localizedName))") { _ in
					self.consoleIO.writeMessage("localizedName: \(String(describing: application.localizedName))")
				}
			}
		}

		var updated = false
		observerGlobalFocus = app.createObserver { (_: Observer, _: UIElement, _: AXNotification, _: [String: AnyObject]?) in
			// do {
			// 	var elementDesc: String!
			// 	if let role = try? element.role()!, role == .window {
			// 		elementDesc = "\(element) '\(try (element.attribute(.title) as String?)!)'"
			// 	} else {
			// 		elementDesc = "\(element)"
			// 	}
			// 	self.consoleIO.writeMessage("\(event) on \(String(describing: elementDesc));", to: .error)
			// } catch {
			// 	self.consoleIO.writeMessage("Error: Could not read type of UIElement [\(element)]: \(error)", to: .error)
			// 	return
			// }

			// Watch events on new windows
			// if event == .mainWindowChanged {
			// 	do {
			// 		try self.observerGlobalFocus!.addNotification(.uiElementDestroyed, forElement: element)
			// 		try self.observerGlobalFocus!.addNotification(.moved, forElement: element)
			// 	} catch {
			// 		self.consoleIO.writeMessage("Error: Could not watch [\(element)]: \(error)", to: .error)
			// 	}
			// }

			// // Group simultaneous events together with --- lines
			// if !updated {
			// 	updated = true
			// 	// Set this code to run after the current run loop, which is dispatching all notifications.
			// 	DispatchQueue.main.async {
			// 		self.consoleIO.writeMessage("---")
			// 		updated = false
			// 	}
			// }
		}

		do {
			try observerGlobalFocus!.addNotification(.windowCreated, forElement: app)
			try observerGlobalFocus!.addNotification(.mainWindowChanged, forElement: app)
			try observerGlobalFocus!.addNotification(.moved, forElement: app)
			try observerGlobalFocus!.addNotification(.focusedWindowChanged, forElement: app)
		} catch {
			consoleIO.writeMessage("Error: Could not add notifications: \(error)", to: .error)
		}
	}
}
