import Foundation

@objc(AppInfo) class AppInfo: NSObject {
  let bundleId: String
  let name: String
  let pid: Int32
  let isFinishedLaunching: Bool

  init(bundleId: String, name: String, pid: Int32, isFinishedLaunching: Bool) {
    self.bundleId = bundleId
    self.name = name
    self.pid = pid
    self.isFinishedLaunching = isFinishedLaunching
  }
}

@objc(AXClientXPCProtocol) protocol AXClientXPCProtocol {
  func notifyXCodeEditorContentUpdate(_ content: String?, withReply reply: @escaping (Bool) -> Void)
  func notifyXCodeAppFocusStatus(_ focusStatus: Bool, withReply reply: @escaping (Bool) -> Void)
  func notifyXCodeEditorFocusStatus(_ focusStatus: Bool, withReply reply: @escaping (Bool) -> Void)
  func notifyAppFocusChange(_ previousApp: AppInfo, _ currentApp: AppInfo, withReply reply: @escaping (Bool) -> Void)
  func anonymousHeartbeat(_ heartbeat: Bool, withReply reply: @escaping (Bool) -> Void)
}
