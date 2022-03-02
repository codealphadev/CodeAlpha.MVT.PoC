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
    print("Added client app: \(client.id)")
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
