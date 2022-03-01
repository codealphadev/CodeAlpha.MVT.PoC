import Cocoa
import Vapor

public func configure(_ app: Application) throws {
    app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

    let websocketManager = WebsocketManager(eventLoop: app.eventLoopGroup.next())

    app.webSocket("channel") { _, ws in
        websocketManager.connect(ws)
    }

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
}
