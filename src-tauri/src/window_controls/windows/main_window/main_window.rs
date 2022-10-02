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
        config::{default_properties, AppWindow, WindowLevel},
        utils::create_default_window_builder,
        windows::{
            utils::{app_window_dimensions, register_tracking_area, update_tracking_area},
            widget_window::WIDGET_MAIN_WINDOW_OFFSET,
            WidgetWindow,
        },
        EventTrackingArea, TrackingArea,
    },
};

use super::listeners::window_control_events_listener;

#[derive(Clone, Debug)]
pub struct MainWindow {
    app_handle: tauri::AppHandle,

    // We keep track of the window's size purely because the parent-child relationship
    // in macOS introduces weird behavior when tied windows between screens. If the child
    // window was resized before it snaps back to initial size at startup the moment both
    // move to a different screen. Our workaround for now is to reset the size on "show".
    size: LogicalSize,
    // The window's tracking area
    tracking_area: TrackingArea,
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

        let (width, height) = default_properties::size(&AppWindow::Main);

        Ok(Self {
            app_handle,
            size: LogicalSize { width, height },
            tracking_area: Self::register_tracking_area(),
        })
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

    pub fn show(&mut self, monitor: &LogicalFrame) -> Option<()> {
        self.update(&self.size.clone(), monitor, true)?;

        let main_tauri_window = self.app_handle.get_window(&AppWindow::Main.to_string())?;
        main_tauri_window.show().ok()?;

        set_shadow(&main_tauri_window, true).expect("Unsupported platform!");

        self.update_tracking_area(true);

        Some(())
    }

    pub fn update(
        &mut self,
        updated_main_window_size: &LogicalSize,
        monitor: &LogicalFrame,
        is_visible: bool,
    ) -> Option<()> {
        // Only update MainWindow origin when the window is visible or _goes_ directly visible afterwards
        let main_tauri_window = app_handle().get_window(&AppWindow::Main.to_string())?;

        if is_visible == true {
            let widget_frame = WidgetWindow::dimensions();

            let mut corrected_position = LogicalPosition {
                x: widget_frame.origin.x
                    - (updated_main_window_size.width
                        - widget_frame.size.width
                        - WIDGET_MAIN_WINDOW_OFFSET),
                y: widget_frame.origin.y - updated_main_window_size.height,
            };

            // If the `main_window_origin` would be within the menu bar area, we need to account for that.
            corrected_position.y += Self::compute_menu_bar_diff(
                &corrected_position,
                &monitor.origin,
                get_menu_bar_height(&monitor),
            );

            let is_flipped =
                Self::is_main_window_flipped_horizontally(&corrected_position, &monitor);
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
                is_flipped,
            );

            main_tauri_window
                .set_position(corrected_position.as_tauri_LogicalPosition())
                .ok()?;
        }

        if *updated_main_window_size != MainWindow::dimensions().size {
            self.size = *updated_main_window_size;
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

        self.update_tracking_area(false);

        Some(())
    }

    fn update_tracking_area(&self, is_visible: bool) {
        update_tracking_area(AppWindow::Main, self.tracking_area.clone(), is_visible)
    }

    fn register_tracking_area() -> TrackingArea {
        register_tracking_area(AppWindow::Main)
    }

    pub fn dimensions() -> LogicalFrame {
        app_window_dimensions(AppWindow::Main)
    }

    fn update_widget_position(
        main_window_frame: LogicalFrame,
        is_main_window_flipped: bool,
    ) -> Option<()> {
        let widget_tauri_window_frame = WidgetWindow::dimensions();

        let updated_widget_window_origin = Self::compute_widget_origin(
            main_window_frame,
            widget_tauri_window_frame,
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

        updated_widget_window_origin
    }

    fn compute_menu_bar_diff(
        main_window_origin: &LogicalPosition,
        monitor_origin: &LogicalPosition,
        menu_bar_height: f64,
    ) -> f64 {
        // If the `main_window_origin` would be within the menu bar area, we need to account for that.
        let mut menu_bar_diff = 0.;
        if main_window_origin.y < monitor_origin.y + menu_bar_height {
            menu_bar_diff = menu_bar_height + (monitor_origin.y - main_window_origin.y);
        }

        menu_bar_diff
    }
}

impl Drop for MainWindow {
    fn drop(&mut self) {
        EventTrackingArea::Remove(vec![self.tracking_area.id]).publish_to_tauri(&app_handle());
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
mod tests {

    use crate::{
        utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
        window_controls::windows::widget_window::WIDGET_MAIN_WINDOW_OFFSET,
    };

    use super::MainWindow;
    use pretty_assertions::assert_eq;

    #[test]
    fn calc_offscreen_distance() {
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
    fn compute_widget_origin() {
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

        let updated_widget_window_origin =
            MainWindow::compute_widget_origin(main_window_frame, widget_tauri_window_frame, false);

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

        let updated_widget_window_origin =
            MainWindow::compute_widget_origin(main_window_frame, widget_tauri_window_frame, false);

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

        let updated_widget_window_origin =
            MainWindow::compute_widget_origin(main_window_frame, widget_tauri_window_frame, true);

        // Placement should be bottom LEFT cornor of main window with an offset of WIDGET_MAIN_WINDOW_OFFSET.
        assert_eq!(
            updated_widget_window_origin.x,
            0. /*main_window_frame.origin.x*/ + WIDGET_MAIN_WINDOW_OFFSET
        );
    }

    #[test]
    fn is_main_window_flipped_horizontally() {
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

    #[test]
    fn menu_bar_diff() {
        let main_window_origin1 = LogicalPosition { x: 0., y: 0. };
        let main_window_origin2 = LogicalPosition {
            x: -1000.,
            y: -980.,
        };
        let main_window_origin3 = LogicalPosition {
            x: -1000.,
            y: -950.,
        };
        let main_window_origin4 = LogicalPosition {
            x: -1000.,
            y: -1000.,
        };
        let main_window_origin5 = LogicalPosition {
            x: -1000.,
            y: -1100.,
        };

        let monitor_origin1 = LogicalPosition { x: 0., y: 0. };
        let monitor_origin2 = LogicalPosition {
            x: -1000.,
            y: -1000.,
        };

        let menu_bar_height = 38.;

        let diff_scenario1 = MainWindow::compute_menu_bar_diff(
            &main_window_origin1,
            &monitor_origin1,
            menu_bar_height,
        );

        assert_eq!(diff_scenario1, 38.);

        let diff_scenario2 = MainWindow::compute_menu_bar_diff(
            &main_window_origin2,
            &monitor_origin2,
            menu_bar_height,
        );

        assert_eq!(diff_scenario2, 18.);

        let diff_scenario3 = MainWindow::compute_menu_bar_diff(
            &main_window_origin3,
            &monitor_origin2,
            menu_bar_height,
        );

        assert_eq!(diff_scenario3, 0.);

        let diff_scenario4 = MainWindow::compute_menu_bar_diff(
            &main_window_origin4,
            &monitor_origin2,
            menu_bar_height,
        );

        assert_eq!(diff_scenario4, 38.);

        let diff_scenario5 = MainWindow::compute_menu_bar_diff(
            &main_window_origin5,
            &monitor_origin2,
            menu_bar_height,
        );

        assert_eq!(diff_scenario5, 138.);
    }
}
