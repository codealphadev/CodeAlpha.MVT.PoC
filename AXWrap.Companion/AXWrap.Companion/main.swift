//
//  main.swift
//  AXWrap.Companion
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

let axWrapCompanion = AXWrapCompanion()
if CommandLine.argc < 2 {
    axWrapCompanion.interactiveMode()
} else {
    let consoleIO = ConsoleIO()
    consoleIO.writeMessage("Static Mode not yet implemented.")
}

