//
//  AXWrap-Companion.swift
//  AXWrap.Companion
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

enum OptionType: String {
    case listen = "l"
    case getFocus = "gf"
    case getContent = "gc"
    case send = "s"
    case quit = "q"
    case unknown
  
  init(value: String) {
    switch value {
    case "s": self = .send
    case "gc": self = .getContent
    case "gf": self = .getFocus
    case "l": self = .listen
    case "q": self = .quit
    default: self = .unknown
    }
  }
}

class AXWrapCompanion {
    
    let consoleIO = ConsoleIO()
    
    func interactiveMode() {
        
        consoleIO.writeMessage("This app is used to interact with the AXWrap service via XPC protocol.")
        //2
        var shouldQuit = false
        while !shouldQuit {
          //3
          consoleIO.writeMessage("Type 'l' to listen for events.")
          consoleIO.writeMessage("Type 'gf' to get information about the currently focused window.")
            consoleIO.writeMessage("Type 'gc' to fetch the content of the focused editor.")
            consoleIO.writeMessage("Type 's' to update the editor's content.")
            consoleIO.writeMessage("Or type 'q' to quit.")
                                 
          let (option, value) = getOption(consoleIO.getInput())
           
          switch option {
          case .listen:
              consoleIO.writeMessage("Case LISTEN not yet implemented")
          case .getContent:
              consoleIO.writeMessage("Case GETCONTENT not yet implemented")
          case .getFocus:
              consoleIO.writeMessage("Case GETFOCUS not yet implemented")
          case .send:
              
            consoleIO.writeMessage("Type the text to be sent to the editor:")
            let updateText = consoleIO.getInput()
            
              consoleIO.writeMessage("Case SEND not yet implemented; The following text is not sent: '\(updateText)'")
        case .quit:
            shouldQuit = true
          default:
              
            //6
            consoleIO.writeMessage("Unknown option \(value)", to: .error)
          }
        }
    }
    
    func getOption(_ option: String) -> (option:OptionType, value: String) {
      return (OptionType(value: option), option)
    }
    
}
