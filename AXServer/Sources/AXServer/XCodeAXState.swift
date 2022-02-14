import AXSwift
import Cocoa

class XCodeAXState {
    let xCodeBundleId = "com.apple.dt.Xcode"

    var uiElementSysScope = systemWideElement
    var xCodeEditorUIElement: UIElement?
    var observer: Observer!

    public func isXCodeEditorInFocus() -> Bool {
        // 1. An instance of XCode must be running
        let XCodeApp = Application.allForBundleID(xCodeBundleId)

        if XCodeApp.count == 0 {
            NSLog("XCode is not started")
            return false
        }

        // 2. XCode must be in focus
        let focusedWindow: UIElement? = try! uiElementSysScope.attribute(.focusedUIElement)
        var focusWindowIsXCode = false
        if let unwrappedFocusedWindow = focusedWindow {
            do {
                // NSLog("PID of XCode: \(try XCodeApp.first!.pid()), PID of focused window: \(try unwrappedFocusedWindow.pid())")
                focusWindowIsXCode = try comparePIDs(unwrappedFocusedWindow.pid(), XCodeApp.first!.pid())
            } catch {
                NSLog("Error: Could not read PID of app [\(unwrappedFocusedWindow)]: \(error)")
                return false
            }
        } else {
            NSLog("Error: Could not read focused window")
            return false
        }

        if !focusWindowIsXCode {
            return false
        }

        // 3. Focus must be on a Text AREA AX element of XCode
        var focusFieldIsTextArea = false
        if let unwrappedFocusedWindow = focusedWindow {
            do {
                let uiElementType = try unwrappedFocusedWindow.role()

                if uiElementType == .textArea {
                    focusFieldIsTextArea = true
                    xCodeEditorUIElement = unwrappedFocusedWindow
                }
            } catch {
                NSLog("Error: Could not read type of UIElement [\(unwrappedFocusedWindow)]: \(error)")
                return false
            }
        }

        return focusFieldIsTextArea
    }

    public func getEditorContent() -> String? {
        if !isXCodeEditorInFocus() {
            return nil
        }

        var content: String?
        if let unwrappedXCodeEditorUIElement = xCodeEditorUIElement {
            if let unwrappedContent: String = try! unwrappedXCodeEditorUIElement.attribute(.value) {
                content = unwrappedContent
            }
        }

        return content
    }

    public func updateEditorContent(_ newContent: String) -> String? {
        if !isXCodeEditorInFocus() {
            return nil
        }

        if let unwrappedXCodeEditorUIElement = xCodeEditorUIElement {
            do {
                try unwrappedXCodeEditorUIElement.setAttribute(.value, value: newContent)
            } catch {
                NSLog("Error: Could not set value of UIElement [\(unwrappedXCodeEditorUIElement)]: \(error)")
                return nil
            }
        }

        var newContent: String?
        if let unwrappedXCodeEditorUIElement = xCodeEditorUIElement {
            if let unwrappedNewContent: String = try! unwrappedXCodeEditorUIElement.attribute(.value) {
                newContent = unwrappedNewContent
            }
        }

        return newContent
    }

    func comparePIDs(_ pidWindow: Int32, _ pidOtherWindow: Int32) -> Bool {
        if pidWindow == pidOtherWindow {
            return true
        } else {
            return false
        }
    }

    // func startWatcher(_ app: Application) throws {
    //     var updated = false
    //     observer = app.createObserver { (observer: Observer, element: UIElement, event: AXNotification, info: [String: AnyObject]?) in
    //         // Watch events on new windows
    //         if event == .windowCreated {
    //             do {
                   
    //                 try observer.addNotification(.uiElementDestroyed, forElement: element)
    //                 try observer.addNotification(.moved, forElement: element)
    //             } catch let error {
    //                 NSLog("Error: Could not watch [\(element)]: \(error)")
    //             }
    //         }

    //         // Group simultaneous events together with --- lines
    //         if !updated {
    //             updated = true
    //             // Set this code to run after the current run loop, which is dispatching all notifications.
    //             DispatchQueue.main.async {
    //                 print("---")
    //                 updated = false
    //             }
    //         }
    //     }

    //     try observer.addNotification(.windowCreated, forElement: app)
    //     try observer.addNotification(.mainWindowChanged, forElement: app)
    // }
}
