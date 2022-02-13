//
//  XPC_AXWrap.h
//  XPC.AXWrap
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

class MyService: NSObject, XPC_AXWrapProtocol {
    func upperCaseString(_ string: String, withReply reply: @escaping (String) -> Void) {
        let response = string.uppercased()
        reply(response)
    }
}
