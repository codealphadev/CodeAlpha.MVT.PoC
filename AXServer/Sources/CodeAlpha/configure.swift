import Cocoa
import Vapor

public func configure(_ app: Application) throws {
	app.middleware.use(FileMiddleware(publicDirectory: app.directory.publicDirectory))

	// Spin up websocket manager; manages all connected clients
	let websocketManager = WebsocketManager(eventLoop: app.eventLoopGroup.next())

	// Start state listeners
	let xCodeAXState = XCodeAXState(websocketManager)
	let globalAXState = GlobalAXState(websocketManager)

	// Setup Websocket routes
	app.webSocket("channel") { _, ws in

		ws.onBinary { _, buffer in

			// Case: Request<Connect>
			if let msg = buffer.decodeWebsocketMessage(Request<Connect>.self) {
				print(msg)

				if msg.data.payload.connect {
					let wsClient = WebsocketClient(id: msg.client, socket: ws)
					websocketManager.connect(client: wsClient)
				}
			}

			// Case: Request<GetXCodeEditorContent>
			if buffer.decodeWebsocketMessage(Request<XCodeEditorContent>.self) != nil {
				xCodeAXState.notifyEditorContent()
			}

			// Case: Request<UpdateXCodeEditorContent>
			if let msg = buffer.decodeWebsocketMessage(Request<XCodeEditorContent>.self) {
				xCodeAXState.updateEditorContent(msg.data.payload.content)
			}

			// Case: Request<GetXCodeFocusStatus>
			if buffer.decodeWebsocketMessage(Request<XCodeFocusStatus>.self) != nil {
				xCodeAXState.notifyXCodeFocusStatus()
			}

			// Case: Request<GetAppFocusState>
			if buffer.decodeWebsocketMessage(Request<AppFocusState>.self) != nil {
				globalAXState.notifyAppFocusStatus()
			}
		}
		ws.onText { ws, _ in
			ws.send("pong")
		}
	}
}
