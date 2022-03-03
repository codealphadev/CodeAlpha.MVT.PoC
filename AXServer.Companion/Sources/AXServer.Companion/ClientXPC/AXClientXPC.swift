import Foundation

@objc class AXClientXPC: NSObject, AXClientXPCProtocol {
    let consoleIO = ConsoleIO()

    func anonymousHeartbeat(_: Bool, withReply reply: @escaping (Bool) -> Void) {
        // consoleIO.writeMessage("AXClientXPC: anonymousHeartbeat: \(heartbeat)")
        reply(true)
    }
}
