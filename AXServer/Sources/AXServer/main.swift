import Cocoa

// Handle CTRL+C event to terminate the app
signal(SIGINT, SIG_IGN)
let sigint = DispatchSource.makeSignalSource(signal: SIGINT, queue: DispatchQueue.main)
sigint.setEventHandler {
    NSApp.terminate(nil)
}
sigint.resume()


let appDelegate = AXAppDelegate()
let application = NSApplication.shared
application.setActivationPolicy(NSApplication.ActivationPolicy.accessory)
application.delegate = appDelegate
application.run()
