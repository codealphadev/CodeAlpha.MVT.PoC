import AXSwift
import Foundation

@objc class AXServerXPC: NSObject, AXServerXPCProtocol {
  var endpoint: NSXPCListenerEndpoint = .init()

  let xCodeAXState: XCodeAXState
  let globalAXState: GlobalAXState

  let widgetBundleIdentifier = "com.googlecode.iterm2"
  let xCodeBundleIdentifier = "com.apple.dt.Xcode"

  init(xCodeAXState: XCodeAXState, globalAXState: GlobalAXState) {
    self.xCodeAXState = xCodeAXState
    self.globalAXState = globalAXState
  }

  func registerNSXPCListenerEndpoint(_ endpoint: NSXPCListenerEndpoint) {
    self.endpoint = endpoint
  }

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
      let xCodeApplication = Application.allForBundleID(xCodeBundleIdentifier)

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

  func toRedString(_ text: String, withReply reply: @escaping (String) -> Void) {
    reply("\u{1B}[31m\(text)\u{1B}[0m")
  }
  func toGreenString(_ text: String, withReply reply: @escaping (String) -> Void) {
    reply("\u{1B}[32m\(text)\u{1B}[0m")
  }
}
