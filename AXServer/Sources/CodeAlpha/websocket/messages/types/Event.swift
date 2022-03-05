import Foundation
import Vapor

enum EventType: String, Codable {
  case AppFocusState
  case XCodeEditorContent
  case XCodeFocusStatus
  case XCodeFocusStatusChange
}

struct Event<T: Codable>: Codable {
  let eventType: EventType
  let payload: T

  enum CodingKeys: String, CodingKey {
    case eventType = "event"
    case payload
  }
}

extension ByteBuffer {
  func decodeEventMessage<T: Codable>(_: T.Type) -> Event<T>? {
    let decoder = JSONDecoder()
    decoder.keyDecodingStrategy = .convertFromSnakeCase
    return try? decoder.decode(Event<T>.self, from: self)
  }
}
  
 
 
