import Foundation

enum RequestType: Codable {
  case GetXCodeEditorContent(String)
  case UpdateXCodeEditorContent(XCodeEditorContent)
  case GetXCodeFocusStatus(String)
  case GetAppFocusState(String)
  case Connect(Connect)
}

struct Request: Codable {
  let requestType: RequestType
}
