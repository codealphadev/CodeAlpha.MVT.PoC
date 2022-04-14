use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use tauri::{Error, Manager};

use crate::{
    ax_interaction::{
        app::observer_app::register_observer_app,
        models::{
            app::AppWindowMovedMessage,
            editor::{
                EditorAppActivatedMessage, EditorAppClosedMessage, EditorAppDeactivatedMessage,
                EditorUIElementFocusedMessage, EditorWindowMovedMessage,
                EditorWindowResizedMessage, FocusedUIElement,
            },
        },
        AXEventApp, AXEventXcode, AX_EVENT_APP_CHANNEL, AX_EVENT_XCODE_CHANNEL,
    },
    window_controls::{close_window, open_window},
};

use super::{create_window, editor_window::EditorWindow, AppWindow};

static HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS: u64 = 200;
static HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS: u64 = 50;
static XCODE_EDITOR_NAME: &str = "Xcode";

#[allow(dead_code)]
#[derive(Clone)]
pub struct WidgetWindow {
    app_handle: tauri::AppHandle,

    /// List of open editor windows. List is managed by WindowStateManager.
    editor_windows: Arc<Mutex<Vec<EditorWindow>>>,

    /// Identitfier of the currently focused editor window. Is None until the first window was focused.
    currently_focused_editor_window: Option<uuid::Uuid>,

    /// Each qualifying incoming event updates the instant until when the widget should be hidden.
    hide_until_instant: Instant,

    /// In case the focus switches from our app to an editor or vice versa it is possible, that there is
    /// a state where seemingly neither is in focus, only because the new "AXActivation" event from the
    /// newly focused entity hasn't arrived yet / wasn't processed yet.
    delay_hide_until_instant: Instant,

    /// Boolean saying if the currently focused application is Xcode.
    is_xcode_focused: bool,

    /// Boolean saying if the currently focused application is our app.
    is_app_focused: bool,

    /// Identitfier of the currently focused app window. Is None until the first window was focused.
    currently_focused_app_window: Option<AppWindow>,
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
        control_widget_visibility(&app_handle, &widget_props);
    }
}

fn register_listener_xcode(app_handle: &tauri::AppHandle, widget_props: &Arc<Mutex<WidgetWindow>>) {
    let widget_props_move_copy = (widget_props).clone();
    app_handle.listen_global(AX_EVENT_XCODE_CHANNEL, move |msg| {
        let mut widget_props_locked = widget_props_move_copy.lock().unwrap();

        let axevent_xcode: AXEventXcode = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match axevent_xcode {
            AXEventXcode::EditorWindowResized(msg) => {
                on_resize_editor_window(&mut *widget_props_locked, &msg);
            }
            AXEventXcode::EditorWindowMoved(msg) => {
                on_move_editor_window(&mut *widget_props_locked, &msg);
            }
            AXEventXcode::EditorUIElementFocused(msg) => {
                on_editor_ui_element_focus_change(&mut *widget_props_locked, &msg);
            }
            AXEventXcode::EditorAppActivated(msg) => {
                on_editor_app_activated(&mut *widget_props_locked, &msg)
            }
            AXEventXcode::EditorAppDeactivated(msg) => {
                on_editor_app_deactivated(&mut *widget_props_locked, &msg)
            }
            AXEventXcode::EditorAppClosed(msg) => {
                on_editor_app_closed(&mut *widget_props_locked, &msg)
            }
            _ => {}
        }
    });
}

fn register_listener_app(app_handle: &tauri::AppHandle, widget_props: &Arc<Mutex<WidgetWindow>>) {
    let widget_props_move_copy = (widget_props).clone();
    app_handle.listen_global(AX_EVENT_APP_CHANNEL, move |msg| {
        let mut widget_props_locked = widget_props_move_copy.lock().unwrap();

        let axevent_app: AXEventApp = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match &axevent_app {
            AXEventApp::AppWindowFocused(msg) => {
                (*widget_props_locked).currently_focused_app_window = Some(msg.window);
            }
            AXEventApp::AppWindowMoved(msg) => {
                on_move_app_window(&mut *widget_props_locked, &msg);
            }
            AXEventApp::AppUIElementFocused(_) => {
                // TODO: Do nothing
            }
            AXEventApp::AppActivated(_) => {
                (*widget_props_locked).is_app_focused = true;
            }
            AXEventApp::AppDeactivated(_) => {
                (*widget_props_locked).is_app_focused = false;

                // Reset hide timer after which the widget should be displayed again
                (*widget_props_locked).delay_hide_until_instant =
                    Instant::now() + Duration::from_millis(HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS);
            }
            AXEventApp::None => {}
        }
    });
}

