//
//  main.swift
//  AXWrap.Companion
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

// ================================================================

func signalHandler(signal _: Int32) {
	axServerCompanion.stopAnonymousListener()
	exit(0)
}

let signals = [SIGINT, SIGTERM, SIGKILL]
for sig in signals {
	signal(sig, signalHandler)
}

// ================================================================

let axServerCompanion = AXServerCompanion()

if CommandLine.argc < 2 {
	axServerCompanion.interactiveMode()
} else {
	let consoleIO = ConsoleIO()
	consoleIO.writeMessage("Static Mode not yet implemented.")
}
