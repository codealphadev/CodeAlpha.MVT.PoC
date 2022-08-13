use core::panic;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::{
    ax_interaction::{
        app::observer_app::register_observer_app, derive_xcode_textarea_dimensions,
        get_textarea_uielement, models::app::ContentWindowState,
    },
    window_controls::{
        actions::{close_window, create_window, open_window, set_position},
        code_overlay::{hide_code_overlay, show_code_overlay},
        config::AppWindow,
        content_window,
        editor_window::EditorWindow,
    },
};

use super::{
    dimension_calculations::prevent_widget_position_off_screen,
    listener_user_interaction::register_listener_user_interactions,
    listeners::{register_listener_app, register_listener_xcode},
    prevent_misalignement_of_content_and_widget,
};

pub static HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS: u64 = 200;
pub static SUPPORTED_EDITORS: &[&str] = &["Xcode"];

#[derive(Clone, Debug)]
pub struct WidgetWindow {
    pub app_handle: tauri::AppHandle,

    /// List of open editor windows. List is managed by WindowStateManager.
    pub editor_windows: Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>>,

    /// Identitfier of the currently focused editor window. Is None until the first window was focused.
    pub currently_focused_editor_window: Option<uuid::Uuid>,

    /// Each qualifying incoming event updates the instant until when the widget should be hidden.
    pub temporary_hide_until_instant: Instant,

    /// If an event requires the widget to be temporarily hidden, it triggers a routine that monitors when
    /// the widget should be shown again. In case another event occurs and this variable is set to true,
    /// only 'temporary_hide_until_instant' will be updated.
    pub temporary_hide_check_active: bool,

    /// Boolean saying if the currently focused application is our app.
    pub is_app_focused: bool,

    /// Boolean saying if the currently focused application is an editor window.
    pub is_editor_focused: bool,

    /// Identitfier of the currently focused app window. Is None until the first window was focused.
    pub currently_focused_app_window: Option<AppWindow>,

    /// Boolean to indicate if the code_overlay_visible is visible or hidden.
    pub code_overlay_visible: Option<bool>,
}

