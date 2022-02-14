## Things done

For the beginning I will track my progress in this README. In the past this has served me well when learning many things at the same time.

### 15.02.2022

**AX Server logic:**
- [ ] Bind observer to XCode application, if XCode is open
- [ ] Unbind observer when XCode is closed
- [ ] Add notification "AXFocusedUIElementChanged" to observer to track when XCode editor loses focus
- [ ] Add notification to observer to track when XCode editor goes (back) in focus
- [ ] Add notification "AXValueChanged" to observer to track when user modified the content

**AX Server connectivity:**
- [ ] XPC Interface for GETTERs
- [ ] Learn how to create anonymous listener from Client app
- [ ] XPC Interface for notifications

**Client App:**
- [ ] Implement "listen" mode
- [ ] Implement GET for focus status
- [ ] Implement GET for editor content
- [ ] Implement POST for content update

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