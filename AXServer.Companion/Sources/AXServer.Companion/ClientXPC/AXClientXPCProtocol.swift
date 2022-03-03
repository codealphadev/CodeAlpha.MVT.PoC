import Foundation

@objc(AXClientXPCProtocol) protocol AXClientXPCProtocol {
  func anonymousHeartbeat(_ heartbeat: Bool, withReply reply: @escaping (Bool) -> Void)
}
