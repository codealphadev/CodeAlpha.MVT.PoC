import AXSwift
import Cocoa

class AXAppDelegate: NSObject, NSApplicationDelegate {
    var axTrusted: Bool = false

    func applicationDidFinishLaunching(_: Notification) {
        guard UIElement.isProcessTrusted(withPrompt: true) else {
            NSLog("No accessibility API permission, exiting")
            NSRunningApplication.current.terminate()
            return
        }

        // Check that we have permission to access accessibility features
        axTrusted = UIElement.isProcessTrusted(withPrompt: true)

        if UIElement.isProcessTrusted(withPrompt: true) {
            axTrusted = true
        }

        _ = XCodeAXState()
    }

    func applicationWillTerminate(_: Notification) {
        // Insert code here to tear down your application
    }

    func applicationSupportsSecureRestorableState(_: NSApplication) -> Bool {
        return true
    }

    // Provide method to trigger the accessibility popup in case the user clicked "deny" the first time it opened.
    func triggerAccessibilityPopup() {
        axTrusted = UIElement.isProcessTrusted(withPrompt: true)
    }
}
