use std::sync::Arc;

use cocoa::{base::id, foundation::NSInteger};

use objc::{msg_send, sel, sel_impl};

use parking_lot::Mutex;
use tauri::Manager;

use crate::{
    app_handle,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::{default_properties, AppWindow, WindowLevel},
        utils::create_default_window_builder,
        windows::utils::{app_window_dimensions, register_tracking_area, update_tracking_area},
        EventTrackingArea, TrackingArea,
    },
};

use super::listeners::window_control_events_listener;

static WIDGET_OFFSET: f64 = 75.;
pub static WIDGET_MAIN_WINDOW_OFFSET: f64 = 24.;

#[derive(Clone, Debug)]
pub struct WidgetWindow {
    app_handle: tauri::AppHandle,

    // Is the main window shown
    main_window_shown: Option<bool>,

    // The widget window's size
    size: LogicalSize,

    // The window's tracking area
    tracking_area: TrackingArea,
}

impl WidgetWindow {
    pub fn new() -> Result<Self, tauri::Error> {
        let app_handle = app_handle();

        // Create Widget Window at startup.
        // If the window is already created, don't open it again.
        if app_handle
            .get_window(&AppWindow::Widget.to_string())
            .is_none()
        {
            create_default_window_builder(&app_handle, AppWindow::Widget)?.build()?;
        }

        Ok(Self {
            app_handle,
            size: LogicalSize {
                width: default_properties::size(&AppWindow::Widget).0,
                height: default_properties::size(&AppWindow::Widget).1,
            },
            main_window_shown: None,
            tracking_area: Self::register_tracking_area(),
        })
    }

    pub fn set_macos_properties(&self) -> Option<()> {
        let ns_window_ptr_widget = self
            .app_handle
            .get_window(&AppWindow::Widget.to_string())?
            .ns_window();

        if let Ok(ns_window_ptr_widget) = ns_window_ptr_widget {
            unsafe {
                // Prevent the widget from causing our application to take focus.
                let _: () = msg_send![ns_window_ptr_widget as id, _setPreventsActivation: true];

                // Set the widget's window level
                let _: () = msg_send![
                    ns_window_ptr_widget as id,
                    setLevel: WindowLevel::Widget as NSInteger
                ];
            }
        }

        Some(())
    }

    pub fn start_event_listeners(widget_window: &Arc<Mutex<WidgetWindow>>) {
        window_control_events_listener(widget_window);
    }

    pub fn set_main_window_shown(&mut self, main_window_shown: Option<bool>) {
        self.main_window_shown = main_window_shown;
    }

    pub fn show(
        &self,
        updated_widget_position: &Option<LogicalPosition>,
        editor_textarea: &LogicalFrame,
        editor_monitor: &LogicalFrame,
    ) -> Option<()> {
        let widget_tauri_window = self.app_handle.get_window(&AppWindow::Widget.to_string())?;

        // In case the widget has never been moved by the user, we set an initial position
        // based on the editor textarea.
        let mut widget_position = if let Some(position) = updated_widget_position.to_owned() {
            position
        } else {
            self.initial_widget_position(editor_textarea)
        };

        let relevant_monitor =
            Self::determine_widget_monitor(&self.size, &widget_position, &editor_monitor)?;

        let (offscreen_dist_x, offscreen_dist_y) =
            Self::calc_off_screen_distance(&self.size, &widget_position, &relevant_monitor);

        if let Some(offscreen_dist_x) = offscreen_dist_x {
            widget_position.x += offscreen_dist_x;
        }

        if let Some(offscreen_dist_y) = offscreen_dist_y {
            widget_position.y += offscreen_dist_y;
        }

        widget_tauri_window
            .set_position(widget_position.as_tauri_LogicalPosition())
            .ok()?;
        widget_tauri_window.show().ok()?;

        self.update_tracking_area(true);

        Some(())
    }

    pub fn hide(&self) -> Option<()> {
        _ = self
            .app_handle
            .get_window(&AppWindow::Widget.to_string())?
            .hide();

        self.update_tracking_area(false);

        Some(())
    }

    fn update_tracking_area(&self, is_visible: bool) {
        update_tracking_area(AppWindow::Widget, self.tracking_area.clone(), is_visible)
    }

