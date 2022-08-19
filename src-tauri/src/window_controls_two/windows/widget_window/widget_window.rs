use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;
use window_shadows::set_shadow;

use crate::{
    app_handle,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls_two::{
        actions::create_default_window_builder,
        config::{default_properties, AppWindow},
    },
};

use super::listeners::window_control_events_listener;

static WIDGET_OFFSET: f64 = 75.;

#[derive(Clone, Debug)]
pub struct WidgetWindow {
    app_handle: tauri::AppHandle,

    // The widget window's size
    size: LogicalSize,
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

        Ok(Self {
            app_handle,
            size: LogicalSize {
                width: default_properties::size(&AppWindow::Widget).0,
                height: default_properties::size(&AppWindow::Widget).1,
            },
        })
    }

    pub fn start_event_listeners(widget_window: &Arc<Mutex<WidgetWindow>>) {
        window_control_events_listener(widget_window);
    }

    pub fn show(
        &self,
        widget_position: &Option<LogicalPosition>,
        editor_textarea: &LogicalFrame,
        monitor: &LogicalFrame,
    ) -> Option<()> {
        let tauri_window = self.app_handle.get_window(&AppWindow::Widget.to_string())?;

        // In case the widget has never been moved by the user, we set an initial position
        // based on the editor textarea.
        let mut corrected_position = if let Some(position) = widget_position.to_owned() {
            position
        } else {
            self.initial_widget_position(editor_textarea)
        };

        // Determine if the widget would be off-screen and needs to be moved.
        let (offscreen_dist_x, offscreen_dist_y) =
            self.calc_off_screen_distance(&corrected_position, &monitor);

        if let Some(offscreen_dist_x) = offscreen_dist_x {
            corrected_position.x += offscreen_dist_x;
        }

        if let Some(offscreen_dist_y) = offscreen_dist_y {
            corrected_position.y += offscreen_dist_y;
        }

        tauri_window
            .set_position(corrected_position.as_tauri_LogicalPosition())
            .ok()?;
        tauri_window.show().ok()?;

        Some(())
    }

    pub fn hide(&self) -> Option<()> {
        _ = self
            .app_handle
            .get_window(&AppWindow::Widget.to_string())?
            .hide();

        Some(())
    }

    pub fn calc_off_screen_distance(
        &self,
        widget_position: &LogicalPosition,
        monitor: &LogicalFrame,
    ) -> (Option<f64>, Option<f64>) {
        let mut dist_x: Option<f64> = None;
        let mut dist_y: Option<f64> = None;

        // prevent widget from going off-screen
        if widget_position.x < monitor.origin.x {
            dist_x = Some(monitor.origin.x - widget_position.x);
        }
        if widget_position.y < monitor.origin.y {
            dist_y = Some(monitor.origin.y - widget_position.y);
        }
        if widget_position.x + self.size.width > monitor.origin.x + monitor.size.width {
            dist_x =
                Some(monitor.origin.x + monitor.size.width - self.size.width - widget_position.x);
        }
        if widget_position.y + self.size.height > monitor.origin.y + monitor.size.height {
            dist_y =
                Some(monitor.origin.y + monitor.size.height - self.size.height - widget_position.y);
        }

        (dist_x, dist_y)
    }

    fn initial_widget_position(&self, editor_textarea: &LogicalFrame) -> LogicalPosition {
        // In case no widget position is set yet, initialize widget position on editor textarea
        LogicalPosition {
            x: editor_textarea.origin.x + editor_textarea.size.width - WIDGET_OFFSET,
            y: editor_textarea.origin.y + editor_textarea.size.height - WIDGET_OFFSET,
        }
    }
}
