import Foundation

class AXClientServiceDelegate: NSObject, NSXPCListenerDelegate {
    let consoleIO = ConsoleIO()

    func listener(_: NSXPCListener, shouldAcceptNewConnection newConnection: NSXPCConnection) -> Bool {
        let exportedObject = AXClientXPC()
        newConnection.exportedInterface = NSXPCInterface(with: AXClientXPCProtocol.self)
        newConnection.exportedObject = exportedObject
        newConnection.invalidationHandler = { self.consoleIO.writeMessage("Connection did invalidate", to: .error) }
        newConnection.interruptionHandler = { self.consoleIO.writeMessage("Connection did interrupt", to: .error) }
        newConnection.resume()
        return true
    }
}
