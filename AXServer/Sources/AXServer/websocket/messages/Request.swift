import Foundation

enum RequestType: String, Codable {
  case getXCodeEditorContent
  case updateXCodeEditorContent
  case getXCodeFocusStatus
  case getAppFocusState
}

struct Request: Codable {
  let type: RequestType
}
