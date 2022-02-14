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

        someLogic()
        NSRunningApplication.current.terminate()
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

    func someLogic() {
        // // Get Active Application
        // if let application = NSWorkspace.shared.frontmostApplication {
        //     NSLog("localizedName: \(String(describing: application.localizedName)), processIdentifier: \(application.processIdentifier)")
        //     let uiApp = Application(application)!
        //     NSLog("windows: \(String(describing: try! uiApp.windows()))")
        //     NSLog("attributes: \(try! uiApp.attributes())")
        //     NSLog("at 0,0: \(String(describing: try! uiApp.elementAtPosition(0, 0)))")
        //     if let bundleIdentifier = application.bundleIdentifier {
        //         NSLog("bundleIdentifier: \(bundleIdentifier)")
        //         let windows = try! Application.allForBundleID(bundleIdentifier).first!.windows()
        //         NSLog("windows: \(String(describing: windows))")
        //     }
        // }

        // // Get Application by bundleIdentifier
        // let app = Application.allForBundleID("com.apple.finder").first!
        // NSLog("finder: \(app)")
        // NSLog("role: \(try! app.role()!)")
        // NSLog("windows: \(try! app.windows()!)")
        // NSLog("attributes: \(try! app.attributes())")
        // if let title: String = try! app.attribute(.title) {
        //     NSLog("title: \(title)")
        // }
        // NSLog("multi: \(try! app.getMultipleAttributes(["AXRole", "asdf", "AXTitle"]))")
        // NSLog("multi: \(try! app.getMultipleAttributes(.role, .title))")

        // // Try to set an unsettable attribute
        // if let window = try! app.windows()?.first {
        //     do {
        //         try window.setAttribute(.title, value: "my title")
        //         let newTitle: String? = try! window.attribute(.title)
        //         NSLog("title set; result = \(newTitle ?? "<none>")")
        //     } catch {
        //         NSLog("error caught trying to set title of window: \(error)")
        //     }
        // }

        // NSLog("system wide:")
        // NSLog("role: \(try! systemWideElement.role()!)")
        // NSLog("windows: \(try! sys.windows())")
        let focusedElement: UIElement? = try! systemWideElement.attribute(.focusedUIElement)
        if let unwrapped = focusedElement {
            // if let title: String = try! unwrapped.attribute(.value) {
            //     NSLog("title: \(title)")
            // }
            NSLog("role: \(try! unwrapped.role()!)")
            NSLog("attributes: \(try! unwrapped.attributesAsStrings())")
        }
    }
}
