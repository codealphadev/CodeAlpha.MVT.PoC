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

@objc(Testen) class Testen: NSObject, NSSecureCoding {
  static var supportsSecureCoding: Bool = true

  func encode(with aCoder: NSCoder) {
    aCoder.encode(text, forKey: "text")
  }

  required init?(coder aDecoder: NSCoder) {
    guard
      let text = aDecoder.decodeObject(of: [NSString.self], forKey: "text") as? String
    else {
      return nil
    }

    self.text = text
  }

  var text: String

  override init() {
    text = "unknown"
    super.init()
  }
}

@objc(AXClientXPCProtocol) protocol AXClientXPCProtocol {
  func notifyXCodeEditorContentUpdate(_ content: String?, withReply reply: @escaping (Bool) -> Void)
  func notifyXCodeAppFocusStatus(_ focusStatus: Bool, withReply reply: @escaping (Bool) -> Void)
  func notifyXCodeEditorFocusStatus(_ focusStatus: Bool, withReply reply: @escaping (Bool) -> Void)
  func notifyAppFocusChange(_ previousApp: AppInfo, _ currentApp: AppInfo, withReply reply: @escaping (Bool) -> Void)
  func notifyAppFocusChange2(_ previousApp: Testen, _ currentApp: Testen, withReply reply: @escaping (Bool) -> Void)
  func anonymousHeartbeat(_ heartbeat: Bool, withReply reply: @escaping (Bool) -> Void)
}