fn on_move_app_window(widget_props: &mut WidgetWindow, move_msg: &AppWindowMovedMessage) {
    let editor_windows = &mut *(widget_props.editor_windows.lock().unwrap());
    if let Some(focused_editor_window_id) = widget_props.currently_focused_editor_window {
        if let Some(editor_window) = editor_windows
            .iter_mut()
            .find(|window| window.id == focused_editor_window_id)
        {
            if move_msg.window == AppWindow::Widget {
                editor_window.update_widget_position(move_msg.window_position);
            }
        }
    }
}

fn on_resize_editor_window(
    widget_props: &mut WidgetWindow,
    resize_msg: &EditorWindowResizedMessage,
) {
    let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

    for window in &mut *editor_list_locked {
        if window.id == resize_msg.id {
            window.update_window_dimensions(
                resize_msg.window_position,
                resize_msg.window_size,
                resize_msg.textarea_position,
                resize_msg.textarea_size,
            );

            // Reset hide timer after which the widget should be displayed again
            widget_props.hide_until_instant =
                Instant::now() + Duration::from_millis(HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS);

            break;
        }
    }
}

fn on_move_editor_window(widget_props: &mut WidgetWindow, moved_msg: &EditorWindowMovedMessage) {
    let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

    for window in &mut *editor_list_locked {
        if window.id == moved_msg.id {
            window.update_window_dimensions(
                moved_msg.window_position,
                moved_msg.window_size,
                None,
                None,
            );

            // Reset hide timer after which the widget should be displayed again
            widget_props.hide_until_instant =
                Instant::now() + Duration::from_millis(HIDE_DELAY_ON_MOVE_OR_RESIZE_IN_MILLIS);

            break;
        }
    }
}

/// Update EditorWindow to which of it's ui elements is currently in focus. Furthermore, also update
/// which of all open editor windows is currently in focus.
fn on_editor_ui_element_focus_change(
    widget_props: &mut WidgetWindow,
    focus_msg: &EditorUIElementFocusedMessage,
) {
    let mut editor_list_locked = widget_props.editor_windows.lock().unwrap();

    for window in &mut *editor_list_locked {
        if window.id == focus_msg.window_id {
            window.update_focused_ui_element(
                &focus_msg.focused_ui_element,
                focus_msg.textarea_position,
                focus_msg.textarea_size,
            );

            // Set which editor window is currently focused
            widget_props.currently_focused_editor_window = Some(window.id);

            break;
        }
    }
}

fn on_editor_app_deactivated(
    widget_props: &mut WidgetWindow,
    deactivated_msg: &EditorAppDeactivatedMessage,
) {
    if deactivated_msg.editor_name == XCODE_EDITOR_NAME {
        widget_props.is_xcode_focused = false;
    }

    widget_props.delay_hide_until_instant =
        Instant::now() + Duration::from_millis(HIDE_DELAY_ON_DEACTIVATE_IN_MILLIS);
}

fn on_editor_app_closed(widget_props: &mut WidgetWindow, closed_msg: &EditorAppClosedMessage) {
    if closed_msg.editor_name == XCODE_EDITOR_NAME {
        widget_props.is_xcode_focused = false;
    }
}

fn on_editor_app_activated(
    widget_props: &mut WidgetWindow,
    activated_msg: &EditorAppActivatedMessage,
) {
    if activated_msg.editor_name == XCODE_EDITOR_NAME {
        widget_props.is_xcode_focused = true;
    }
}

