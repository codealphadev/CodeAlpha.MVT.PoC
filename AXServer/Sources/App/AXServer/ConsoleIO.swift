//
//  ConsoleIO.swift
//  AXWrap.Companion
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

let SUBFOLDER_NAME = "axserver-xpc-logs"
let LOG_FILE_NAME = "AXServer"

enum OutputType {
  case error
  case standard
}

struct StderrOutputStream: TextOutputStream {
  mutating func write(_ string: String) {
    fputs(string, stderr)
  }
}

struct FileHandlerOutputStream: TextOutputStream {
  private let fileHandle: FileHandle
  let encoding: String.Encoding = .utf8

  init?(subfolderName: String, logFileName: String) {
    let documents = NSSearchPathForDirectoriesInDomains(.documentDirectory, .userDomainMask, true)[0]
    let documentsFolderURL = URL(fileURLWithPath: documents)
    let subfolderURL = documentsFolderURL.appendingPathComponent(subfolderName)
    if !FileManager.default.fileExists(atPath: subfolderURL.path) {
      do {
        try FileManager.default.createDirectory(at: subfolderURL, withIntermediateDirectories: true, attributes: nil)
      } catch {
        print("Error: Could not create folder at \(subfolderURL.path)")
        return nil
      }
    }
    let logFileURL = subfolderURL.appendingPathComponent("[\(Date())]-\(logFileName).log")
    do {
      fileHandle = try FileHandle(forWritingTo: logFileURL)
    } catch {
      FileManager.default.createFile(atPath: logFileURL.path, contents: nil, attributes: nil)
      fileHandle = try! FileHandle(forWritingTo: logFileURL)
    }
  }

  mutating func write(_ string: String) {
    if let data = string.data(using: encoding) {
      if #available(macOS 10.15.4, *) {
        do {
          try fileHandle.seekToEnd()
          fileHandle.write(data)
        } catch {
          print("Error: Could not write to file handle")
        }
      } else {
        fileHandle.write(data)
      }
    }
  }
}

class ConsoleIO {
  var standardError = StderrOutputStream()
  var fileHandleOutput: FileHandlerOutputStream?

  init() {
    fileHandleOutput = FileHandlerOutputStream(subfolderName: SUBFOLDER_NAME, logFileName: LOG_FILE_NAME)
  }

  func writeMessage(_ message: String, to: OutputType = .standard) {
    switch to {
    case .standard:
      print("\u{001B}[;m\(message)")
    case .error:
      print("\u{001B}[0;31m\(message)\n", to: &standardError)
      if var unwrappedFileHandleOutput = fileHandleOutput {
        print("\(Date()): \(message)", to: &unwrappedFileHandleOutput)
      }
    }
  }

  func printUsage() {
    _ = (CommandLine.arguments[0] as NSString).lastPathComponent

    writeMessage("usage:")
  }

  func getInput() -> String {
    let keyboard = FileHandle.standardInput
    let inputData = keyboard.availableData
    let strData = String(data: inputData, encoding: String.Encoding.utf8)!
    return strData.trimmingCharacters(in: CharacterSet.newlines)
  }
}
