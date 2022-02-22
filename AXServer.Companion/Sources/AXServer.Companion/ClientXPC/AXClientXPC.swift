import Foundation

@objc class AXClientXPC: NSObject, AXClientXPCProtocol {
    let consoleIO = ConsoleIO()

    func notifyXCodeEditorContentUpdate(_ content: String?, withReply reply: @escaping (Bool) -> Void) {
        if let unwrappedContent = content {
            consoleIO.writeMessage("The content of the focused editor is: \n\n '\(unwrappedContent)'", to: .error)
        } else {
            consoleIO.writeMessage("The content of the focused editor could not be fetched.", to: .error)
        }

        reply(true)
    }

    func notifyXCodeAppFocusStatus(_ focusStatus: Bool, withReply reply: @escaping (Bool) -> Void) {
        consoleIO.writeMessage("AXClientXPC: notifyAppFocusStatus: \(focusStatus)", to: .error)
        reply(true)
    }

    func notifyXCodeEditorFocusStatus(_ focusStatus: Bool, withReply reply: @escaping (Bool) -> Void) {
        consoleIO.writeMessage("AXClientXPC: notifyEditorFocusStatus: \(focusStatus)", to: .error)
        reply(true)
    }

    func notifyAppFocusChange(_ previousApp: AppInfo, _ currentApp: AppInfo, withReply reply: @escaping (Bool) -> Void) {
        consoleIO.writeMessage("AXClientXPC: notifyAppFocusChange: \(previousApp.name) -> \(currentApp.name)", to: .error)
        reply(true)
    }

    func anonymousHeartbeat(_: Bool, withReply reply: @escaping (Bool) -> Void) {
        // consoleIO.writeMessage("AXClientXPC: anonymousHeartbeat: \(heartbeat)")
        reply(true)
    }
}
