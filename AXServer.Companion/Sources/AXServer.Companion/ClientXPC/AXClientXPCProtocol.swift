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
    aCoder.encode(bundleId, forKey: "bundleId")
    aCoder.encode(name, forKey: "name")
    aCoder.encode(pid, forKey: "pid")
    aCoder.encode(isFinishedLaunching, forKey: "isFinishedLaunching")
  }

  required init?(coder aDecoder: NSCoder) {
    guard
      let bundleId = aDecoder.decodeObject(of: [NSString.self], forKey: "bundleId") as? String,
      let name = aDecoder.decodeObject(of: [NSString.self], forKey: "name") as? String,
      let isFinishedLaunching = aDecoder.decodeBool(forKey: "isFinishedLaunching") as Bool?,
      let pid = aDecoder.decodeInt32(forKey: "pid") as Int32?
    else {
      return nil
    }

    self.bundleId = bundleId
    self.name = name
    self.pid = pid
    self.isFinishedLaunching = isFinishedLaunching
  }

  let bundleId: String
  let name: String
  let pid: Int32
  let isFinishedLaunching: Bool

  init(bundleId: String, name: String, pid: Int32, isFinishedLaunching: Bool) {
    self.bundleId = bundleId
    self.name = name
    self.pid = pid
    self.isFinishedLaunching = isFinishedLaunching
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
