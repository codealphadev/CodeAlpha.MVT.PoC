use crate::{
    ax_events::{models::XCodeFocusElement, Event},
    window_controls::{close_window, open_window, AppWindow},
};

use super::window_positioning;
use std::sync::{Arc, Mutex};
use tauri::{EventHandler, Manager, PhysicalSize};

static EDITOR_NAME: &str = "Xcode";

// Features:
// [x] Which windows to load at startup
// [ ] Listening to movement of widget window --> updating position accordingly
//   [ ] Move logic from TS into Rust
//   [ ] Detect "GhostClicks in Rust instead of in Frontend"
// [x] Listening to XCode Twin messages and update window visibility accordingly

pub struct WindowStateMachine {
    tauri_app_handle: tauri::AppHandle,
    listener_app_focus_status: Option<EventHandler>,
    listener_xcode_focus_status_change: Option<EventHandler>,

    last_known_editor_position: Arc<Mutex<Option<tauri::PhysicalPosition<i32>>>>,
    last_known_editor_size: Arc<Mutex<Option<tauri::PhysicalSize<i32>>>>,
    last_repositioned_widget_position: Arc<Mutex<Option<tauri::PhysicalPosition<i32>>>>,
    last_focused_app_pid: Arc<Mutex<Option<u32>>>,
}

