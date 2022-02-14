//
//  XPC_AXWrap.h
//  XPC.AXWrap
//
//  Created by Adrian Hupka on 13.02.22.
//

import Foundation
import AXSwift

class XPC_AXWrap: NSObject, XPC_AXWrapProtocol {
    func upperCaseString(_ string: String, withReply reply: @escaping (String) -> Void) {
        let response = string.uppercased()
        reply(response)
    }
    
    func checkAccessibilityPermission() -> Bool {
        guard UIElement.isProcessTrusted(withPrompt: true) else {
            NSLog("No accessibility API permission, exiting")
            return false
        }
        
        return true
    }
}
