import Foundation

enum XCodeFocusElement: String, Codable {
  case editor
  case app
}

struct XCodeFocusStatusChange: Codable {
  let focusElementChange: XCodeFocusElement
  let isInFocus: Bool
}
