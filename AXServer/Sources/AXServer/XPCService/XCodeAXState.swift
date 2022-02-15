import AXSwift
import Cocoa

class XCodeAXState {
    let xCodeBundleId: String

    var xCodeApp: Application?
    var lastFocusedXCodeEditorUIElement: UIElement?

    var observerContent: Observer?
    var count = 0 // to be removed later

    init(_ xCodeBundleId: String) {
        print("IsEditorFocused:")

        self.xCodeBundleId = xCodeBundleId
        timerFetchXCodeApplication()
        timerCheckFocusXCodeEditor()
    }

    public func isXCodeEditorInFocus() -> Bool {
        // 1. XCode must be in focus
        if !isXCodeAppInFocus() {
            return false
        }

        guard let unwrappedFocusedWindow = try? systemWideElement.attribute(.focusedUIElement) as UIElement? else {
            NSLog("Error: Could not read focused window")
            return false
        }

        // 2. Focus must be on a Text AREA AX element of XCode
        var focusFieldIsTextArea = false
        do {
            let uiElementType = try unwrappedFocusedWindow.role()

            if uiElementType == .textArea {
                focusFieldIsTextArea = true
                lastFocusedXCodeEditorUIElement = unwrappedFocusedWindow
            }
        } catch {
            NSLog("Error: Could not read type of UIElement [\(unwrappedFocusedWindow)]: \(error)")
            return false
        }

        return focusFieldIsTextArea
    }

    public func isXCodeAppInFocus() -> Bool {
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
            focusAppIsXCode = try comparePIDs(unwrappedFocusedWindow.pid(), app.pid())
        } catch {
            NSLog("Error: Could not read PID of app [\(unwrappedFocusedWindow)]: \(error)")
            return false
        }

        return focusAppIsXCode
    }

    public func isXCodeAppRunning() -> Bool {
        // 1. An instance of XCode must be running
        let xCodeApplication = Application.allForBundleID(xCodeBundleId)

        if xCodeApplication.count == 0 {
            xCodeApp = nil
            return false
        } else {
            xCodeApp = xCodeApplication[0]
            return true
        }
    }

    public func getEditorContent() -> String? {
        // Logic: If XCode is still running and the editor UI element is known, return its value
        if !isXCodeAppRunning() {
            return nil
        }

        var content: String?
        if let unwrappedXCodeEditorUIElement = lastFocusedXCodeEditorUIElement {
            if let unwrappedContent: String = try? unwrappedXCodeEditorUIElement.attribute(.value) {
                content = unwrappedContent
            } else {
                return nil
            }
        }

        return content
    }

    public func updateEditorContent(_ newContent: String) -> String? {
        // Logic: If XCode is still running and the editor UI element is known, update its value
        if !isXCodeAppRunning() {
            return nil
        }

        guard let unwrappedXCodeEditorUIElement = lastFocusedXCodeEditorUIElement else { return nil }

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
        // Fetch the XCode application _again_ to later compare it with the previous one
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
        let a = isXCodeEditorInFocus()

        print("IsEditorFocused: \(a)")
    }
}
