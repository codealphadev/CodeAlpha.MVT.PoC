//
//  main.m
//  XPC.AXWrap
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

let delegate = XPC_AXWrapDelegate()
let listener = NSXPCListener.service()
listener.delegate = delegate
listener.resume()

let connection = NSXPCConnection(serviceName: "com.adrianhupka.XPC_AXWrap")
connection.remoteObjectInterface = NSXPCInterface(with: XPC_AXWrapProtocol.self)
connection.resume()

let service = connection.remoteObjectProxyWithErrorHandler { error in
    print("Received error:", error)
} as? XPC_AXWrapProtocol

service?.upperCaseString("hello XPC") { response in
    print("Response from XPC service:", response)
}
