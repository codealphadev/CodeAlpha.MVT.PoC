//
//  AXWrap-Companion.swift
//  AXWrap.Companion
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

enum OptionType: String {
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
    case "q": self = .quit
    default: self = .unknown
    }
  }
}

class AXServerCompanion {
  let consoleIO = ConsoleIO()

  let connection: NSXPCConnection
  var service: AXServerXPCProtocol?
  var anonymousListener: NSXPCListener
  var axServerXpcReachable: Bool = true

  init() {
    // Prepare bi-directional connection
    // Because this connection is generated at runtime, we need to use the anonymous listener.
    anonymousListener = NSXPCListener.anonymous()

    // Register a class object that implements the XPC protocol for the listening events.
    let listenerServiceDelegate = AXClientServiceDelegate()
    anonymousListener.delegate = listenerServiceDelegate
    anonymousListener.resume()

    // =====
    connection = NSXPCConnection(machServiceName: "com.codeAlpha.AXServerXPC")
    connection.remoteObjectInterface = NSXPCInterface(with: AXServerXPCProtocol.self)
    connection.resume()

    axServerXpcReachable = true
    if #available(macOS 10.11, *) {
      service = connection.synchronousRemoteObjectProxyWithErrorHandler { error in
        self.consoleIO.writeMessage("Received error: \(error.localizedDescription). Terminating.", to: .error)
        self.axServerXpcReachable = false
      } as? AXServerXPCProtocol
    } else {
      service = connection.remoteObjectProxyWithErrorHandler { error in
        self.consoleIO.writeMessage("Received error: \(error.localizedDescription). Terminating.", to: .error)
        self.axServerXpcReachable = false
      } as? AXServerXPCProtocol
    }

    guard let unwrappedService = service else {
      consoleIO.writeMessage("Service not available", to: .error)
      return
    }
    let endpoint = anonymousListener.endpoint
    unwrappedService.startAnonymousListener(endpoint) { accepted in
      if accepted {
        self.consoleIO.writeMessage("The listener is now registered.")
      } else {
        self.consoleIO.writeMessage("The listener could not be registered.", to: .error)
        return
      }
    }
  }

  func interactiveMode() {
    consoleIO.writeMessage("This app is used to interact with the AXWrap service via XPC protocol.")
    // 2
    var shouldQuit = false
    while !shouldQuit {
      // 3
      consoleIO.writeMessage("Type 'l' to listen for events.")
      consoleIO.writeMessage("Type 'gf' to get information about the currently focused window.")
      consoleIO.writeMessage("Type 'gc' to fetch the content of the focused editor.")
      consoleIO.writeMessage("Type 's' to update the editor's content.")
      consoleIO.writeMessage("Or type 'q' to quit.")

      let (option, value) = getInputOption(consoleIO.getInput())

      consoleIO.writeMessage("You chose option '\(option.rawValue)'.")

      switch option {
      case .getContent:
        getContent()
      case .getFocus:
        // call getFocus() 10 times and wait in-between for 0.5 seconds
        for _ in 0 ..< 10 {
          getFocus()
          sleep(1)
        }
      case .send:
        updateContent()
      case .quit:
        stopAnonymousListener()
        shouldQuit = true
      default:
        consoleIO.writeMessage("Unknown option \(value)", to: .error)
      }

      // axServerXPCReachable is set to "false" in error catch block of connection.synchronousRemoteObjectProxyWithErrorHandler
      if !axServerXpcReachable {
        stopAnonymousListener()
        shouldQuit = true
      }
    }
  }

  func getInputOption(_ option: String) -> (option: OptionType, value: String) {
    return (OptionType(value: option), option)
  }

  func getFocus() {
    // guard let unwrappedService = service else {
    //   consoleIO.writeMessage("Service not available", to: .error)
    //   return
    // }

    // unwrappedService.getXCodeAppFocusStatus { focused in
    //   if focused {
    //     self.consoleIO.writeMessage("XCode is focused.")
    //   } else {
    //     self.consoleIO.writeMessage("XCode is not focused.")
    //   }
    // }
  }

  func getContent() {
    // guard let unwrappedService = service else {
    //   consoleIO.writeMessage("Service not available", to: .error)
    //   return
    // }

    // unwrappedService.getXCodeEditorContent { content in
    //   if let unwrappedContent = content {
    //     self.consoleIO.writeMessage("The content of the focused editor is: \n\n '\(unwrappedContent)'")
    //   } else {
    //     self.consoleIO.writeMessage("The content of the focused editor could not be fetched.")
    //   }
    // }
  }

  func updateContent() {
    // guard let unwrappedService = service else {
    //   consoleIO.writeMessage("Service not available", to: .error)
    //   return
    // }

    // consoleIO.writeMessage("Type the text to be sent to the editor:")
    // let updateText = consoleIO.getInput()

    // unwrappedService.updateXCodeEditorContent(updateText) { content in
    //   if let unwrappedContent = content {
    //     self.consoleIO.writeMessage("The content of the focused editor is: \n\n '\(unwrappedContent)' \n\n")
    //   } else {
    //     self.consoleIO.writeMessage("The content of the focused editor could not be fetched.")
    //   }
    // }
  }

  func stopAnonymousListener() {
    guard let unwrappedService = service else {
      consoleIO.writeMessage("Service not available", to: .error)
      return
    }

    unwrappedService.stopAnonymousListener { accepted in
      if accepted {
        self.consoleIO.writeMessage("The listener is now unregistered.")
      } else {
        self.consoleIO.writeMessage("The listener could not be unregistered.", to: .error)
        return
      }
    }
  }
}
