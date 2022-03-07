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
    // let debugWsMessage = WebsocketMessage<Request<String>>(client: UUID(), data: Request<String>(requestType: .GetXCodeEditorContent, payload: nil))
    // let data2 = try! JSONEncoder().encode(debugWsMessage)
    // print("\(String(decoding: data2, as: UTF8.self))\n")
    do {
      let data = try JSONEncoder().encode(message)
      consoleIO.writeMessage("\(String(decoding: data, as: UTF8.self))\n")
    } catch {
      // Do nothing
    }
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
