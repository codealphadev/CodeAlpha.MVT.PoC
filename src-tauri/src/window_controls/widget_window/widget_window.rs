use std::{
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use tauri::{Error, LogicalPosition};

use crate::{
    ax_interaction::app::observer_app::register_observer_app,
    window_controls::{
        close_window, create_window, current_monitor_of_window, default_properties,
        editor_window::EditorWindow, open_window, set_position, AppWindow,
    },
};

use super::{
    decision_tree_show_hide_app::{validate_decision_tree_show_hide_widget, ShowHide},
    listeners::{register_listener_app, register_listener_xcode},
};

pub static HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS: u64 = 200;
pub static HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS: u64 = 50;
pub static XCODE_EDITOR_NAME: &str = "Xcode";

#[allow(dead_code)]
#[derive(Clone)]
pub struct WidgetWindow {
    pub app_handle: tauri::AppHandle,

    /// List of open editor windows. List is managed by WindowStateManager.
    pub editor_windows: Arc<Mutex<Vec<EditorWindow>>>,

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
    ) -> Result<Self, Error> {
        // Create Tauri Window
        create_window(&app_handle, AppWindow::Widget)?;

        // Register Observer for Widget AX Events
        if register_observer_app(&app_handle).is_err() {
            return Err(Error::CreateWindow);
        }

        Ok(Self {
            app_handle: app_handle.clone(),
            editor_windows: editor_windows.clone(),
            hide_until_instant: Instant::now(),
            delay_hide_until_instant: Instant::now(),
            currently_focused_editor_window: None,
            is_xcode_focused: false,
            is_app_focused: false,
            currently_focused_app_window: None,
        })
    }

    pub fn setup_widget_listeners(
        app_handle: &tauri::AppHandle,
        widget_props: &Arc<Mutex<WidgetWindow>>,
    ) {
        // Register listener for AXEvents from our app
        register_listener_app(&app_handle, &widget_props);

        // Register listener for xcode events relevant for displaying/hiding and positioning the widget
        register_listener_xcode(&app_handle, &widget_props);
    }

    pub fn start_widget_visibility_control(
        app_handle: &tauri::AppHandle,
        widget_props: &Arc<Mutex<WidgetWindow>>,
    ) {
        Self::control_widget_visibility(&app_handle, &widget_props);
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
                thread::sleep(std::time::Duration::from_millis(25));
                // ==================================================================

                let widget = &*(widget_props_move_copy.lock().unwrap());
                let editor_windows = &*(widget.editor_windows.lock().unwrap());

                // Update Widget Position
                // 1. Get position from currently focused window
                // 2. Get content dimensions to calculate full extent of visible windows
                // 3. Adapt position to prevent display offscreen

                // Control widget visibility
                match validate_decision_tree_show_hide_widget(widget, editor_windows) {
                    ShowHide::Show => {
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

                                    set_position(
                                        &widget.app_handle,
                                        AppWindow::Widget,
                                        &widget_position,
                                    );
                                }
                            }
                        }

                        // 2. Get content dimensions to calculate full extent of visible windows
                        // 3. Adapt position to prevent display offscreen
                        open_window(&widget.app_handle, AppWindow::Widget)
                    }
                    ShowHide::Hide => close_window(&widget.app_handle, AppWindow::Widget),
                    ShowHide::Continue => {}
                }
            }
        });
    }
}

fn prevent_widget_position_off_screen(
    app_handle: &tauri::AppHandle,
    widget_position: &mut LogicalPosition<f64>,
) {
    if let Some(monitor) = current_monitor_of_window(&app_handle, AppWindow::Widget) {
        // 0. Get Screen dimensions
        // TODO: figure out the correct screen
        let screen_position = (*monitor.position()).to_logical::<f64>(monitor.scale_factor());
        let screen_size = (*monitor.size()).to_logical::<f64>(monitor.scale_factor());

        // prevent widget from going off-screen
        if widget_position.x < screen_position.x {
            widget_position.x = screen_position.x;
        }
        if widget_position.y < screen_position.y {
            widget_position.y = screen_position.y;
        }
        if widget_position.x + default_properties::size(&AppWindow::Widget).0
            > screen_position.x + screen_size.width
        {
            widget_position.x = screen_position.x + screen_size.width
                - default_properties::size(&AppWindow::Widget).0
        }
        if widget_position.y + default_properties::size(&AppWindow::Widget).1
            > screen_position.y + screen_size.height
        {
            widget_position.y = screen_position.y + screen_size.height
                - default_properties::size(&AppWindow::Widget).1;
        }
    }
}
