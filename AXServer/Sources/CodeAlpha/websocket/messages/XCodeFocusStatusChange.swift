import Foundation

enum XCodeFocusElement: String, Codable {
  case Editor
  case App
}

struct XCodeFocusStatusChange: Codable {
  let focusElementChange: XCodeFocusElement
  let isInFocus: Bool
}
