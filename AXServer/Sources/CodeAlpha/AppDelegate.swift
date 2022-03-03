import AXSwift
import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {
  let consoleIO = ConsoleIO()
  func applicationDidFinishLaunching(_: Notification) {}

  func applicationWillTerminate(_: Notification) {
    // Insert code here to tear down your application
  }

  func applicationSupportsSecureRestorableState(_: NSApplication) -> Bool {
    return true
  }
}
