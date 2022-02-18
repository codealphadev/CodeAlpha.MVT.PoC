
## Installation instructions

1. Clone Git repository, navigate to repository's root folder
2. Build the project: `swift build`
3. Execute `sh Assets/updateDaemon.sh` to load `*plist` file to `launchd`. This registers this service to be launched at user login. 
4. Check if the daemon runs without errors (for now...): `launchctl list | grep codeAlpha`. If there is an error, go investigate. 🤓

## Accepted Constraints

* [15.02.2022]: It is not yet gracefully handled when the user denys accessibility permissions. Right now, the app terminates but the XPC service keeps running. 

## Things done

For the beginning I will track my progress in this README. In the past this has served me well when learning many things at the same time.

### 17.02.2022

**AX Server logic:**
- [x] Improve routine to write to log file on disk
- [ ] Debug "getXCodeAppFocusStatus" method when called via XPC
- [ ] Continue testing server logic
- [ ] Change focus notification to "when editor goes in or out of focus"

**AX Server connectivity:**
- [ ] Debug XPC Interface for notifications 

**Client App:**
- [ ] Testing control flow

### 16.02.2022

**Misc**
- [x] Create first architecture sketch as part of the documentation

**AX Server logic:**
- [x] Write debug logs to file
- [x] Remove "RedString/GreenString" helpers
- [x] Refactor class structure
- [x] Implement anonymous listener interface

**AX Server connectivity:**
- [x] Learn how to create anonymous listener from Client app  

**Client App:**
- [x] Implement "listen" mode
- [x] Implement proper CLI companion app
### 15.02.2022

**AX Server logic:**
- [x] Bind observer to XCode application, if XCode is open
- [x] Unbind observer when XCode is closed
- [x] Track when XCode editor area loses focus
- [x] Track when XCode editor goes (back) in focus
- [x] Add notification "AXValueChanged" to observer to track when user modified the content

**AX Server connectivity:**
- [x] XPC Interface for GETTERs

**Client App:**
- [x] Implement GET for focus status
- [x] Implement GET for editor content
- [x] Implement POST for content update

### **14.02.2022**

**Learnings:**
- [x] Learn how to develop without XCode
  - [x] Understand `*.plist` files
  - [x] Understand `Package.swift`
  - [x] Understand how to combine XPC-Service and NSApplication within one app
- [x] Figure out how to catch `SIGINT` in running application
- [x] Learned that NSAccessibility features require a running NSApplication 🤓

**AX Server logic implementation:**
- [x] Implementation of GET logic for XCode content
- [x] Implementation of GET logic for XCode focus status
- [x] Implementation of POST logic for XCode editor content