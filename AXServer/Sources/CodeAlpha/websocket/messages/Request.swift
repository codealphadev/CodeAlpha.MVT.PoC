import Foundation

enum RequestType: String, Codable {
  case GetXCodeEditorContent
  case UpdateXCodeEditorContent
  case GetXCodeFocusStatus
  case GetAppFocusState
}

struct Request: Codable {
  let requestType: RequestType
}
