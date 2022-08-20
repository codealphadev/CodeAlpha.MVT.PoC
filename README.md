# CodeAlpha.MVT.PoC

In preparation to build a _Minimum Viable Test (MVT)_ later this year, this project is a technical Proof of Concept (PoC).

## Technical Details

### Building and running the application

- Install Rust using `rustup` and NOT `homebrew`!

  ```
  # from: https://www.rust-lang.org/tools/install

  ❯ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ❯ source "$HOME/.cargo/env"
  ❯ rustc --version
    rustc 1.63.0 (4b91a6ea7 2022-08-08)
  ```

- Before building for the first time, run
  ```
  softwareupdate --install-rosetta
  ```
  to install Rosetta (if on Darwin architecture)

1. `npm install` installs all JS dependencies
2. `npm run tauri dev` builds both frontend and backend, and starts the application. It also opens the native Developer Tools window for debugging.

### Architecture

The application is built on [Tauri](https://tauri.app/), a framework for multi-platform development.

- Tauri allows a clean separation of _frontend_ and _backend_ in a desktop application; the frontend is written in TypeScript and the backend is written in Rust.
- Tauri has a much smaller memory footprint than Electron, the largest competitor in this space.

#### Frontend

The frontend is built in TypeScript, with [Svelte](https://svelte.dev/) and [TailwindCSS](https://tailwindcss.com).

#### Backend

`src-tauri/src` contains the Rust backend, organized into the following modules:

- `ax_interaction`: The interface of the application with the native accessibility UIs.
- `core_engine`: Core business logic, completely agnostic about enviroment and operating system.
- `window_controls`: Funtionality for rendering, including the `code_overlay`.

Communication between these modules is event-driven, as is communication between the backend and frontend, by serializing and de-serializing structs to JSON.

The shared interfaces between the frontend and backend are generated from Rust structs with the `TS` annotation, and exported to TypeScript interfaces in `src-tauri/bindings/`, where they can be imported by the frontend. This is provided by [ts-rs](https://github.com/Aleph-Alpha/ts-rs)