impl WindowStateMachine {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            tauri_app_handle: app_handle.clone(),
            listener_app_focus_status: None,
            listener_xcode_focus_status_change: None,
            last_known_editor_position: Arc::new(Mutex::new(None)),
            last_known_editor_size: Arc::new(Mutex::new(None)),
            last_repositioned_widget_position: Arc::new(Mutex::new(None)),
            last_focused_app_pid: Arc::new(Mutex::new(None)),
        }
    }

    pub fn setup(&mut self) {
        // Registering listener for a change in Global App Focus
        // ==================================
        // 1. Copy variable Arcs to be moved into closure (can't move sef into closure)
        let tauri_app_handle_copy = self.tauri_app_handle.clone();
        let last_focused_app_pid_copy = self.last_focused_app_pid.clone();

        // 2. Create listener
        // Store listener id to be able to safely remove it later
        let listener_app_focus_status =
            self.tauri_app_handle
                .listen_global("StateEvent-AppFocusState", move |msg| {
                    // Create Listener-Closure and register it at the App(Handle)
                    // =========================================================
                    // Only execute if incoming msg contains a payload
                    if let Some(msg_s) = msg.payload() {
                        let parsed_msg: Event = serde_json::from_str(&msg_s).unwrap();
                        // Parse msg as Event of correct type
                        if let Event::AppFocusState(payload) = parsed_msg {
                            let _app_name = &tauri_app_handle_copy.package_info().name;

                            // For now, on this listener, only hide widget if neither widget or editor are in focus
                            if ![EDITOR_NAME].contains(&&payload.current_app.name.as_str())
                                && payload.current_app.pid != std::process::id()
                            {
                                close_window(tauri_app_handle_copy.clone(), AppWindow::Widget);
                            }

                            // Update last focused app
                            let mut last_focused_app_pid =
                                last_focused_app_pid_copy.lock().unwrap();
                            *last_focused_app_pid = Some(payload.current_app.pid);
                        }
                    }
                });
        self.listener_app_focus_status = Some(listener_app_focus_status);

        // Registering listener for Editor Focus
        // ==================================
        // 1. Copy variable Arcs to be moved into closure (can't move sef into closure)
        let last_known_editor_position_copy = self.last_known_editor_position.clone();
        let last_known_editor_size_copy = self.last_known_editor_size.clone();
        let last_repositioned_widget_position_copy = self.last_repositioned_widget_position.clone();
        let tauri_app_handle_copy = self.tauri_app_handle.clone();
        let last_focused_app_pid_copy = self.last_focused_app_pid.clone();

        // 2. Create listener
        // Store listener id to be able to safely remove it later
        let listener_xcode_focus_status_change =
            self.tauri_app_handle
                .listen_global("StateEvent-XCodeFocusStatusChange", move |msg| {
                    // Create Listener-Closure and register it at the App(Handle)
                    // =========================================================
                    // Only execute if incoming msg contains a payload
                    if let Some(msg_s) = msg.payload() {
                        // Parse msg as Event of correct type
                        let parsed_msg: Event = serde_json::from_str(&msg_s).unwrap();
                        if let Event::XCodeFocusStatusChange(payload) = parsed_msg {
                            // For now, on this listener, only react to focus changes on the Editor
                            if let XCodeFocusElement::Editor = payload.focus_element_change {
                                // Show widget if ...
                                // 1. Last focused app before receiving this msg was NOT this app
                                // 2. Editor was focused; restore content window visibility.
                                let last_focused_app_pid =
                                    last_focused_app_pid_copy.lock().unwrap();

                                if payload.is_in_focus
                                    && *last_focused_app_pid != Some(std::process::id())
                                {
                                    let editor_position = tauri::PhysicalPosition {
                                        x: payload.ui_element_x as i32,
                                        y: payload.ui_element_y as i32,
                                    };

                                    let editor_size = tauri::PhysicalSize {
                                        width: payload.ui_element_w as i32,
                                        height: payload.ui_element_h as i32,
                                    };

                                    // 1. Reposition
                                    Self::smartly_position_widget(
                                        tauri_app_handle_copy.clone(),
                                        editor_position,
                                        last_known_editor_position_copy.clone(),
                                        editor_size,
                                        last_known_editor_size_copy.clone(),
                                        last_repositioned_widget_position_copy.clone(),
                                    );

                                    // 2. Show Widget
                                    open_window(tauri_app_handle_copy.clone(), AppWindow::Widget);
                                }
                            } else {
                                close_window(tauri_app_handle_copy.clone(), AppWindow::Widget);
                            }
                        }
                    }
                });
        self.listener_xcode_focus_status_change = Some(listener_xcode_focus_status_change);
    }

    fn smartly_position_widget(
        app_handle: tauri::AppHandle,
        curr_editor_pos: tauri::PhysicalPosition<i32>,
        prev_editor_pos: Arc<Mutex<Option<tauri::PhysicalPosition<i32>>>>,
        curr_editor_size: tauri::PhysicalSize<i32>,
        prev_editor_size: Arc<Mutex<Option<tauri::PhysicalSize<i32>>>>,
        prev_widget_pos: Arc<Mutex<Option<tauri::PhysicalPosition<i32>>>>,
    ) {
        // Lock & Unpack Arcs
        let mut prev_editor_pos_mutex = prev_editor_pos.lock().unwrap();
        let mut prev_editor_size_mutex = prev_editor_size.lock().unwrap();
        let mut prev_widget_pos_mutex = prev_widget_pos.lock().unwrap();

        let widget_window = app_handle.get_window(&AppWindow::Widget.to_string());
        if let Some(widget_window) = widget_window {
            if let (Some(prev_editor_pos), Some(prev_editor_size), Some(prev_widget_pos)) = (
                *prev_editor_pos_mutex,
                *prev_editor_size_mutex,
                *prev_widget_pos_mutex,
            ) {
                // Get current widget position
                let curr_widget_pos = (widget_window.outer_position()).unwrap();

                // Calculate differences to previous state
                let editor_move_dist = calc_move_distance(prev_editor_pos, curr_editor_pos);
                let editor_resize_dist = calc_resize(prev_editor_size, curr_editor_size);

                // Update states:
                *prev_editor_pos_mutex = Some(curr_editor_pos.clone());
                *prev_editor_size_mutex = Some(curr_editor_size.clone());

                // Determine, why we are here!
                if editor_move_dist.width != 0 || editor_move_dist.height != 0 {
                    // Case 1: Window was moved
                    let updated_widget_pos = tauri::PhysicalPosition {
                        x: prev_widget_pos.x + editor_move_dist.width,
                        y: prev_widget_pos.y + editor_move_dist.height,
                    };
                    *prev_widget_pos_mutex = Some(updated_widget_pos.clone());
                    let _ =
                        widget_window.set_position(tauri::Position::Physical(updated_widget_pos));
                } else if editor_resize_dist.width != 0 || editor_resize_dist.width != 0 {
                    // Case 2: Window was resized
                    // For now: do nothing
                    *prev_widget_pos_mutex = Some(curr_widget_pos.clone());
                } else {
                    // Case 3: Window was just refocused
                    // Determine if widget is within editor extends
                    if is_pos_within_editor_window(
                        curr_widget_pos,
                        curr_editor_pos,
                        curr_editor_size,
                    ) {
                        // Rule: If curr_widget_pos is inside editor dimensions, update prev_widget_pos
                        *prev_widget_pos_mutex = Some(curr_widget_pos.clone());
                    } else {
                        // Rule: If curr_widget_pos is outside editor dimensions, restore prev_widget_pos
                        let _ =
                            widget_window.set_position(tauri::Position::Physical(prev_widget_pos));
                    }
                }

                let _ = window_positioning::cmd_update_content_position(app_handle.clone());
            } else {
                // Case: No previous positions known, just position at bottom right cornor (startup)
                Self::position_widget_bottom_right(
                    app_handle.clone(),
                    curr_editor_pos,
                    curr_editor_size,
                );

                // Update states
                *prev_editor_pos_mutex = Some(curr_editor_pos.clone());
                *prev_editor_size_mutex = Some(curr_editor_size.clone());
                let curr_widget_pos = (widget_window.outer_position()).unwrap();
                *prev_widget_pos_mutex = Some(curr_widget_pos.clone());
            }
        }
    }

    fn position_widget_bottom_right(
        app_handle: tauri::AppHandle,
        editor_position: tauri::PhysicalPosition<i32>,
        editor_size: tauri::PhysicalSize<i32>,
    ) {
        let widget_window = app_handle.get_window(&AppWindow::Widget.to_string());
        if widget_window.is_some() {
            let widget_window = widget_window.unwrap();

            let widget_size = PhysicalSize::<u32> {
                width: widget_window.outer_size().unwrap().width,
                height: widget_window.outer_size().unwrap().height,
            };

            // Get Screen Size
            let screen = widget_window.current_monitor().unwrap().unwrap();
            let screen_size = screen.size();

            // Calculate new position
            // Rule 1: place it in the bottom right corner of the editor
            // Rule 2: move it inwards x/y by the size of the widget
            let mut new_widget_pos = tauri::PhysicalPosition {
                x: editor_position.x + editor_size.width
                    - 5 * Self::helper_u32_to_i32(widget_size.width),
                y: editor_position.y + editor_size.height
                    - 2 * Self::helper_u32_to_i32(widget_size.width),
            };

            // Prevent Widget from going off-screen
            if new_widget_pos.x < 0 {
                new_widget_pos.x = 0;
            }
            if new_widget_pos.y < 0 {
                new_widget_pos.y = 0;
            }
            if new_widget_pos.x + Self::helper_u32_to_i32(widget_size.width)
                > Self::helper_u32_to_i32(screen_size.width)
            {
                new_widget_pos.x = Self::helper_u32_to_i32(screen_size.width)
                    - Self::helper_u32_to_i32(widget_size.width);
            }
            if new_widget_pos.y + Self::helper_u32_to_i32(widget_size.height)
                > Self::helper_u32_to_i32(screen_size.height)
            {
                new_widget_pos.y = Self::helper_u32_to_i32(screen_size.height)
                    - Self::helper_u32_to_i32(widget_size.height);
            }

            // Set new position
            let _ = widget_window.set_position(tauri::Position::Physical(new_widget_pos));

            // Update Content Window Position
            let _ = window_positioning::cmd_update_content_position(app_handle.clone());
        }
    }

    // Just a helper to convert between i32 and u32; it is safe, because on screens u32 can be expected to be small enough.
    fn helper_u32_to_i32(v: u32) -> i32 {
        i32::try_from(v).ok().unwrap()
    }
}

