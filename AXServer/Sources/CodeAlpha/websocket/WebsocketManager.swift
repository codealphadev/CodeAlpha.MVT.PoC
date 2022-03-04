import Vapor

class WebsocketManager {
  let consoleIO = ConsoleIO()
  var clients: WebsocketClients
  let loop: EventLoop

  init(eventLoop: EventLoop) {
    clients = WebsocketClients(eventLoop: eventLoop)
    loop = eventLoop
  }

  func connect(client: WebsocketClient) {
    clients.add(client)
    consoleIO.writeMessage("Added client app: \(client.id)", to: .error)
  }

  func notify<T: Codable>(message: T) {
    // Debug
    let debugWsMessage = WebsocketMessage(client: UUID(), data: EventType(from: AppFocusState(message as! AppFocusState)))
    let data = try! JSONEncoder().encode(debugWsMessage)
    print("\(String(decoding: data, as: UTF8.self))\n")
    print(debugWsMessage)
    // /Debug

    let connectedClients = clients.active.compactMap { $0 as WebsocketClient }
    guard !connectedClients.isEmpty else {
      return
    }

    connectedClients.forEach { client in
      do {
        let wsMessage = WebsocketMessage(client: client.id, data: message)
        let data = try JSONEncoder().encode(wsMessage)
        client.socket.send([UInt8](data))
      } catch {
        consoleIO.writeMessage("Error: Could not encode websocket message: \(error)", to: .error)
      }
    }
  }
}
