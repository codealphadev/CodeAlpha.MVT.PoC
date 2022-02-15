import Foundation

class ServiceDelegate: NSObject, NSXPCListenerDelegate {
    let aXServerXPC: AXServerXPC

    init(xCodeAXState: XCodeAXState, globalAXState: GlobalAXState) {
        self.aXServerXPC = AXServerXPC(xCodeAXState: xCodeAXState, globalAXState: globalAXState)
    }

    func listener(_: NSXPCListener, shouldAcceptNewConnection newConnection: NSXPCConnection) -> Bool {
        let exportedObject = aXServerXPC
        newConnection.exportedInterface = NSXPCInterface(with: AXServerXPCProtocol.self)
        newConnection.exportedObject = exportedObject
        newConnection.resume()
        return true
    }
}
