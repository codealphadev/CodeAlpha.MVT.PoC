import Foundation

struct AppInfo: Codable {
  let bundleId: String
  let name: String
  let pid: Int32
  let isFinishedLaunching: Bool
}
