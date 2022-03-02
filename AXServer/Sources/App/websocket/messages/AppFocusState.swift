import Foundation

struct AppFocusState: Codable {
  let previousApp: AppInfo
  let currentApp: AppInfo
}
