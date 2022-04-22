use core::panic;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::{
    ax_interaction::{app::observer_app::register_observer_app, models::app::ContentWindowState},
    window_controls::{
        actions::{close_window, create_window, is_visible, open_window, set_position},
        editor_window::EditorWindow,
        AppWindow,
    },
};

use super::{
    decision_tree_show_hide_widget::{validate_decision_tree_show_hide_widget, ShowHide},
    dimension_calculations::prevent_widget_position_off_screen,
    listeners::{register_listener_app, register_listener_xcode},
    prevent_misalignement_of_content_and_widget,
};

pub static HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS: u64 = 200;
pub static HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS: u64 = 50;
pub static XCODE_EDITOR_NAME: &str = "Xcode";

#[derive(Clone, Debug)]
pub struct WidgetWindow {
    pub app_handle: tauri::AppHandle,

    /// List of open editor windows. List is managed by WindowStateManager.
    pub editor_windows: Arc<Mutex<Vec<EditorWindow>>>,

    /// Identitfier of the currently focused editor window. Is None until the first window was focused.
    pub currently_focused_editor_window: Option<uuid::Uuid>,

    /// Each qualifying incoming event updates the instant until when the widget should be hidden.
    pub temporary_hide_until_instant: Instant,

    /// If an event requires the widget to be temporarily hidden, it triggers a routine that monitors when
    /// the widget should be shown again. In case another event occurs and this variable is set to true,
    /// only 'temporary_hide_until_instant' will be updated.
    pub temporary_hide_check_active: bool,

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
            temporary_hide_until_instant: Instant::now(),
            temporary_hide_check_active: false,
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
        //control_widget_visibility(app_handle, &widget_window);
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
            thread::sleep(std::time::Duration::from_millis(50));
            // ==================================================================

            let widget = &*(widget_props_move_copy.lock().unwrap());
            let editor_windows = &mut *(widget.editor_windows.lock().unwrap());

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

                                    // If content window was open before, also check that it would not go offscreen
                                    if editor_window.content_window_state
                                        == ContentWindowState::Active
                                    {
                                        prevent_misalignement_of_content_and_widget(
                                            &app_handle_move_copy,
                                            &mut widget_position,
                                        );
                                    }

                                    let _ = set_position(
                                        &widget.app_handle,
                                        AppWindow::Widget,
                                        &widget_position,
                                    );
                                }
                            }
                        }
                    }

                    // // Recover ContentWindowState
                    // if let Some(focused_window_id) = widget.currently_focused_editor_window {
                    //     if let Some(editor_window) = editor_windows
                    //         .iter()
                    //         .find(|window| window.id == focused_window_id)
                    //     {
                    //         match dbg!(editor_window.content_window_state) {
                    //             ContentWindowState::Active => {
                    //                 let _ = content_window::open(&app_handle_move_copy);
                    //             }
                    //             ContentWindowState::Inactive => {
                    //                 let _ = content_window::hide(&app_handle_move_copy);
                    //             }
                    //         }
                    //     }
                    // }

                    open_window(&widget.app_handle, AppWindow::Widget)
                }
                ShowHide::Hide => {
                    // Preserve the state of the content window
                    let is_content_visible = is_visible(&app_handle_move_copy, AppWindow::Content);
                    if let Some(focused_window_id) = widget.currently_focused_editor_window {
                        if let Some(editor_window) = editor_windows
                            .iter_mut()
                            .find(|window| window.id == focused_window_id)
                        {
                            if is_content_visible {
                                editor_window
                                    .update_content_window_state(&ContentWindowState::Active);
                            } else {
                                editor_window
                                    .update_content_window_state(&ContentWindowState::Inactive);
                            }
                        }
                    }

                    close_window(&widget.app_handle, AppWindow::Widget)
                }
                ShowHide::Continue => {}
            }
        }
    });
}

pub fn temporary_hide_check_routine(
    app_handle: &tauri::AppHandle,
    widget_props: &Arc<Mutex<WidgetWindow>>,
) {
    let widget = &mut *(widget_props.lock().unwrap());

    // Update the Instant time stamp when the widget should be shown again
    widget.temporary_hide_until_instant =
        Instant::now() + Duration::from_millis(HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS);

    // Check if another instance of this routine is already running
    if widget.temporary_hide_check_active {
        return;
    }

    // Gracefully hide widget
    let editor_windows = &mut *(widget.editor_windows.lock().unwrap());
    hide_widget_routine(app_handle, widget, editor_windows);

    // Start temporary hide check routine
    let widget_props_move_copy = widget_props.clone();
    let app_handle_move_copy = app_handle.clone();

    thread::spawn(move || loop {
        // !!!!! Sleep first to not block the locked Mutexes afterwards !!!!!
        // ==================================================================
        thread::sleep(std::time::Duration::from_millis(25));
        // ==================================================================

        let widget_window = &mut *(widget_props_move_copy.lock().unwrap());

        if widget_window.temporary_hide_until_instant > Instant::now() {
            let editor_windows = &mut *(widget_window.editor_windows.lock().unwrap());
            show_widget_routine(&app_handle_move_copy, &widget_window, &editor_windows);

            // Indicate that the routine finished
            widget_window.temporary_hide_check_active = false;
            break;
        }
    });
}

pub fn show_widget_routine(
    app_handle: &tauri::AppHandle,
    widget: &WidgetWindow,
    editor_windows: &Vec<EditorWindow>,
) {
    println!("show_widget_routine");
    // Check if the widget position should be updated before showing it
    if let Some(focused_window_id) = widget.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows
            .iter()
            .find(|window| window.id == focused_window_id)
        {
            if let Some(mut widget_position) = editor_window.widget_position {
                prevent_widget_position_off_screen(&app_handle, &mut widget_position);

                // If content window was open before, also check that it would not go offscreen
                if editor_window.content_window_state == ContentWindowState::Active {
                    prevent_misalignement_of_content_and_widget(&app_handle, &mut widget_position);
                }

                let _ = set_position(&widget.app_handle, AppWindow::Widget, &widget_position);
            }
        }
    }

    open_window(&widget.app_handle, AppWindow::Widget)
}

pub fn hide_widget_routine(
    app_handle: &tauri::AppHandle,
    widget: &WidgetWindow,
    editor_windows: &mut Vec<EditorWindow>,
) {
    // Preserve the state of the content window
    let is_content_visible = is_visible(&app_handle, AppWindow::Content);
    if let Some(focused_window_id) = widget.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows
            .iter_mut()
            .find(|window| window.id == focused_window_id)
        {
            if is_content_visible {
                editor_window.update_content_window_state(&ContentWindowState::Active);
            } else {
                editor_window.update_content_window_state(&ContentWindowState::Inactive);
            }
        }
    }

    close_window(&widget.app_handle, AppWindow::Widget)
}
