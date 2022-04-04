use std::thread;

use accessibility::AXUIElement;

use super::{global::observer_global, utils::TauriState, xcode::observer_xcode};

static LOOP_TIME_IN_MS: u64 = 150;

// This is the entry point for the Observer registrations
// It is called from the main thread at program startup
pub fn setup_observers(tauri_handle: TauriState) {
    let tauri_handle_move_copy = TauriState {
        handle: tauri_handle.handle.clone(),
    };
    thread::spawn(move || {
        let mut focused_app: Option<AXUIElement> = None;
        let mut xcode_app: Option<AXUIElement> = None;

        loop {
            // Register XCode observer
            // =======================
            let _ = observer_xcode(&mut xcode_app, &tauri_handle_move_copy);

            // Monitor which app is currently in focus
            // =======================================
            let _ = observer_global(&mut focused_app, &tauri_handle_move_copy);

            thread::sleep(std::time::Duration::from_millis(LOOP_TIME_IN_MS));
        }
    });
}