impl WidgetWindow {
    pub fn new(
        app_handle: &tauri::AppHandle,
        editor_windows: &Arc<Mutex<HashMap<uuid::Uuid, EditorWindow>>>,
    ) -> Self {
        // Create App Windows
        if create_window(&app_handle, AppWindow::Widget).is_err() {
            panic!("Could not create Widget Window");
        }
        if create_window(&app_handle, AppWindow::Content).is_err() {
            panic!("Could not create Content Window");
        }
        if create_window(&app_handle, AppWindow::CodeOverlay).is_err() {
            panic!("Could not create CodeOverlay Window");
        }

        // Register Observer for Widget AX Events
        if register_observer_app(&app_handle).is_err() {
            panic!("Could not register observer app");
        }

        Self {
            app_handle: app_handle.clone(),
            editor_windows: editor_windows.clone(),
            temporary_hide_until_instant: Instant::now(),
            temporary_hide_check_active: false,
            currently_focused_editor_window: None,
            is_app_focused: false,
            is_editor_focused: false,
            currently_focused_app_window: None,
            code_overlay_visible: None,
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

        register_listener_user_interactions(app_handle, &widget_window);
    }

    pub fn temporary_hide_check_routine(
        app_handle: &tauri::AppHandle,
        widget_props: &Arc<Mutex<WidgetWindow>>,
        hide_widget: bool,
        hide_codeoverlay: bool,
    ) {
        {
            let widget = &mut *(match widget_props.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            });

            // Update the Instant time stamp when the widget should be shown again
            widget.temporary_hide_until_instant =
                Instant::now() + Duration::from_millis(HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS);

            // Check if another instance of this routine is already running
            if widget.temporary_hide_check_active {
                return;
            } else {
                widget.temporary_hide_check_active = true;
            }

            if hide_widget {
                close_window(app_handle, AppWindow::Widget);
            }
            if hide_codeoverlay {
                close_window(app_handle, AppWindow::CodeOverlay);
            }
        }

        // Start temporary hide check routine
        let widget_props_move_copy = widget_props.clone();
        let app_handle_move_copy = app_handle.clone();

        tauri::async_runtime::spawn(async move {
            loop {
                // !!!!! Sleep first to not block the locked Mutexes afterwards !!!!!
                // ==================================================================
                thread::sleep(std::time::Duration::from_millis(25));
                // ==================================================================

                let widget_window = &mut *(match widget_props_move_copy.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                });

                if widget_window.temporary_hide_until_instant < Instant::now() {
                    let mut editor_windows = &mut *(match widget_window.editor_windows.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    });

                    // Debounce in case scrolling was still happening while switching
                    // between windows using e.g. CMD+Tab
                    if widget_window.is_editor_focused {
                        Self::show_widget_routine(
                            &app_handle_move_copy,
                            &widget_window,
                            &mut editor_windows,
                        );
                    }

                    // Indicate that the routine finished
                    widget_window.temporary_hide_check_active = false;
                    break;
                }
            }
        });
    }

    pub fn show_widget_routine(
        app_handle: &tauri::AppHandle,
        widget: &WidgetWindow,
        editor_windows: &mut HashMap<uuid::Uuid, EditorWindow>,
    ) -> Option<bool> {
        let editor_window = editor_windows.get_mut(&widget.currently_focused_editor_window?)?;

        // Redundant check if we really have the correct textarea dimensions
        // We should implement a periodic check for this in the future; the editor window
        // should query AX api every second to update the textarea dimensions
        if let Ok((position, size)) =
            derive_xcode_textarea_dimensions(&get_textarea_uielement(editor_window.pid)?)
        {
            editor_window.update_window_dimensions(
                editor_window.window_position(),
                editor_window.window_size(),
                Some(position),
                Some(size),
            );
        }

        // Recover ContentWindowState for this editor window and open CodeOverlay window
        match editor_window.content_window_state {
            ContentWindowState::Active => {
                let _ = content_window::open(
                    &app_handle,
                    &editor_window.get_monitor_for_editor_window(app_handle)?,
                );
            }
            ContentWindowState::Inactive => {
                let _ = content_window::hide(&app_handle);
            }
        }

        if let Some(code_overlay_visible) = widget.code_overlay_visible {
            if code_overlay_visible {
                let _ = show_code_overlay(
                    app_handle,
                    editor_window.textarea_position(true),
                    editor_window.textarea_size(),
                );
            } else {
                let _ = hide_code_overlay(app_handle);
            }
        } else {
            let _ = show_code_overlay(
                app_handle,
                editor_window.textarea_position(true),
                editor_window.textarea_size(),
            );
        }

        // Check if the widget position should be updated before showing it
        let mut widget_position = editor_window.widget_position(true)?;

        let editor_window_monitor = editor_window.get_monitor_for_editor_window(app_handle)?;
        prevent_widget_position_off_screen(&editor_window_monitor, &mut widget_position);

        // If content window was open before, also check that it would not go offscreen
        if editor_window.content_window_state == ContentWindowState::Active {
            prevent_misalignement_of_content_and_widget(
                &app_handle,
                &editor_window_monitor,
                &mut widget_position,
            );
        }

        let _ = set_position(&widget.app_handle, AppWindow::Widget, &widget_position);

        open_window(&widget.app_handle, AppWindow::Widget);

        Some(true)
    }

    pub fn hide_widget_routine(app_handle: &tauri::AppHandle) {
        close_window(app_handle, AppWindow::Widget);
        close_window(app_handle, AppWindow::CodeOverlay);
    }

    pub fn update_code_overlay_visible(&mut self, code_overlay_visible: bool) {
        self.code_overlay_visible = Some(code_overlay_visible);
    }
}
