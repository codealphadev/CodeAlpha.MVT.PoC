use std::sync::Arc;

use cocoa::{appkit::NSWindowOrderingMode, base::id, foundation::NSInteger};

use objc::{msg_send, sel, sel_impl};

use parking_lot::Mutex;
use tauri::Manager;
use window_shadows::set_shadow;

use crate::{
    app_handle,
    platform::macos::{get_menu_bar_height, models::app::AppWindowMovedMessage, AXEventApp},
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::{AppWindow, WindowLevel},
        utils::create_default_window_builder,
        windows::{widget_window::WIDGET_MAIN_WINDOW_OFFSET, WidgetWindow},
    },
};

use super::listeners::window_control_events_listener;

#[derive(Clone, Debug)]
pub struct MainWindow {
    app_handle: tauri::AppHandle,
}

impl MainWindow {
    pub fn new() -> Result<Self, tauri::Error> {
        let app_handle = app_handle();

        // Create Main Window at startup.
        // If the window is already created, don't open it again.
        if app_handle
            .get_window(&AppWindow::Main.to_string())
            .is_none()
        {
            create_default_window_builder(&app_handle, AppWindow::Main)?.build()?;
        }

        Ok(Self { app_handle })
    }

    pub fn set_macos_properties(&self) -> Option<()> {
        let ns_window_ptr_main = self
            .app_handle
            .get_window(&AppWindow::Main.to_string())?
            .ns_window();

        if let Ok(ns_window_ptr_main) = ns_window_ptr_main {
            unsafe {
                // Prevent the main from causing our application to take focus.
                let _: () = msg_send![ns_window_ptr_main as id, _setPreventsActivation: true];

                // Set the main's window level
                let _: () = msg_send![
                    ns_window_ptr_main as id,
                    setLevel: WindowLevel::Main as NSInteger
                ];
            }
        }

        Some(())
    }

    pub fn start_event_listeners(main_window: &Arc<Mutex<MainWindow>>) {
        window_control_events_listener(main_window);
    }

    pub fn show(&self, monitor: &LogicalFrame) -> Option<()> {
        let main_window_frame = Self::dimensions();
        Self::update(&main_window_frame.size, monitor)?;

        let main_tauri_window = self.app_handle.get_window(&AppWindow::Main.to_string())?;
        main_tauri_window.show().ok()?;

        set_shadow(&main_tauri_window, true).expect("Unsupported platform!");

        Some(())
    }

    pub fn update(updated_main_window_size: &LogicalSize, monitor: &LogicalFrame) -> Option<()> {
        let widget_frame = WidgetWindow::dimensions();

        let mut corrected_position = LogicalPosition {
            x: widget_frame.origin.x
                - (updated_main_window_size.width
                    - widget_frame.size.width
                    - WIDGET_MAIN_WINDOW_OFFSET),
            y: widget_frame.origin.y - updated_main_window_size.height,
        };

        let is_flipped = Self::is_main_window_flipped_horizontally(&corrected_position, &monitor);
        if is_flipped {
            corrected_position.x = widget_frame.origin.x - WIDGET_MAIN_WINDOW_OFFSET;
        }

        let (offscreen_dist_x, offscreen_dist_y) = Self::calc_off_screen_distance(
            &updated_main_window_size,
            &corrected_position,
            &monitor,
        );

        if let Some(offscreen_dist_x) = offscreen_dist_x {
            corrected_position.x += offscreen_dist_x;
        }

        if let Some(offscreen_dist_y) = offscreen_dist_y {
            corrected_position.y += offscreen_dist_y;
        }

        Self::update_widget_position(
            LogicalFrame {
                origin: corrected_position,
                size: *updated_main_window_size,
            },
            &monitor,
            is_flipped,
        );

        let main_tauri_window = app_handle().get_window(&AppWindow::Main.to_string())?;
        main_tauri_window
            .set_position(corrected_position.as_tauri_LogicalPosition())
            .ok()?;

        if *updated_main_window_size != WidgetWindow::dimensions().size {
            main_tauri_window
                .set_size(updated_main_window_size.as_tauri_LogicalSize())
                .ok()?;
        }

        Some(())
    }

    pub fn hide(&self) -> Option<()> {
        _ = self
            .app_handle
            .get_window(&AppWindow::Main.to_string())?
            .hide();

        Some(())
    }

    pub fn dimensions() -> LogicalFrame {
        let main_tauri_window = app_handle()
            .get_window(&AppWindow::Main.to_string())
            .expect("Could not get MainWindow!");

        let scale_factor = main_tauri_window
            .scale_factor()
            .expect("Could not get MainWindow scale factor!");
        let main_window_origin = LogicalPosition::from_tauri_LogicalPosition(
            &main_tauri_window
                .outer_position()
                .expect("Could not get MainWindow outer position!")
                .to_logical::<f64>(scale_factor),
        );
        let main_window_size = LogicalSize::from_tauri_LogicalSize(
            &main_tauri_window
                .outer_size()
                .expect("Could not get MainWindow outer size!")
                .to_logical::<f64>(scale_factor),
        );

        LogicalFrame {
            origin: main_window_origin,
            size: main_window_size,
        }
    }

