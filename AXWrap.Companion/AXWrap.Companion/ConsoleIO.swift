//
//  ConsoleIO.swift
//  AXWrap.Companion
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

enum OutputType {
  case error
  case standard
}

class ConsoleIO {
    
    func writeMessage(_ message: String, to: OutputType = .standard) {
        switch to {
        case .standard:
          // 1
          print("\u{001B}[;m\(message)")
        case .error:
          // 2
          fputs("\u{001B}[0;31m\(message)\n", stderr)
        }
    }
    
    func printUsage() {

      let executableName = (CommandLine.arguments[0] as NSString).lastPathComponent
            
        writeMessage("usage:")
        writeMessage("\(executableName) -listen")
        writeMessage("or")
        writeMessage("\(executableName) -get -focus")
        writeMessage("or")
        writeMessage("\(executableName) -get -content")
        writeMessage("or")
        writeMessage("\(executableName) -send \"Content string\"")
        writeMessage("or")
        writeMessage("Type \(executableName) without an option to enter interactive mode.")
    }
    
    func getInput() -> String {
      
      let keyboard = FileHandle.standardInput
      let inputData = keyboard.availableData
      let strData = String(data: inputData, encoding: String.Encoding.utf8)!
      return strData.trimmingCharacters(in: CharacterSet.newlines)
    }

}

