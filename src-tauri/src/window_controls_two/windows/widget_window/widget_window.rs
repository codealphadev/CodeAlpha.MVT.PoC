use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;
use window_shadows::set_shadow;

use crate::{
    app_handle,
    window_controls_two::{actions::create_default_window_builder, config::AppWindow},
};

use super::listeners::window_control_events_listener;

static WIDGET_OFFSET: f64 = 75.;

#[derive(Clone, Debug)]
pub struct WidgetWindow {
    pub app_handle: tauri::AppHandle,
}

impl WidgetWindow {
    pub fn new() -> Result<Self, tauri::Error> {
        let app_handle = app_handle();

        // Create CodeOverlay Window at startup.
        // If the window is already created, don't open it again.
        if app_handle
            .get_window(&AppWindow::Widget.to_string())
            .is_none()
        {
            let window_builder = create_default_window_builder(&app_handle, AppWindow::Widget)?;
            let window = window_builder.build()?;

            set_shadow(&window, false).expect("Unsupported platform!");
        }

        Ok(Self { app_handle })
    }

    pub fn start_event_listeners(widget_window: &Arc<Mutex<WidgetWindow>>) {
        window_control_events_listener(widget_window);
    }
}