impl Drop for WindowStateMachine {
    fn drop(&mut self) {
        // Unregister listener for AppFocusStatus
        if let Some(listener_app_focus_status) = self.listener_app_focus_status.take() {
            self.tauri_app_handle.unlisten(listener_app_focus_status);
        }

        // Unregister listener for XCodeFocusStatusChange
        if let Some(listener_xcode_focus_status_change) =
            self.listener_xcode_focus_status_change.take()
        {
            self.tauri_app_handle
                .unlisten(listener_xcode_focus_status_change);
        }
    }
}

fn is_pos_within_editor_window(
    pos: tauri::PhysicalPosition<i32>,
    editor_pos: tauri::PhysicalPosition<i32>,
    editor_size: tauri::PhysicalSize<i32>,
) -> bool {
    let mut x_within = false;
    let mut y_within = false;

    if pos.x > editor_pos.x && pos.x < (editor_pos.x + editor_size.width) {
        x_within = true;
    }

    if pos.y > editor_pos.y && pos.y < (editor_pos.y + editor_size.height) {
        y_within = true;
    }

    return x_within && y_within;
}

fn calc_move_distance(
    prev_pos: tauri::PhysicalPosition<i32>,
    new_pos: tauri::PhysicalPosition<i32>,
) -> tauri::PhysicalSize<i32> {
    tauri::PhysicalSize {
        width: new_pos.x - prev_pos.x,
        height: new_pos.y - prev_pos.y,
    }
}

fn calc_resize(
    prev_size: tauri::PhysicalSize<i32>,
    new_size: tauri::PhysicalSize<i32>,
) -> tauri::PhysicalSize<i32> {
    tauri::PhysicalSize {
        width: new_size.width - prev_size.width,
        height: new_size.height - prev_size.height,
    }
}

#[cfg(test)]
mod tests {

    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};

    mod test_position_relative_to_editor_window {
        use crate::utils::window_state_machine::is_pos_within_editor_window;

        #[test]
        fn is_outside_left() {
            assert_eq!(
                verify_is_position_within_editor_window(tauri::PhysicalPosition { x: 10, y: 70 }),
                false
            );
        }

        #[test]
        fn is_inside_edge() {
            // if position is "on the edge" of the editor, count it as "in"
            assert_eq!(
                verify_is_position_within_editor_window(tauri::PhysicalPosition { x: 50, y: 50 }),
                false
            );
        }

        #[test]
        fn is_inside() {
            assert_eq!(
                verify_is_position_within_editor_window(tauri::PhysicalPosition { x: 500, y: 80 }),
                true
            );
        }

        fn verify_is_position_within_editor_window(position: tauri::PhysicalPosition<i32>) -> bool {
            let editor_position = tauri::PhysicalPosition { x: 50, y: 50 };
            let editor_size = tauri::PhysicalSize {
                width: 1000,
                height: 500,
            };

            is_pos_within_editor_window(position, editor_position, editor_size)
        }
    }
}
