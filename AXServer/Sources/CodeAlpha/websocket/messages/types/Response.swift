import Foundation
import Vapor

enum ResponseType: String, Codable {
  case resp
}

struct Response<T: Codable>: Codable {
  let responseType: ResponseType
  let payload: T

  enum CodingKeys: String, CodingKey {
    case responseType = "response"
    case payload
  }
}

extension ByteBuffer {
  func decodeResponseMessage<T: Codable>(_: T.Type) -> Response<T>? {
    let decoder = JSONDecoder()
    decoder.keyDecodingStrategy = .convertFromSnakeCase
    return try? decoder.decode(Response<T>.self, from: self)
  }
}
