import Foundation

enum XCodeFocusElement: String, Codable {
  case Editor
  case App
}

struct XCodeFocusStatusChange: Codable {
  let focusElementChange: XCodeFocusElement
  let isInFocus: Bool
  let uiElementX: Float
  let uiElementY: Float
  let uiElementW: Float
  let uiElementH: Float
}
