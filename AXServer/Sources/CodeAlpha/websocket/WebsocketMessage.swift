import Vapor

struct WebsocketMessage<T: Codable>: Codable {
  let client: UUID
  let data: T
}

extension ByteBuffer {
  func decodeWebsocketMessage<T: Codable>(_: T.Type) -> WebsocketMessage<T>? {
    let decoder = JSONDecoder()
    decoder.keyDecodingStrategy = .convertFromSnakeCase
    return try? decoder.decode(WebsocketMessage<T>.self, from: self)
  }
}
