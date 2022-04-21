use tauri::{LogicalPosition, LogicalSize};

use crate::window_controls::{
    actions::{current_monitor_of_window, get_size},
    default_properties, AppWindow,
};

// Hard coded values for the positioning of the content window to a widget of size 48px
pub static POSITIONING_OFFSET_X: f64 = 24.;
pub static POSITIONING_OFFSET_Y: f64 = 8.;

pub fn prevent_widget_position_off_screen(
    app_handle: &tauri::AppHandle,
    widget_position: &mut LogicalPosition<f64>,
) {
    let widget_size = LogicalSize {
        width: default_properties::size(&AppWindow::Widget).0,
        height: default_properties::size(&AppWindow::Widget).1,
    };

    if let Some(move_distance) = calc_off_screen_distance(app_handle, widget_position, &widget_size)
    {
        widget_position.x += move_distance.width;
        widget_position.y += move_distance.height;
    }
}

/// Updates the provided position to prevent the window from being off screen.
/// Returns an optional LogicalSize to tell how much the window was moved.
pub fn calc_off_screen_distance(
    app_handle: &tauri::AppHandle,
    window_position: &LogicalPosition<f64>,
    window_size: &LogicalSize<f64>,
) -> Option<LogicalSize<f64>> {
    if let Some(monitor) = current_monitor_of_window(&app_handle, AppWindow::Widget) {
        // 0. Get Screen dimensions
        // TODO: figure out the correct screen
        let screen_position = (*monitor.position()).to_logical::<f64>(monitor.scale_factor());
        let screen_size = (*monitor.size()).to_logical::<f64>(monitor.scale_factor());

        let mut move_distance = LogicalSize {
            width: 0.0,
            height: 0.0,
        };

        // prevent widget from going off-screen
        if window_position.x < screen_position.x {
            move_distance.width = screen_position.x - window_position.x;
        }
        if window_position.y < screen_position.y {
            move_distance.height = screen_position.y - window_position.y;
        }
        if window_position.x + window_size.width > screen_position.x + screen_size.width {
            move_distance.width =
                screen_position.x + screen_size.width - window_size.width - window_position.x;
        }
        if window_position.y + window_size.height > screen_position.y + screen_size.height {
            move_distance.height =
                screen_position.y + screen_size.height - window_size.height - window_position.y;
        }

        if move_distance.width == 0.0 && move_distance.height == 0.0 {
            return None;
        } else {
            Some(move_distance)
        }
    } else {
        None
    }
}

pub fn prevent_misalignement_of_content_and_widget(
    app_handle: &tauri::AppHandle,
    widget_position: &mut LogicalPosition<f64>,
) {
    if let Ok(content_size) = get_size(&app_handle, AppWindow::Content) {
        // TODO: select the correct screen
        if let Some(monitor) = current_monitor_of_window(&app_handle, AppWindow::Widget) {
            let monitor_position = monitor.position().to_logical::<f64>(monitor.scale_factor());

            let widget_size = LogicalSize {
                width: default_properties::size(&AppWindow::Widget).0,
                height: default_properties::size(&AppWindow::Widget).1,
            };

            // only reposition, if widget is too close to upper end of screen
            if (monitor_position.y) < (widget_position.y - content_size.height) {
                return;
            }

            // Update widget position to respect content window dimensions
            // widget_position.x =
            //     content_position.x + content_size.width - widget_size.width - POSITIONING_OFFSET_Y;
            widget_position.y = monitor_position.y
                + content_size.height
                + (widget_size.height / 2.)
                + POSITIONING_OFFSET_X;
        }
    }
}
