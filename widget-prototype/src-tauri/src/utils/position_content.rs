use tauri::{LogicalPosition, LogicalSize, Manager};

use super::window_controls;

#[allow(dead_code)]
pub fn position_content<R: tauri::Runtime>(
    handle: tauri::AppHandle<R>,
    current_content_size: Option<LogicalSize<f64>>,
) -> Result<(), tauri::Error> {
    let content_window = handle.get_window(&window_controls::AppWindow::Content.to_string());
    let widget_window = handle.get_window(&window_controls::AppWindow::Widget.to_string());

    // Reposition Content Window and Widget Window
    if let Some(content) = content_window {
        if let Some(widget) = widget_window {
            let screen = widget.current_monitor()?.unwrap();
            let screen_scale_factor = screen.scale_factor();
            let screen_position = screen.position().to_logical::<f64>(screen_scale_factor);

            let pos_widget = (widget.outer_position())?.to_logical::<f64>(screen_scale_factor);

            let widget_size = LogicalSize::<f64> {
                width: widget
                    .outer_size()?
                    .to_logical::<f64>(screen_scale_factor)
                    .width,
                height: widget
                    .outer_size()?
                    .to_logical::<f64>(screen_scale_factor)
                    .height as f64,
            };
            let content_size = LogicalSize::<f64> {
                width: content
                    .outer_size()?
                    .to_logical::<f64>(screen_scale_factor)
                    .width as f64,
                height: content
                    .outer_size()?
                    .to_logical::<f64>(screen_scale_factor)
                    .height as f64,
            };

            // UGLYYYY
            let mut new_content_pos = LogicalPosition {
                x: pos_widget.x + (widget_size.width - content_size.width),
                y: pos_widget.y - content_size.height,
            };

            if let Some(content_size) = current_content_size {
                new_content_pos = LogicalPosition {
                    x: pos_widget.x + (widget_size.width - content_size.width),
                    y: pos_widget.y - content_size.height,
                }
            }

            // Check if the content would be outside the upper end of the screen, if so, move widget accordingly before positioning content
            let updated_content_pos = reposition_widget(widget, new_content_pos);

            match updated_content_pos {
                Ok(mut position) => {
                    // Check if the content would be outside the left end of the screen, if so, flip the content window position horizontally

                    if screen_position.x > position.x {
                        position.x = pos_widget.x;
                    }

                    return content.set_position(tauri::Position::Logical(position));
                }
                Err(e) => return Err(e),
            };
        }
    }

    return Ok(());
}

fn reposition_widget<R: tauri::Runtime>(
    widget: tauri::Window<R>,
    content_position: LogicalPosition<f64>,
) -> Result<LogicalPosition<f64>, tauri::Error> {
    let screen = widget.current_monitor()?.unwrap();
    let screen_scale_factor = screen.scale_factor();
    let screen_position = screen.position().to_logical::<f64>(screen_scale_factor);

    let pos_widget = widget
        .outer_position()?
        .to_logical::<f64>(screen_scale_factor);

    if screen_position.y <= content_position.y {
        return Ok(content_position);
    }

    let new_widget_pos = LogicalPosition {
        x: pos_widget.x,
        y: pos_widget.y + (screen_position.y - content_position.y),
    };

    match widget.set_position(tauri::Position::Logical(new_widget_pos)) {
        Ok(()) => {
            let updated_content_pos = LogicalPosition {
                x: content_position.x,
                y: content_position.y - (screen_position.y - content_position.y),
            };

            return Ok(updated_content_pos);
        }
        Err(e) => return Err(e),
    };
}
