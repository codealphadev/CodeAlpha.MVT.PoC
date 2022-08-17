use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;
use window_shadows::set_shadow;

use crate::{
    app_handle,
    window_controls_two::{actions::create_default_window_builder, config::AppWindow},
    DEV_MODE,
};

#[derive(Clone, Debug)]
pub struct CodeOverlay {
    app_handle: tauri::AppHandle,
}

impl CodeOverlay {
    pub fn new() -> Result<Self, tauri::Error> {
        let app_handle = app_handle();

        // Create CodeOverlay Window at startup.
        // If the window is already created, don't open it again.
        if app_handle
            .get_window(&AppWindow::CodeOverlay.to_string())
            .is_none()
        {
            let window_builder =
                create_default_window_builder(&app_handle, AppWindow::CodeOverlay)?;
            let window = window_builder.build()?;

            set_shadow(&window, false).expect("Unsupported platform!");

            if DEV_MODE {
                window.open_devtools();
            }
        }

        Ok(Self { app_handle })
    }

    pub fn start_event_listeners(code_overlay_window: &Arc<Mutex<CodeOverlay>>) {}
}
