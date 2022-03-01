import Foundation

@objc(AXServerXPCProtocol) protocol AXServerXPCProtocol {
	func startAnonymousListener(_ endpoint: NSXPCListenerEndpoint, withReply reply: @escaping (Bool) -> Void)
	func stopAnonymousListener(withReply reply: @escaping (Bool) -> Void)
	func getXCodeEditorContent(withReply reply: @escaping (String?) -> Void)
	func updateXCodeEditorContent(_ newContent: String, withReply reply: @escaping (String?) -> Void)
	func getXCodeAppFocusStatus(withReply reply: @escaping (Bool) -> Void)
	func getXCodeEditorFocusStatus(withReply reply: @escaping (Bool) -> Void)
}
