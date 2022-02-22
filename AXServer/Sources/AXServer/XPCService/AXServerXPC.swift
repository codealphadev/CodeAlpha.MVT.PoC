import AXSwift
import Foundation

@objc class AXServerXPC: NSObject, AXServerXPCProtocol {
  let consoleIO = ConsoleIO()
  var anonymousXPCService: AXClientXPCProtocol?

  let xCodeAXState = XCodeAXState()
  let globalAXState = GlobalAXState()

  func getXCodeEditorContent(withReply reply: @escaping (String?) -> Void) {
    reply(xCodeAXState.getEditorContent())
  }

  func updateXCodeEditorContent(_ newContent: String, withReply reply: @escaping (String?) -> Void) {
    reply(xCodeAXState.updateEditorContent(newContent))
  }

  func getXCodeAppFocusStatus(withReply reply: @escaping (Bool) -> Void) {
    reply(xCodeAXState.isXCodeAppInFocus())
  }

  func getXCodeEditorFocusStatus(withReply reply: @escaping (Bool) -> Void) {
    reply(xCodeAXState.isXCodeEditorInFocus())
  }

  func startAnonymousListener(_ endpoint: NSXPCListenerEndpoint, withReply reply: @escaping (Bool) -> Void) {
    let connection = NSXPCConnection(listenerEndpoint: endpoint)
    connection.remoteObjectInterface = NSXPCInterface(with: AXClientXPCProtocol.self)
    connection.resume()

    if #available(macOS 10.11, *) {
      anonymousXPCService = connection.synchronousRemoteObjectProxyWithErrorHandler { error in
        self.consoleIO.writeMessage("Received error: \(error.localizedDescription)", to: .error)
      } as? AXClientXPCProtocol
    } else {
      anonymousXPCService = connection.remoteObjectProxyWithErrorHandler { error in
        self.consoleIO.writeMessage("Received error: \(error.localizedDescription)", to: .error)
      } as? AXClientXPCProtocol
    }

    // I have no idea why, but without this line, the anonymous XPC service will not work.
    anonymousXPCService?.anonymousHeartbeat(true) { _ in
      self.consoleIO.writeMessage("Anonymous XPC service started", to: .error)
    }

    // xCodeAXState.setXPCService(anonymousXPCService)
    globalAXState.setXPCService(anonymousXPCService)

    reply(true)
  }

  func stopAnonymousListener(withReply reply: @escaping (Bool) -> Void) {
    anonymousXPCService = nil
    xCodeAXState.setXPCService(anonymousXPCService)
    globalAXState.setXPCService(anonymousXPCService)
    reply(true)
  }
}
