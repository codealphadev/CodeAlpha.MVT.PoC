use tauri::{LogicalPosition, LogicalSize, Manager};

use super::window_controls;

static POSITIONING_OFFSET_X: f64 = 24.;
static POSITIONING_OFFSET_Y: f64 = 8.;

#[derive(Clone, serde::Serialize)]
struct PayloadBubbleOrientationEvent {
    orientation_right: bool,
}

#[tauri::command]
// Checking if widget's position after being dragged is still valid
pub fn cmd_update_widget_position<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    // Get both app windows which need to be positioned in relation to one another
    let content_window = handle.get_window(&window_controls::AppWindow::Content.to_string());
    let widget_window = handle.get_window(&window_controls::AppWindow::Widget.to_string());

    // Return if either window is not found
    if widget_window.is_none() {
        return;
    }

    // Reposition Content Window and Widget Window
    if let (Some(content), Some(widget)) = (content_window, widget_window) {
        let screen = widget.current_monitor().unwrap().unwrap();
        let screen_scale_factor = screen.scale_factor();
        let pos_screen = screen.position().to_logical::<f64>(screen_scale_factor);

        let pos_widget = (widget.outer_position())
            .unwrap()
            .to_logical::<f64>(screen_scale_factor);

        let pos_content = (content.outer_position())
            .unwrap()
            .to_logical::<f64>(screen_scale_factor);

        let content_size = LogicalSize::<f64> {
            width: content
                .outer_size()
                .unwrap()
                .to_logical::<f64>(screen_scale_factor)
                .width as f64,
            height: content
                .outer_size()
                .unwrap()
                .to_logical::<f64>(screen_scale_factor)
                .height as f64,
        };

        let widget_size = LogicalSize::<f64> {
            width: widget
                .outer_size()
                .unwrap()
                .to_logical::<f64>(screen_scale_factor)
                .width,
            height: widget
                .outer_size()
                .unwrap()
                .to_logical::<f64>(screen_scale_factor)
                .height as f64,
        };

        // only reposition, if widget is too close to upper end of screen
        if pos_screen.y < (pos_widget.y - content_size.height) {
            return;
        }

        let new_pos_widget = LogicalPosition {
            x: pos_content.x + content_size.width - widget_size.width - POSITIONING_OFFSET_Y,
            y: pos_screen.y
                + content_size.height
                + (widget_size.height / 2.)
                + POSITIONING_OFFSET_X,
        };

        let _ = widget.set_position(tauri::Position::Logical(new_pos_widget));
    }
}

#[tauri::command]
pub fn cmd_update_content_position<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    // Get both app windows which need to be positioned in relation to one another
    let content_window = handle.get_window(&window_controls::AppWindow::Content.to_string());
    let widget_window = handle.get_window(&window_controls::AppWindow::Widget.to_string());

    // Return if either window is not found
    if content_window.is_none() || widget_window.is_none() {
        return;
    }

    // Reposition Content Window and Widget Window
    if let (Some(content), Some(widget)) = (content_window, widget_window) {
        let screen = widget.current_monitor().unwrap().unwrap();
        let screen_scale_factor = screen.scale_factor();
        let pos_screen = screen.position().to_logical::<f64>(screen_scale_factor);

        let pos_widget = (widget.outer_position())
            .unwrap()
            .to_logical::<f64>(screen_scale_factor);

        let widget_size = LogicalSize::<f64> {
            width: widget
                .outer_size()
                .unwrap()
                .to_logical::<f64>(screen_scale_factor)
                .width,
            height: widget
                .outer_size()
                .unwrap()
                .to_logical::<f64>(screen_scale_factor)
                .height as f64,
        };
        let content_size = LogicalSize::<f64> {
            width: content
                .outer_size()
                .unwrap()
                .to_logical::<f64>(screen_scale_factor)
                .width as f64,
            height: content
                .outer_size()
                .unwrap()
                .to_logical::<f64>(screen_scale_factor)
                .height as f64,
        };

        let mut new_content_pos = LogicalPosition {
            x: pos_widget.x + (widget_size.width - content_size.width) + POSITIONING_OFFSET_X,
            y: pos_widget.y - content_size.height - POSITIONING_OFFSET_Y,
        };

        // Check if the content would be outside the left end of the screen, if so, flip the content window position horizontally
        let mut bubble_orientation_right = true;
        if pos_screen.x > new_content_pos.x {
            new_content_pos.x = pos_widget.x - POSITIONING_OFFSET_X;
            bubble_orientation_right = false;
        }

        // Emit event to content window to update its orientation
        handle
            .emit_to(
                &window_controls::AppWindow::Content.to_string(),
                "evt-bubble-icon-orientation",
                PayloadBubbleOrientationEvent {
                    orientation_right: bubble_orientation_right,
                },
            )
            .unwrap();

        let _ = content.set_position(tauri::Position::Logical(new_content_pos));
    }
}

#[tauri::command]
pub fn cmd_start_dragging_widget<R: tauri::Runtime>(handle: tauri::AppHandle<R>) {
    let widget_window = handle.get_window(&window_controls::AppWindow::Widget.to_string());

    // Return if window is not found
    if widget_window.is_none() {
        return;
    }

    // Reposition Content Window and Widget Window
    if let Some(widget) = widget_window {
        let _ = widget.start_dragging();
    }
}