    fn register_tracking_area() -> TrackingArea {
        register_tracking_area(AppWindow::Widget)
    }

    fn determine_widget_monitor(
        widget_size: &LogicalSize,
        widget_position: &LogicalPosition,
        editor_monitor: &LogicalFrame,
    ) -> Option<LogicalFrame> {
        // We compute the position relative to the monitor the widget is on. If the widget is "offscreen" to
        // its own monitor, we move it to the monitor the editor window is on.

        // We fetch the window where the widget is on
        let widget_window = app_handle().get_window(&AppWindow::Widget.to_string())?;
        let monitor = widget_window.current_monitor().ok()??;

        let scale_factor = monitor.scale_factor();
        let widget_monitor_origin = LogicalPosition::from_tauri_LogicalPosition(
            &monitor.position().to_logical::<f64>(scale_factor),
        );
        let widget_monitor_size =
            LogicalSize::from_tauri_LogicalSize(&monitor.size().to_logical::<f64>(scale_factor));

        let widget_monitor = LogicalFrame {
            origin: widget_monitor_origin,
            size: widget_monitor_size,
        };

        let (offscreen_dist_x, offscreen_dist_y) =
            Self::calc_off_screen_distance(&widget_size, &widget_position, &widget_monitor);

        if offscreen_dist_x.is_some() || offscreen_dist_y.is_some() {
            Some(editor_monitor.to_owned())
        } else {
            Some(widget_monitor)
        }
    }

    fn calc_off_screen_distance(
        widget_size: &LogicalSize,
        widget_position: &LogicalPosition,
        monitor: &LogicalFrame,
    ) -> (Option<f64>, Option<f64>) {
        let mut dist_x: Option<f64> = None;
        let mut dist_y: Option<f64> = None;

        let corrected_size = widget_size.to_owned();
        let corrected_position = widget_position.to_owned();

        // prevent widget from going off-screen
        if corrected_position.x < monitor.origin.x {
            dist_x = Some(monitor.origin.x - corrected_position.x);
        }
        if corrected_position.y < monitor.origin.y {
            dist_y = Some(monitor.origin.y - corrected_position.y);
        }
        if corrected_position.x + corrected_size.width > monitor.origin.x + monitor.size.width {
            dist_x = Some(
                monitor.origin.x + monitor.size.width - corrected_size.width - corrected_position.x,
            );
        }
        if corrected_position.y + corrected_size.height > monitor.origin.y + monitor.size.height {
            dist_y = Some(
                monitor.origin.y + monitor.size.height
                    - corrected_size.height
                    - corrected_position.y,
            );
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

    pub fn dimensions() -> LogicalFrame {
        app_window_dimensions(AppWindow::Widget)
    }
}

#[cfg(test)]
mod tests_widget_window {

    use crate::utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize};

    use super::WidgetWindow;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_calc_offscreen_distance() {
        let widget_size = LogicalSize {
            width: 48.,
            height: 48.,
        };
        let monitor = LogicalFrame {
            origin: LogicalPosition { x: 0., y: 0. },
            size: LogicalSize {
                width: 100.,
                height: 100.,
            },
        };

        let widget_position = LogicalPosition { x: 0., y: 0. };
        let (dist_x, dist_y) =
            WidgetWindow::calc_off_screen_distance(&widget_size, &widget_position, &monitor);

        assert_eq!(dist_x, None);
        assert_eq!(dist_y, None);

        let widget_position = LogicalPosition { x: 100., y: 100. };
        let (dist_x, dist_y) =
            WidgetWindow::calc_off_screen_distance(&widget_size, &widget_position, &monitor);

        assert_eq!(dist_x, Some(-48.));
        assert_eq!(dist_y, Some(-48.));

        let widget_position = LogicalPosition {
            x: 100. - widget_size.width,
            y: 100. - widget_size.height,
        };
        let (dist_x, dist_y) =
            WidgetWindow::calc_off_screen_distance(&widget_size, &widget_position, &monitor);

        assert_eq!(dist_x, None);
        assert_eq!(dist_y, None);
    }
}

impl Drop for WidgetWindow {
    fn drop(&mut self) {
        EventTrackingArea::Remove(vec![self.tracking_area.id]).publish_to_tauri(&app_handle());
    }
}
