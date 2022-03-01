import Foundation

class ServiceDelegate: NSObject, NSXPCListenerDelegate {
	let consoleIO = ConsoleIO()
	let axServerXPC = AXServerXPC()

	func listener(_: NSXPCListener, shouldAcceptNewConnection newConnection: NSXPCConnection) -> Bool {
		let exportedObject = axServerXPC
		newConnection.exportedInterface = NSXPCInterface(with: AXServerXPCProtocol.self)
		newConnection.exportedObject = exportedObject
		newConnection.invalidationHandler = { self.consoleIO.writeMessage("Connection did invalidate", to: .error) }
		newConnection.interruptionHandler = { self.consoleIO.writeMessage("Connection did interrupt", to: .error) }
		newConnection.resume()
		return true
	}
}
