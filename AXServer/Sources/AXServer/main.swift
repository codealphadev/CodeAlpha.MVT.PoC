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

// Register XPC listener to receive requests from clients
let machServiceName = "com.codeAlpha.AXServerXPC"
let delegate = ServiceDelegate()
let listener = NSXPCListener(machServiceName: machServiceName)
listener.delegate = delegate
listener.resume()

application.run()
