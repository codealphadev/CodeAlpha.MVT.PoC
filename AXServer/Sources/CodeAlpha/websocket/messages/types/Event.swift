import Foundation

enum EventType: Codable {
  case AppFocusState(AppFocusState)
  case XCodeEditorContent(XCodeEditorContent)
  case XCodeFocusStatus(XCodeFocusStatus)
  case XCodeFocusStatusChange(XCodeFocusStatusChange)
}

struct Event: Codable {
  let eventType: EventType
}
