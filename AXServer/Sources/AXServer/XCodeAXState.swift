import AXSwift
import Cocoa

class XCodeAXState {
    let xCodeBundleId = "com.apple.dt.Xcode"

    var xCodeApp: Application?
    var xCodeEditorUIElement: UIElement?
    var observer: Observer?

    var observerContent: Observer?
    var count = 0

    init() {
        timerFetchXCodeApplication()
        timerCheckFocusXCodeEditor()
    }

    public func isXCodeEditorInFocus() -> Bool {
        guard let unwrappedFocusedWindow = try? systemWideElement.attribute(.focusedUIElement) as UIElement? else {
            NSLog("Error: Could not read focused window")
            return false
        }

        // 1. An instance of XCode must be running
        guard let app = xCodeApp else {
            return false
        }

        // 2. XCode must be in focus
        var focusAppIsXCode = false
        do {
            // NSLog("PID of XCode: \(try XCodeApp.first!.pid()), PID of focused window: \(try unwrappedFocusedWindow.pid())")
            focusAppIsXCode = try comparePIDs(unwrappedFocusedWindow.pid(), app.pid())
        } catch {
            NSLog("Error: Could not read PID of app [\(unwrappedFocusedWindow)]: \(error)")
            return false
        }

        if !focusAppIsXCode {
            return false
        }

        // 3. Focus must be on a Text AREA AX element of XCode
        var focusFieldIsTextArea = false
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

        return focusFieldIsTextArea
    }

    public func getEditorContent() -> String? {
        if !isXCodeEditorInFocus() {
            return nil
        }

        var content: String?
        if let unwrappedXCodeEditorUIElement = xCodeEditorUIElement {
            if let unwrappedContent: String = try? unwrappedXCodeEditorUIElement.attribute(.value) {
                content = unwrappedContent
            } else {
                return nil
            }
        }

        return content
    }

    public func updateEditorContent(_ newContent: String) -> String? {
        if !isXCodeEditorInFocus() {
            return nil
        }

        guard let unwrappedXCodeEditorUIElement = xCodeEditorUIElement else { return nil }

        do {
            try unwrappedXCodeEditorUIElement.setAttribute(.value, value: newContent)
        } catch {
            NSLog("Error: Could not set value of UIElement [\(unwrappedXCodeEditorUIElement)]: \(error)")
            return nil
        }

        var newContent: String?
        if let unwrappedNewContent: String = try! unwrappedXCodeEditorUIElement.attribute(.value) {
            newContent = unwrappedNewContent
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

    func observeEditorContentChanges() throws {
        guard let unwrappedApp = xCodeApp else { return }

        var updated = false
        observerContent = unwrappedApp.createObserver { (_: Observer, element: UIElement, event: AXNotification, _: [String: AnyObject]?) in
            // var elementDesc: String!
            // if let role = try? element.role()!, role == .window {
            //     elementDesc = "\(element) \"\(try! (element.attribute(.title) as String?)!)\""
            // } else {
            //     elementDesc = "\(element)"
            // }
            // // print("\(event) on \(String(describing: elementDesc)); info: \(info ?? [:])")

            if event == .valueChanged {
                // Focus must be on a Text AREA AX element of XCode
                do {
                    if !(try element.attributeIsSupported(.role)) {
                        return
                    }
                    let uiElementType = try element.role()
                    if uiElementType == .textArea {
                        print("CHANGE DETECTED \(self.count)")
                        self.count = self.count + 1
                        // // Group simultaneous events together with --- lines
                        if !updated {
                            updated = true
                            // Set this code to run after the current run loop, which is dispatching all notifications.
                            DispatchQueue.main.async {
                                print("---")
                                updated = false
                            }
                        }
                    }
                } catch {
                    NSLog("Error: Could not read type of UIElement [\(element)]: \(error)")
                    return
                }
            }
        }

        try observerContent!.addNotification(.valueChanged, forElement: unwrappedApp)
    }

    func timerFetchXCodeApplication() {
        _ = Timer.scheduledTimer(
            timeInterval: 0.1,
            target: self,
            selector: #selector(fetchXCodeApplication),
            userInfo: nil,
            repeats: true
        )
    }

    @objc func fetchXCodeApplication() {
        // 1. An instance of XCode must be running
        let xCodeApplication = Application.allForBundleID(xCodeBundleId)

        if xCodeApplication.count == 0 {
            NSLog("XCode is not started.")
            xCodeApp = nil
            return
        }

        // only continue when app has changed
        if xCodeApp == xCodeApplication.first! {
            return
        } else {
            xCodeApp = xCodeApplication.first!
            do {
                try observeEditorContentChanges()
            } catch {
                print("Error: Could not watch content: \(error)")
                xCodeApp = nil
            }
        }
    }

    func timerCheckFocusXCodeEditor() {
        _ = Timer.scheduledTimer(
            timeInterval: 0.1,
            target: self,
            selector: #selector(checkFocusXCodeEditor),
            userInfo: nil,
            repeats: true
        )
    }

    @objc func checkFocusXCodeEditor() {
        _ = isXCodeEditorInFocus()
    }
}
