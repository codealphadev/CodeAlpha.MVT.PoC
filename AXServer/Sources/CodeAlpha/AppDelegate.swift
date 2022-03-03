import AXSwift
import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {
  let consoleIO = ConsoleIO()
  func applicationDidFinishLaunching(_: Notification) {
    guard UIElement.isProcessTrusted(withPrompt: true) else {
      consoleIO.writeMessage("No accessibility API permission, exiting", to: .error)
      NSRunningApplication.current.terminate()
      return
    }
  }

  func applicationWillTerminate(_: Notification) {
    // Insert code here to tear down your application
  }

  func applicationSupportsSecureRestorableState(_: NSApplication) -> Bool {
    return true
  }
}
