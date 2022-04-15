# CodeAlpha.MVT.PoC

In preparation to build a _Minimum Viable Test (MVT)_ later this year, this project is a technical Proof of Concept (PoC).

## Technical Details

- Build with **Tauri**, an alternative to Electron. Tauri allows a much cleaner separation of _frontend_ and _backend_ in a desktop application, where the frontend is written in TypeScript and the backend is written in Rust.
- Frontend: TypeScript with Svelte and TailwindCSS
- Backend: Rust

## UX Rule Log

**Rules to be implemented for MVT:**

- [x] The widget should only appear when the user's cursor is in an _editor textarea_.
- [x] The widget should appear in **the bottom right corner** of the focused _editor textarea_.
- [ ] If the _editor textarea_ is off-screen the moment it is being focused, the widget should **stay hidden**.
- [x] While receiving `AXMoved` notifications for the _editor window_ ...
  - [x] the widget should be hidden until `200ms` have elapsed after the last received `AXMoved` notification
  - [x] calculate the distance it moved from the last received notification and update the widget's position accordingly.
  - [x] If the widget would move off-screen, move it only so far that it still stays on-screen **AND** on a remaining piece of the _editor textarea_
- [x] At all times, the widget should be _tied_ to one of the _editor textarea's_ horizontal and vertical boundaries `left|right` `bottom|top`.
  - [x] Ties to _editor text_area boundaries_ are being determined by the minimum distance.
  - [x] Ties to _editor text_area boundaries_ are defined as the distance in pixels to a boundary.
  - [x] Ties to _editor text_area boundaries_ are only recalculated when the widget is being **moved by the user**.
- [x] While receiving `AXResized` notifications for the _editor window_ ...
  - [x] the widget should be hidden until `200ms` have elapsed after the last received `AXResized` notification
  - [x] Using the updated _position_ and _size_ of the _editor textarea_ and the boundaries, recalculate & update the widget's position.
- [x] Hide the widget if `AXApplicationDeactivated` notification is received
- [x] Evaluate if widget should be shown when `AXApplicationActivated` is received
  - [x] Lookup currently focused UI element, if it has role `AXTextArea`, then show the widget

**XCode Behavior**

- If `AXResized` is triggered, it is only the editor text area which gets resized.
- If the editor text area is resized by changing ui elements within the editor window, the event `AXValueChanged` is triggered for a ui element with role `AXScrollBar`.
- If XCode obtains focus, the `AXApplicationActivated` notification is triggered.
- If XCode loses focus, the `AXApplicationDeactivated` notification is triggered.

## Known Issues

- [ ] "Many Clicks" on widget can lead to widget disappearing -> more gracefully handle invokation of "Content Open" routine - can be jammed if many clicks are done on widget
- [ ] Show / hide content window requires waaay too many clicks ("Ghostclicks")
- [ ] When moving the widget with opened content too far up, the widget can get stuck behind the content window --> likely going to be fixed when listener for widget-movement is implemented.
- [ ] Show/Hide decision tree for widget does not yet include the behavior of the content window
- [ ] Decision Tree is badly implemented - not logical; needs refactoring

## Bugs

- [ ] Prevent-off-screen-widget
  - [ ] Only respecting the current screen - no multiscreen support
  - [ ] Not including the content window into calculations
- [ ] Widget can be selected in app-carousel
