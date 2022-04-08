use serde::{Deserialize, Serialize};
use tauri::{Error, Manager};

// This file contains the list of all the app windows and their initial sizes and features
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AppWindow {
    Settings,
    Analytics,
    Widget,
    Content,
    None,
}

impl std::fmt::Display for AppWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub mod default_properties {
    use super::*;

    pub fn url(window: &AppWindow) -> String {
        match window {
            AppWindow::Settings => {
                format!("{}{}", r"\", AppWindow::Settings.to_string().to_lowercase())
            }

            AppWindow::Analytics => format!(
                "{}{}",
                r"\",
                AppWindow::Analytics.to_string().to_lowercase()
            ),
            AppWindow::Widget => {
                format!("{}{}", r"\", AppWindow::Widget.to_string().to_lowercase())
            }

            AppWindow::Content => {
                format!("{}{}", r"\", AppWindow::Content.to_string().to_lowercase())
            }

            AppWindow::None => "".to_string(),
        }
    }

    pub fn title(window: &AppWindow) -> String {
        match window {
            AppWindow::Settings => "CodeAlpha - Settings".to_string(),
            AppWindow::Analytics => "CodeAlpha - Analytics".to_string(),
            AppWindow::Widget => "CodeAlpha - Widget".to_string(),
            AppWindow::Content => "CodeAlpha - Guide".to_string(),
            AppWindow::None => "".to_string(),
        }
    }

    pub fn size(window: &AppWindow) -> (f64, f64) {
        match window {
            AppWindow::Settings => (800.0, 600.0),
            AppWindow::Analytics => (1280.0, 786.0),
            AppWindow::Widget => (48.0, 48.0),
            AppWindow::Content => (322.0, 398.0),
            AppWindow::None => (0.0, 0.0),
        }
    }

    // If we tie windows together as parent/child, they will be moved together.
    // For now, only the content window is supposed to have the Widget as a parent.
    pub fn parent_window(window: &AppWindow) -> AppWindow {
        match window {
            AppWindow::Settings => AppWindow::None,
            AppWindow::Analytics => AppWindow::None,
            AppWindow::Widget => AppWindow::None,
            AppWindow::Content => AppWindow::Widget,
            AppWindow::None => AppWindow::None,
        }
    }

    pub fn is_resizable(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::None => false,
        }
    }
    pub fn is_transparent(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => false,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::None => false,
        }
    }

    pub fn has_decorations(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::None => false,
        }
    }

    pub fn is_visible(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::None => true,
        }
    }

    pub fn is_always_on_top(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => false,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::None => false,
        }
    }

    pub fn skip_taskbar(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::None => true,
        }
    }
}

static POSITIONING_OFFSET_X: f64 = 24.;
static POSITIONING_OFFSET_Y: f64 = 8.;

#[derive(Clone, serde::Serialize)]
struct PayloadBubbleOrientationEvent {
    orientation_right: bool,
}

pub fn special_default_position_for_content_window(
    handle: &tauri::AppHandle,
) -> Result<Option<(f64, f64)>, Error> {
    if let Some(widget_window) = handle.get_window(&AppWindow::Widget.to_string()) {
        if let Some(screen) = widget_window.current_monitor()? {
            let screen_scale_factor = screen.scale_factor();
            let pos_screen = screen.position().to_logical::<f64>(screen_scale_factor);

            let pos_widget = widget_window
                .outer_position()?
                .to_logical::<f64>(screen_scale_factor);

            let widget_size = tauri::LogicalSize::<f64> {
                width: widget_window
                    .outer_size()?
                    .to_logical::<f64>(screen_scale_factor)
                    .width,
                height: widget_window
                    .outer_size()?
                    .to_logical::<f64>(screen_scale_factor)
                    .height as f64,
            };

            let content_size = tauri::LogicalSize::<f64> {
                width: default_properties::size(&AppWindow::Content).0,
                height: default_properties::size(&AppWindow::Content).1,
            };

            let mut new_content_pos = tauri::LogicalPosition {
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
            handle.emit_to(
                &AppWindow::Content.to_string(),
                "evt-bubble-icon-orientation",
                PayloadBubbleOrientationEvent {
                    orientation_right: bubble_orientation_right,
                },
            )?;

            return Ok(Some((new_content_pos.x as f64, new_content_pos.y as f64)));
        }
    }

    Ok(None)
}
