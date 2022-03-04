import Foundation

struct XCodeFocusStatus: Codable {
  let appStatus: XCodeFocusStatusChange
  let editorStatus: XCodeFocusStatusChange
}
