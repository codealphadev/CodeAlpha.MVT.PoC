import Vapor

class WebsocketManager {
  let consoleIO = ConsoleIO()
  var clients: WebsocketClients

  init(eventLoop: EventLoop) {
    clients = WebsocketClients(eventLoop: eventLoop)
  }

  func connect(_ ws: WebSocket) {
    ws.onBinary { [unowned self] ws, buffer in
      if let msg = buffer.decodeWebsocketMessage(Connect.self) {
        let app = WebsocketClient(id: msg.client, socket: ws)
        self.clients.add(app)
        print("Added client app: \(msg.client)")
      }
    }

    ws.onText { ws, _ in
      ws.send("pong")
    }
  }

  func notify<T: Codable>(message: T) {
    let connectedClients = clients.active.compactMap { $0 as WebsocketClient }
    guard !connectedClients.isEmpty else {
      return
    }

    do {
      let data = try JSONEncoder().encode(message)
      connectedClients.forEach { client in
        client.socket.send([UInt8](data))
      }
    } catch {
      consoleIO.writeMessage("Error: Could not encode websocket message: \(error)", to: .error)
    }
  }
}
