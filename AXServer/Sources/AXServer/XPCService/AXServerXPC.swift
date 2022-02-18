import AXSwift
import Foundation

@objc class AXServerXPC: NSObject, AXServerXPCProtocol {
  let consoleIO = ConsoleIO()
  var anonymousXPCService: AXClientXPCProtocol?

  let xCodeAXState = XCodeAXState()
  let globalAXState = GlobalAXState()

  let widgetBundleIdentifier = "com.googlecode.iterm2" // To be refactored later.

  func getXCodeEditorContent(withReply reply: @escaping (String?) -> Void) {
    reply(xCodeAXState.getEditorContent())
  }

  func updateXCodeEditorContent(_ newContent: String, withReply reply: @escaping (String?) -> Void) {
    reply(xCodeAXState.updateEditorContent(newContent))
  }

  func getXCodeAppFocusStatus(withReply reply: @escaping (Bool) -> Void) {
    // Case: XCode is in focus
    if xCodeAXState.isXCodeAppInFocus() {
      reply(true)
    } else {
      // Case: Check if app in Focus right now is the Widget AND app previously in focus was XCode
      // 1. get process Ids (PIDs)
      guard let currentFocusedAppPid = globalAXState.getCurrentFocusedAppPid() else {
        reply(false)
        return
      }
      guard let previousFocusedAppPid = globalAXState.getPreviousFocusedAppPid() else {
        reply(false)
        return
      }

      // 2. check whether XCode AND Widget are running applications
      let xCodeApplication = Application.allForBundleID(xCodeAXState.getXCodeBundleId())

      if xCodeApplication.count == 0 {
        reply(false)
        return
      }
      let widgetApplication = Application.allForBundleID(widgetBundleIdentifier)

      if widgetApplication.count == 0 {
        reply(false)
        return
      }

      // 3. Return true, if XCode was previous PID and current PID is the Widget PID
      do {
        let pidXCode = try xCodeApplication.first!.pid()
        let pidWidget = try widgetApplication.first!.pid()
        if previousFocusedAppPid == pidXCode, currentFocusedAppPid == pidWidget {
          reply(true)
        } else {
          reply(false)
          return
        }
      } catch {
        reply(false)
        return
      }
    }
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

    xCodeAXState.setXPCService(anonymousXPCService)
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
