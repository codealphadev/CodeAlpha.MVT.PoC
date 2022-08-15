use std::sync::Arc;

use parking_lot::Mutex;

use crate::app_handle;

use super::listeners::window_control_events_listener;

static WIDGET_OFFSET: f64 = 75.;

#[derive(Clone, Debug)]
pub struct WidgetWindow {
    pub app_handle: tauri::AppHandle,
}

impl WidgetWindow {
    pub fn new() -> Self {
        Self {
            app_handle: app_handle(),
        }
    }

    pub fn start_event_listeners(widget_window: &Arc<Mutex<WidgetWindow>>) {
        window_control_events_listener(widget_window);
    }
}
