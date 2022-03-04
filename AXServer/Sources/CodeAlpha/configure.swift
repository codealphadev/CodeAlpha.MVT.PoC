import Cocoa
import Vapor

public func configure(_ app: Application) throws {
	app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

	// // Register AppDelegate to interact with AX features
	// let appDelegate = AppDelegate()
	// let application = NSApplication.shared
	// application.setActivationPolicy(NSApplication.ActivationPolicy.accessory)
	// application.delegate = appDelegate

	// Spin up websocket manager; manages all connected clients
	let websocketManager = WebsocketManager(eventLoop: app.eventLoopGroup.next())

	// Start state listeners
	let xCodeAXState = XCodeAXState(websocketManager)
	let globalAXState = GlobalAXState(websocketManager)

	// Setup Websocket routes
	app.webSocket("channel") { _, ws in

		ws.onBinary { _, buffer in
			print(buffer.getString(at: 0, length: 120))

			if let msg = buffer.decodeWebsocketMessage(Request.self) {
				print(msg)
				switch msg.data.requestType {
				case let .Connect(connectStatus):
					if connectStatus.connect {
						let wsClient = WebsocketClient(id: msg.client, socket: ws)
						websocketManager.connect(client: wsClient)
					}
				case .GetXCodeEditorContent:
					xCodeAXState.notifyEditorContent()
				case .GetXCodeFocusStatus:
					xCodeAXState.notifyXCodeFocusStatus()
				case .GetAppFocusState:
					globalAXState.notifyAppFocusStatus()
				case let .UpdateXCodeEditorContent(editorContent):
					xCodeAXState.updateEditorContent(editorContent.content)
				}
			}
		}
		ws.onText { ws, _ in
			ws.send("pong")
		}
	}
}
