//
//  XPC_AXWrapProtocol.h
//  XPC.AXWrap
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation

@objc public protocol XPC_AXWrapProtocol {
    func upperCaseString(_ string: String, withReply reply: @escaping (String) -> Void)
}
