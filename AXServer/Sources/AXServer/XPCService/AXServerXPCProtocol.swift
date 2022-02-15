import Foundation

@objc(AXServerXPCProtocol) protocol AXServerXPCProtocol {
  func registerNSXPCListenerEndpoint(_ endpoint: NSXPCListenerEndpoint)
  func getXCodeEditorContent(withReply reply: @escaping (String?) -> Void)
  func updateXCodeEditorContent(_ newContent: String, withReply reply: @escaping (String?) -> Void)
  func getXCodeAppFocusStatus(withReply reply: @escaping (Bool) -> Void)
  func getXCodeEditorFocusStatus(withReply reply: @escaping (Bool) -> Void)
  func toRedString(_ text: String, withReply reply: @escaping (String) -> Void)
  func toGreenString(_ text: String, withReply reply: @escaping (String) -> Void)
}
