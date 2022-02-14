//
//  XPC_AXWrap.m
//  XPC.AXWrap
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

class XPC_AXWrapDelegate: NSObject, NSXPCListenerDelegate {
    func listener(_ listener: NSXPCListener, shouldAcceptNewConnection newConnection: NSXPCConnection) -> Bool {
        let exportedObject = XPC_AXWrap()
        newConnection.exportedInterface = NSXPCInterface(with: XPC_AXWrapProtocol.self)
        newConnection.exportedObject = exportedObject
        newConnection.resume()
        
        return true
    }
}