fn control_widget_visibility(
    _app_handle: &tauri::AppHandle,
    widget_props: &Arc<Mutex<WidgetWindow>>,
) {
    let widget_props_move_copy = widget_props.clone();
    thread::spawn(move || {
        loop {
            // !!!!! Sleep first to not block the locked Mutexes afterwards !!!!!
            // ==================================================================
            thread::sleep(std::time::Duration::from_millis(25));
            // ==================================================================

            let widget = &*(widget_props_move_copy.lock().unwrap());
            let editor_windows = &*(widget.editor_windows.lock().unwrap());

            // Control widget visibility
            match validate_decision_tree_show_hide_widget(widget, editor_windows) {
                ShowHide::Show => open_window(&widget.app_handle, AppWindow::Widget),
                ShowHide::Hide => close_window(&widget.app_handle, AppWindow::Widget),
                ShowHide::Continue => {}
            }
        }
    });
}

enum ShowHide {
    Show,
    Hide,
    Continue,
}

fn validate_decision_tree_show_hide_widget(
    widget: &WidgetWindow,
    editor_windows: &Vec<EditorWindow>,
) -> ShowHide {
    match check_if_xcode_is_running(editor_windows) {
        ShowHide::Show => ShowHide::Show,
        ShowHide::Hide => ShowHide::Hide,
        ShowHide::Continue => check_if_either_widget_or_editor_is_focused(widget, editor_windows),
    }
}

fn check_if_xcode_is_running(editor_windows: &Vec<EditorWindow>) -> ShowHide {
    // Hide if no editor_window exists
    if editor_windows.len() == 0 {
        ShowHide::Hide
    } else {
        ShowHide::Continue
    }
}

fn check_if_either_widget_or_editor_is_focused(
    widget: &WidgetWindow,
    editor_windows: &Vec<EditorWindow>,
) -> ShowHide {
    // Case: Hide if neither xcode nor app is focused
    if widget.is_xcode_focused || widget.is_app_focused {
        // Trying something

        match check_if_an_editor_window_is_focused(
            &widget.currently_focused_editor_window,
            editor_windows,
        ) {
            ShowHide::Hide => ShowHide::Hide,
            ShowHide::Show => {
                check_if_widget_should_be_temporarily_hidden(&widget.hide_until_instant)
            }
            ShowHide::Continue => ShowHide::Continue,
        }
    } else {
        check_if_hiding_delay_elapsed(widget.delay_hide_until_instant)
    }
}

fn check_if_hiding_delay_elapsed(delay_hide_until_instant: Instant) -> ShowHide {
    if delay_hide_until_instant < Instant::now() {
        ShowHide::Hide
    } else {
        ShowHide::Continue
    }
}

fn check_if_widget_should_be_temporarily_hidden(hide_until_instant: &Instant) -> ShowHide {
    // Case: Check if widget is supposed to be temporarily hidden due
    //       to the editor window being either resized or moved.
    if *hide_until_instant > Instant::now() {
        ShowHide::Hide
    } else {
        ShowHide::Show
    }
}

fn check_if_an_editor_window_is_focused(
    focused_window_id: &Option<uuid::Uuid>,
    editor_windows: &Vec<EditorWindow>,
) -> ShowHide {
    if let Some(focused_window_id) = focused_window_id {
        return check_if_focused_window_is_still_available(focused_window_id, editor_windows);
    } else {
        return ShowHide::Hide;
    }
}

fn check_if_focused_window_is_still_available(
    focused_window_id: &uuid::Uuid,
    editor_windows: &Vec<EditorWindow>,
) -> ShowHide {
    let editor_window = editor_windows
        .iter()
        .find(|window| window.id == *focused_window_id);
    if let Some(window) = editor_window {
        return check_if_focused_ui_element_is_textarea(window.focused_ui_element.as_ref());
    } else {
        return ShowHide::Hide;
    }
}

fn check_if_focused_ui_element_is_textarea(ui_element: Option<&FocusedUIElement>) -> ShowHide {
    if let Some(ui_element) = ui_element {
        match ui_element {
            FocusedUIElement::Textarea => ShowHide::Show,
            FocusedUIElement::Other => ShowHide::Hide,
        }
    } else {
        ShowHide::Hide
    }
}