    fn update_widget_position(
        main_window_frame: LogicalFrame,
        monitor: &LogicalFrame,
        is_main_window_flipped: bool,
    ) -> Option<()> {
        let widget_tauri_window_frame = WidgetWindow::dimensions();

        let updated_widget_window_origin = Self::compute_widget_origin(
            main_window_frame,
            widget_tauri_window_frame,
            &monitor,
            is_main_window_flipped,
        );

        let msg = AppWindowMovedMessage {
            window: AppWindow::Widget,
            window_position: updated_widget_window_origin.as_tauri_LogicalPosition(),
        };
        AXEventApp::AppWindowMoved(msg).publish_to_tauri(&app_handle());

        // Set widget position
        let widget_tauri_window = app_handle().get_window(&AppWindow::Widget.to_string())?;
        widget_tauri_window
            .set_position(updated_widget_window_origin.as_tauri_LogicalPosition())
            .ok()?;

        Some(())
    }

    fn calc_off_screen_distance(
        main_window_size: &LogicalSize,
        main_window_origin: &LogicalPosition,
        monitor: &LogicalFrame,
    ) -> (Option<f64>, Option<f64>) {
        let mut dist_x: Option<f64> = None;
        let mut dist_y: Option<f64> = None;

        // prevent main from going off-screen
        if main_window_origin.x < monitor.origin.x {
            dist_x = Some(monitor.origin.x - main_window_origin.x);
        }
        if main_window_origin.y < monitor.origin.y {
            dist_y = Some(monitor.origin.y - main_window_origin.y);
        }
        if main_window_origin.x + main_window_size.width > monitor.origin.x + monitor.size.width {
            dist_x = Some(
                monitor.origin.x + monitor.size.width
                    - main_window_size.width
                    - main_window_origin.x,
            );
        }
        if main_window_origin.y + main_window_size.height > monitor.origin.y + monitor.size.height {
            dist_y = Some(
                monitor.origin.y + monitor.size.height
                    - main_window_size.height
                    - main_window_origin.y,
            );
        }

        (dist_x, dist_y)
    }

    fn is_main_window_flipped_horizontally(
        main_window_origin: &LogicalPosition,
        monitor: &LogicalFrame,
    ) -> bool {
        if main_window_origin.x < monitor.origin.x {
            true
        } else {
            false
        }
    }

    fn compute_widget_origin(
        main_window_frame: LogicalFrame,
        widget_tauri_window_frame: LogicalFrame,
        monitor: &LogicalFrame,
        is_main_window_flipped: bool,
    ) -> LogicalPosition {
        let mut updated_widget_window_origin = LogicalPosition {
            x: main_window_frame.origin.x + main_window_frame.size.width
                - widget_tauri_window_frame.size.width
                - WIDGET_MAIN_WINDOW_OFFSET,
            y: main_window_frame.origin.y + main_window_frame.size.height,
        };

        if is_main_window_flipped {
            updated_widget_window_origin.x = main_window_frame.origin.x + WIDGET_MAIN_WINDOW_OFFSET;
        }

        // If monitor is the primary monitor, we need to account for the menu bar.
        let menu_bar_height = get_menu_bar_height(&monitor);
        if menu_bar_height > 0. {
            if main_window_frame.origin.y < menu_bar_height && main_window_frame.origin.y >= 0. {
                // Case: the main window is positioned where the menu bar is -> it will be pushed down
                // and overlap with the repositioned widget.
                updated_widget_window_origin.y += menu_bar_height - main_window_frame.origin.y;
            }
        }
        updated_widget_window_origin
    }
}

#[tauri::command]
pub fn cmd_rebind_main_widget() {
    // Rebind the MainWindow and WidgetWindow. Because of how MacOS works, we need to have some
    // delay between setting a new position and recreating the parent/child relationship.
    // Pausing the main thread is not possible. Also, running this task async is also not trivial.
    // We send a message to the main thread to run this task.
    // EventWindowControls::RebindMainAndWidget.publish_to_tauri(&app_handle());
    _ = rebind_main_and_widget_window();
}

fn rebind_main_and_widget_window() -> Option<()> {
    let widget_tauri_window = app_handle().get_window(&AppWindow::Widget.to_string())?;

    let main_tauri_window = app_handle().get_window(&AppWindow::Main.to_string())?;
    if let (Ok(parent_ptr), Ok(child_ptr)) = (
        widget_tauri_window.ns_window(),
        main_tauri_window.ns_window(),
    ) {
        unsafe {
            let _: () = msg_send![parent_ptr as id, addChildWindow: child_ptr as id ordered: NSWindowOrderingMode::NSWindowBelow];
        }
    }

    Some(())
}
#[cfg(test)]
mod tests_main_window {

