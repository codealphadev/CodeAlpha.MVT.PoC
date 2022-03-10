import Foundation
import Vapor

enum RequestType: String, Codable {
  case GetXCodeEditorContent
  case UpdateXCodeEditorContent
  case GetXCodeFocusStatus
  case GetAppFocusState
  case Connect
}

struct Request<T: Codable>: Codable {
  let requestType: RequestType
  let payload: T?

  enum CodingKeys: String, CodingKey {
    case requestType = "request"
    case payload
  }
}

extension ByteBuffer {
  func decodeRequestMessage<T: Codable>(_: T.Type) -> Request<T>? {
    let decoder = JSONDecoder()
    decoder.keyDecodingStrategy = .convertFromSnakeCase
    return try? decoder.decode(Request<T>.self, from: self)
  }
}

extension Request {
  init(from decoder: Decoder) throws {
    let container = try decoder.container(keyedBy: CodingKeys.self)
    requestType = try container.decode(RequestType.self,
                                       forKey: .requestType)
    payload = try container.decodeIfPresent(T.self,
                                            forKey: .payload)
  }

  func encode(to encoder: Encoder) throws {
    var container = encoder.container(keyedBy: CodingKeys.self)

    try container.encode(requestType, forKey: .requestType)
    try container.encodeIfPresent(payload, forKey: .payload)
  }
}