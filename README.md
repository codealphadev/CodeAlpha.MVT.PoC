# CodeAlpha.MVT.PoC

In preparation to build a _Minimum Viable Test (MVT)_ later this year, this project is a technical Proof of Concept (PoC).

## Technical Details

- Build with **Tauri**, an alternative to Electron. Tauri allows a much cleaner separation of _frontend_ and _backend_ in a desktop application, where the frontend is written in TypeScript and the backend is written in Rust.
- Frontend: TypeScript with Svelte and TailwindCSS
- Backend: Rust

## Known Issues

- "Many Clicks" on widget can lead to widget disappearing -> more gracefully handle invokation of "Content Open" routine - can be jammed if many clicks are done on widget
- Moving the editor window does not hide the widget while re-positioning
- Repositioning logic on re-size or move of editor window is far from perfect
- Show / hide content window requires waaay too many clicks ("Ghostclicks")
- In "observer_global.rs" exists a hack: I changed around the order of which "callback" is executed first, because this sufficiently improved UX behavior for now.
- Restarting XCode leads to strange behavior
- Closing XCode does not close the widget

## Design Debt

- Everything accessibility related shall not spill out of the `ax_interactions` module in order to more easily implement Windows Accessibility APIs. This includes not exposing any specific structs, enums and errors.
  - Refactoring approach: start by adjusting the `mod.rs` file an `ax_interactions`. The moment everything is made private, the compiler will throw all errors. :-)