    use crate::{
        utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
        window_controls::windows::widget_window::WIDGET_MAIN_WINDOW_OFFSET,
    };

    use super::MainWindow;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_calc_offscreen_distance() {
        let main_window_size = LogicalSize {
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

        let main_window_origin = LogicalPosition { x: 0., y: 0. };
        let (dist_x, dist_y) =
            MainWindow::calc_off_screen_distance(&main_window_size, &main_window_origin, &monitor);

        assert_eq!(dist_x, None);
        assert_eq!(dist_y, None);

        let main_window_origin = LogicalPosition { x: 100., y: 100. };
        let (dist_x, dist_y) =
            MainWindow::calc_off_screen_distance(&main_window_size, &main_window_origin, &monitor);

        assert_eq!(dist_x, Some(-48.));
        assert_eq!(dist_y, Some(-48.));

        let main_window_origin = LogicalPosition {
            x: 100. - main_window_size.width,
            y: 100. - main_window_size.height,
        };
        let (dist_x, dist_y) =
            MainWindow::calc_off_screen_distance(&main_window_size, &main_window_origin, &monitor);

        assert_eq!(dist_x, None);
        assert_eq!(dist_y, None);
    }

    #[test]
    fn test_compute_widget_origin() {
        let main_window_frame = LogicalFrame {
            origin: LogicalPosition { x: 0., y: 0. },
            size: LogicalSize {
                width: 100.,
                height: 100.,
            },
        };
        let widget_tauri_window_frame = LogicalFrame {
            origin: LogicalPosition { x: 0., y: 0. },
            size: LogicalSize {
                width: 48.,
                height: 48.,
            },
        };
        let primary_monitor = LogicalFrame {
            origin: LogicalPosition { x: 0., y: 0. },
            size: LogicalSize {
                width: 1000.,
                height: 1000.,
            },
        };

        let secondary_monitor = LogicalFrame {
            origin: LogicalPosition { x: -1., y: -1. },
            size: LogicalSize {
                width: 1000.,
                height: 1000.,
            },
        };

        let updated_widget_window_origin = MainWindow::compute_widget_origin(
            main_window_frame,
            widget_tauri_window_frame,
            &primary_monitor,
            false,
        );

        // Placement should be bottom right cornor of main window with an offset of WIDGET_MAIN_WINDOW_OFFSET.
        assert_eq!(
            updated_widget_window_origin.x,
            0. /*main_window_frame.origin.x*/ + 100. /*main_window_frame.size.width*/
                    - 48. /*widget_tauri_window_frame.size.width*/
                    - WIDGET_MAIN_WINDOW_OFFSET
        );
        assert_eq!(
            updated_widget_window_origin.y,
            100. /*main_window_frame.size.height*/ + 38. /*menu bar height*/
        );

        let updated_widget_window_origin = MainWindow::compute_widget_origin(
            main_window_frame,
            widget_tauri_window_frame,
            &secondary_monitor,
            false,
        );

        // Placement should be bottom right cornor of main window with an offset of WIDGET_MAIN_WINDOW_OFFSET.
        assert_eq!(
            updated_widget_window_origin.x,
            0. /*main_window_frame.origin.x*/ + 100. /*main_window_frame.size.width*/
                    - 48. /*widget_tauri_window_frame.size.width*/
                    - WIDGET_MAIN_WINDOW_OFFSET
        );
        assert_eq!(
            updated_widget_window_origin.y,
            100. /*main_window_frame.size.height*/
        );

        let updated_widget_window_origin = MainWindow::compute_widget_origin(
            main_window_frame,
            widget_tauri_window_frame,
            &secondary_monitor,
            true,
        );

        // Placement should be bottom LEFT cornor of main window with an offset of WIDGET_MAIN_WINDOW_OFFSET.
        assert_eq!(
            updated_widget_window_origin.x,
            0. /*main_window_frame.origin.x*/ + WIDGET_MAIN_WINDOW_OFFSET
        );
    }

    #[test]
    fn test_is_main_window_flipped_horizontally() {
        let main_window_frame = LogicalFrame {
            origin: LogicalPosition { x: 0., y: 0. },
            size: LogicalSize {
                width: 100.,
                height: 100.,
            },
        };

        let primary_monitor = LogicalFrame {
            origin: LogicalPosition { x: 0., y: 0. },
            size: LogicalSize {
                width: 1000.,
                height: 1000.,
            },
        };

        let secondary_monitor = LogicalFrame {
            origin: LogicalPosition { x: 1., y: 1. },
            size: LogicalSize {
                width: 1000.,
                height: 1000.,
            },
        };

        let is_flipped = MainWindow::is_main_window_flipped_horizontally(
            &main_window_frame.origin,
            &primary_monitor,
        );

        assert_eq!(is_flipped, false);

        let is_flipped = MainWindow::is_main_window_flipped_horizontally(
            &main_window_frame.origin,
            &secondary_monitor,
        );

        assert_eq!(is_flipped, true);
    }
}
