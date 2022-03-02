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

if UIElement.isProcessTrusted(withPrompt: true) {
	var env = try Environment.detect()
	try LoggingSystem.bootstrap(from: &env)
	let app = Application(env)
	defer { app.shutdown() }
	try configure(app)
	try app.run()
} else {
	consoleIO.writeMessage("No accessibility API permission, exiting", to: .error)
	throw CustomError.runtimeError("No accessibility API permission, exiting")
}
