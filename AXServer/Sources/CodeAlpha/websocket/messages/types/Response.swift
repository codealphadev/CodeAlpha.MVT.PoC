import Foundation

enum ResponseType: Codable {
  case resp(String)
}

struct Response: Codable {
  let responseType: ResponseType
}
