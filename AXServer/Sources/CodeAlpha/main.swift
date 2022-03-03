import App
import AXSwift
import Cocoa
import Vapor

let consoleIO = ConsoleIO()

enum CustomError: Error {
	case runtimeError(String)
}

// Handle CTRL+C event to terminate the app
signal(SIGINT, SIG_IGN)
let sigint = DispatchSource.makeSignalSource(signal: SIGINT, queue: DispatchQueue.main)
sigint.setEventHandler {
	NSApp.terminate(nil)
}

sigint.resume()

// Register AppDelegate to interact with AX features
let appDelegate = AppDelegate()
let application = NSApplication.shared
application.setActivationPolicy(NSApplication.ActivationPolicy.accessory)
application.delegate = appDelegate

// Configure vapor webserver
var env = try Environment.detect()
try LoggingSystem.bootstrap(from: &env)
let app = Application(env)
defer { app.shutdown() }
try configure(app)

// Spin up websocket server on background thread to not block the main thread
DispatchQueue.global(qos: .background).async {
	do {
		try app.run()
	} catch {
		consoleIO.writeMessage("Error: \(error)", to: .error)
	}
}

// Start main event loop
application.run()
