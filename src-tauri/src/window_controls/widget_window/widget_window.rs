use core::panic;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use tauri::{Error, LogicalPosition, LogicalSize};

use crate::{
    ax_interaction::{app::observer_app::register_observer_app, models::app::ContentWindowState},
    window_controls::{
        actions::{
            close_window, create_window, current_monitor_of_window, get_position, get_size,
            is_visible, open_window, set_position,
        },
        default_properties,
        editor_window::EditorWindow,
        AppWindow, ContentWindow,
    },
};

use super::{
    decision_tree_show_hide_widget::{validate_decision_tree_show_hide_widget, ShowHide},
    dimension_calculations::{
        prevent_misalignement_of_content_and_widget, prevent_widget_position_off_screen,
    },
    listeners::{register_listener_app, register_listener_xcode},
};

pub static HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS: u64 = 200;
pub static HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS: u64 = 50;
pub static XCODE_EDITOR_NAME: &str = "Xcode";

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct WidgetWindow {
    pub app_handle: tauri::AppHandle,

    /// List of open editor windows. List is managed by WindowStateManager.
    pub editor_windows: Arc<Mutex<Vec<EditorWindow>>>,

    /// Properties of the content window
    // pub content_window: Arc<Mutex<ContentWindow>>,

    /// Identitfier of the currently focused editor window. Is None until the first window was focused.
    pub currently_focused_editor_window: Option<uuid::Uuid>,

    /// Each qualifying incoming event updates the instant until when the widget should be hidden.
    pub hide_until_instant: Instant,

    /// In case the focus switches from our app to an editor or vice versa it is possible, that there is
    /// a state where seemingly neither is in focus, only because the new "AXActivation" event from the
    /// newly focused entity hasn't arrived yet / wasn't processed yet.
    pub delay_hide_until_instant: Instant,

    /// Boolean saying if the currently focused application is Xcode.
    pub is_xcode_focused: bool,

    /// Boolean saying if the currently focused application is our app.
    pub is_app_focused: bool,

    /// Identitfier of the currently focused app window. Is None until the first window was focused.
    pub currently_focused_app_window: Option<AppWindow>,
}

impl WidgetWindow {
    pub fn new(
        app_handle: &tauri::AppHandle,
        editor_windows: &Arc<Mutex<Vec<EditorWindow>>>,
        content_window: &Arc<Mutex<ContentWindow>>,
    ) -> Self {
        // Create Tauri Window
        if create_window(&app_handle, AppWindow::Widget).is_err() {
            panic!("Could not create Widget Window");
        }

        // Register Observer for Widget AX Events
        if register_observer_app(&app_handle).is_err() {
            panic!("Could not register observer app");
        }

        Self {
            app_handle: app_handle.clone(),
            editor_windows: editor_windows.clone(),
            // content_window: content_window.clone(),
            hide_until_instant: Instant::now(),
            delay_hide_until_instant: Instant::now(),
            currently_focused_editor_window: None,
            is_xcode_focused: false,
            is_app_focused: false,
            currently_focused_app_window: None,
        }
    }

    pub fn setup_widget_listeners(
        app_handle: &tauri::AppHandle,
        widget_window: &Arc<Mutex<WidgetWindow>>,
    ) {
        // Register listener for AXEvents from our app
        register_listener_app(app_handle, &widget_window);

        // Register listener for xcode events relevant for displaying/hiding and positioning the widget
        register_listener_xcode(app_handle, &widget_window);
    }

    pub fn start_widget_visibility_control(
        app_handle: &tauri::AppHandle,
        widget_window: &Arc<Mutex<WidgetWindow>>,
    ) {
        control_widget_visibility(app_handle, &widget_window);
    }
}

fn control_widget_visibility(
    app_handle: &tauri::AppHandle,
    widget_props: &Arc<Mutex<WidgetWindow>>,
) {
    let widget_props_move_copy = widget_props.clone();
    let app_handle_move_copy = app_handle.clone();

    thread::spawn(move || {
        loop {
            // !!!!! Sleep first to not block the locked Mutexes afterwards !!!!!
            // ==================================================================
            thread::sleep(std::time::Duration::from_millis(100));
            // ==================================================================

            let widget = &*(widget_props_move_copy.lock().unwrap());
            let editor_windows = &*(widget.editor_windows.lock().unwrap());

            // Control widget visibility
            match validate_decision_tree_show_hide_widget(widget, editor_windows) {
                ShowHide::Show => {
                    // 0. Only proceed if widget is currently not visible
                    if !is_visible(&app_handle_move_copy, AppWindow::Widget) {
                        // 1. Get position from currently focused window
                        if let Some(focused_window_id) = widget.currently_focused_editor_window {
                            if let Some(editor_window) = editor_windows
                                .iter()
                                .find(|window| window.id == focused_window_id)
                            {
                                if let Some(mut widget_position) = editor_window.widget_position {
                                    prevent_widget_position_off_screen(
                                        &app_handle_move_copy,
                                        &mut widget_position,
                                    );

                                    // // If content window was open before, also check that it would not go offscreen
                                    // if editor_window.content_window_state
                                    //     == ContentWindowState::Active
                                    // {
                                    //     prevent_misalignement_of_content_and_widget(
                                    //         &app_handle_move_copy,
                                    //         &mut widget_position,
                                    //     );
                                    // }

                                    let _ = set_position(
                                        &widget.app_handle,
                                        AppWindow::Widget,
                                        &widget_position,
                                    );
                                }
                            }
                        }
                    }

                    open_window(&widget.app_handle, AppWindow::Widget)
                }
                ShowHide::Hide => close_window(&widget.app_handle, AppWindow::Widget),
                ShowHide::Continue => {}
            }
        }
    });
}

static POSITIONING_OFFSET_X: f64 = 24.;
static POSITIONING_OFFSET_Y: f64 = 8.;

fn check_widget_position_update_required(
    app_handle: &tauri::AppHandle,
    widget_position: &LogicalPosition<f64>,
    content_position: &LogicalPosition<f64>,
    content_size: &LogicalSize<f64>,
) -> Result<Option<LogicalPosition<f64>>, Error> {
    if let Some(monitor) = current_monitor_of_window(&app_handle, AppWindow::Widget) {
        let widget_size = LogicalSize {
            width: default_properties::size(&AppWindow::Widget).0,
            height: default_properties::size(&AppWindow::Widget).1,
        };

        let monitor_position = monitor.position().to_logical::<f64>(monitor.scale_factor());

        // only reposition, if widget is too close to upper end of screen
        if (monitor_position.y) < (widget_position.y - content_size.height) {
            return Ok(None);
        }

        let updated_widget_position = LogicalPosition {
            x: content_position.x + content_size.width - widget_size.width - POSITIONING_OFFSET_Y,
            y: monitor_position.y
                + content_size.height
                + (widget_size.height / 2.)
                + POSITIONING_OFFSET_X,
        };

        Ok(Some(updated_widget_position))
    } else {
        Err(Error::AssetNotFound("No screen found".to_string()))
    }
}
