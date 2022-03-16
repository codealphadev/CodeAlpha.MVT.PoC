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
                if let payload = msg.data.payload {
					if payload.connect {
						let wsClient = WebsocketClient(id: msg.client, socket: ws)
						websocketManager.connect(client: wsClient)
					}
				}
			}

			// Case: Request<GetXCodeEditorContent>
			if let msg = buffer.decodeWebsocketMessage(Request<String>.self) {
				switch msg.data.requestType {
				case .GetXCodeEditorContent:
					xCodeAXState.notifyEditorContent()
				case .GetXCodeFocusStatus:
					xCodeAXState.notifyXCodeFocusStatus()
				case .GetAppFocusState:
					globalAXState.notifyAppFocusStatus()
				default:
					break
				}
			}

			// Case: Request<UpdateXCodeEditorContent>
			if let msg = buffer.decodeWebsocketMessage(Request<XCodeEditorContent>.self) {
				if let payload = msg.data.payload {
					xCodeAXState.updateEditorContent(payload.content)
				}
			}
		}
	}
}
