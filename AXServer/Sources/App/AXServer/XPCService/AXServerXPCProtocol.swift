import Foundation

@objc(AXServerXPCProtocol) protocol AXServerXPCProtocol {
	func startAnonymousListener(_ endpoint: NSXPCListenerEndpoint, withReply reply: @escaping (Bool) -> Void)
	func stopAnonymousListener(withReply reply: @escaping (Bool) -> Void)
}
