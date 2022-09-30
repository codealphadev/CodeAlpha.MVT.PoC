use std::sync::Arc;

use cocoa::{base::id, foundation::NSInteger};

use objc::{msg_send, sel, sel_impl};

use parking_lot::Mutex;
use tauri::Manager;
use window_shadows::set_shadow;

use crate::{
    app_handle,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::{default_properties, AppWindow, WindowLevel},
        utils::create_default_window_builder,
        windows::MainWindow,
    },
};

use super::listeners::window_control_events_listener;

static WIDGET_OFFSET: f64 = 75.;

#[derive(Clone, Debug)]
pub struct WidgetWindow {
    app_handle: tauri::AppHandle,

    // Is the main window shown
    main_window_shown: Option<bool>,

    // The widget window's size
    size: LogicalSize,
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

    pub fn set_main_window_shown(&mut self, main_window_shown: bool) {
        self.main_window_shown = Some(main_window_shown);
    }

    pub fn show(
        &self,
        updated_widget_position: &Option<LogicalPosition>,
        editor_textarea: &LogicalFrame,
        monitor: &LogicalFrame,
    ) -> Option<()> {
        let tauri_window = self.app_handle.get_window(&AppWindow::Widget.to_string())?;

        // In case the widget has never been moved by the user, we set an initial position
        // based on the editor textarea.
        let mut widget_position = if let Some(position) = updated_widget_position.to_owned() {
            position
        } else {
            self.initial_widget_position(editor_textarea)
        };

        // Determine if the widget would be off-screen and needs to be moved.
        let mut main_window_frame = None;
        if let Some(main_window_shown) = self.main_window_shown {
            if main_window_shown {
                main_window_frame = Some(MainWindow::dimensions());
            }
        }
        let (offscreen_dist_x, offscreen_dist_y) = Self::calc_off_screen_distance(
            &self.size,
            &widget_position,
            main_window_frame,
            &monitor,
        );

        if let Some(offscreen_dist_x) = offscreen_dist_x {
            widget_position.x += offscreen_dist_x;
        }

        if let Some(offscreen_dist_y) = offscreen_dist_y {
            widget_position.y += offscreen_dist_y;
        }

        // Needs to be reset on each show.
        set_shadow(&tauri_window, true).expect("Unsupported platform!");

        tauri_window
            .set_position(widget_position.as_tauri_LogicalPosition())
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
        widget_size: &LogicalSize,
        widget_position: &LogicalPosition,
        main_window_frame: Option<LogicalFrame>,
        monitor: &LogicalFrame,
    ) -> (Option<f64>, Option<f64>) {
        let mut dist_x: Option<f64> = None;
        let mut dist_y: Option<f64> = None;

        // For the off-screen-check we add the widget to the container
        let mut corrected_size = widget_size.to_owned();
        let mut corrected_position = widget_position.to_owned();

        if let Some(main_window_frame) = main_window_frame {
            corrected_position = LogicalPosition {
                x: corrected_position.x - (main_window_frame.size.width - corrected_size.width),
                y: corrected_position.y - main_window_frame.size.height,
            };

            corrected_size = LogicalSize {
                width: main_window_frame.size.width,
                height: corrected_size.height + main_window_frame.size.height,
            };

            // If monitor is the primary monitor, we need to account for the menu bar.
            if monitor.origin == (LogicalPosition { x: 0., y: 0. }) {
                corrected_size.height += main_window_frame.origin.y;
                corrected_position.y -= main_window_frame.origin.y;
            }
        }

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
        let widget_tauri_window = app_handle()
            .get_window(&AppWindow::Widget.to_string())
            .expect("Could not get WidgetWindow!");

        let scale_factor = widget_tauri_window
            .scale_factor()
            .expect("Could not get WidgetWindow scale factor!");
        let widget_position = LogicalPosition::from_tauri_LogicalPosition(
            &widget_tauri_window
                .outer_position()
                .expect("Could not get WidgetWindow outer position!")
                .to_logical::<f64>(scale_factor),
        );
        let widget_size = LogicalSize::from_tauri_LogicalSize(
            &widget_tauri_window
                .outer_size()
                .expect("Could not get WidgetWindow outer size!")
                .to_logical::<f64>(scale_factor),
        );

        LogicalFrame {
            origin: widget_position,
            size: widget_size,
        }
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
            WidgetWindow::calc_off_screen_distance(&widget_size, &widget_position, None, &monitor);

        assert_eq!(dist_x, None);
        assert_eq!(dist_y, None);

        let widget_position = LogicalPosition { x: 100., y: 100. };
        let (dist_x, dist_y) =
            WidgetWindow::calc_off_screen_distance(&widget_size, &widget_position, None, &monitor);

        assert_eq!(dist_x, Some(-48.));
        assert_eq!(dist_y, Some(-48.));

        let widget_position = LogicalPosition {
            x: 100. - widget_size.width,
            y: 100. - widget_size.height,
        };
        let (dist_x, dist_y) =
            WidgetWindow::calc_off_screen_distance(&widget_size, &widget_position, None, &monitor);

        assert_eq!(dist_x, None);
        assert_eq!(dist_y, None);

        // with main window
    }

    #[test]
    fn test_calc_offscreen_distance_with_main_window() {
        let widget_position = LogicalPosition { x: 0., y: 0. };
        let widget_size = LogicalSize {
            width: 48.,
            height: 48.,
        };

        let main_window_frame = LogicalFrame {
            origin: LogicalPosition { x: -50., y: 38. },
            size: LogicalSize {
                width: 50.,
                height: 30.,
            },
        };

        let monitor = LogicalFrame {
            origin: LogicalPosition { x: 0., y: 0. },
            size: LogicalSize {
                width: 100.,
                height: 100.,
            },
        };

        let (dist_x, dist_y) = WidgetWindow::calc_off_screen_distance(
            &widget_size,
            &widget_position,
            Some(main_window_frame),
            &monitor,
        );

        assert_eq!(dist_x, Some(2.));
        assert_eq!(dist_y, Some(68.));
    }
}
