import Foundation

@objc(AXClientXPCProtocol) protocol AXClientXPCProtocol {
  func notifyXCodeEditorContentUpdate(_ content: String?, withReply reply: @escaping (Bool) -> Void)
  func notifyXCodeAppFocusStatus(_ focusStatus: Bool, withReply reply: @escaping (Bool) -> Void)
  func notifyXCodeEditorFocusStatus(_ focusStatus: Bool, withReply reply: @escaping (Bool) -> Void)
  func notifyAppFocusChange(_ focusAppName: String, withReply reply: @escaping (Bool) -> Void)
  func anonymousHeartbeat(_ heartbeat: Bool, withReply reply: @escaping (Bool) -> Void)
}
