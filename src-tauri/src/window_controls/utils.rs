#![allow(dead_code)]

use crate::ax_events_deprecated::Event;

use super::config::AppWindow;

// Method calculates if a given position is inside the given editor rectangle
pub fn is_pos_within_editor_window(
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

// Calculates the distance between two points. This is used to determine how
// far the editor window was moved.
pub fn calc_move_distance(
    prev_pos: tauri::PhysicalPosition<i32>,
    new_pos: tauri::PhysicalPosition<i32>,
) -> tauri::PhysicalSize<i32> {
    tauri::PhysicalSize {
        width: new_pos.x - prev_pos.x,
        height: new_pos.y - prev_pos.y,
    }
}

// Calculate how much the dimensions of a window changed during resizing.
pub fn calc_resize(
    prev_size: tauri::PhysicalSize<i32>,
    new_size: tauri::PhysicalSize<i32>,
) -> tauri::PhysicalSize<i32> {
    tauri::PhysicalSize {
        width: new_size.width - prev_size.width,
        height: new_size.height - prev_size.height,
    }
}

// Just a helper to convert between i32 and u32; it is safe, because on screens u32 can be expected to be small enough.
pub fn helper_u32_to_i32(v: u32) -> i32 {
    i32::try_from(v).ok().unwrap()
}

// This helper function cuts out the boilerplate code for matching a tauri event to an ax event type.
// The result Event can the be matched using this snipped:
// match parse_into_ax_event_type(tauri_event_msg) {
//     Event::AppFocusState(val) => { /* CONTROL LOGIC */   }
//     _ => { /* do nothing */ },
// }
pub fn parse_into_ax_event_type(event: tauri::Event) -> Event {
    if let Some(msg_s) = event.payload() {
        if let Ok(parsed_msg) = serde_json::from_str::<Event>(&msg_s) {
            match parsed_msg {
                Event::AppFocusState(payload) => Event::AppFocusState(payload),
                Event::XCodeFocusStatusChange(payload) => Event::XCodeFocusStatusChange(payload),
                _ => Event::None,
            }
        } else {
            Event::None
        }
    } else {
        Event::None
    }
}

pub fn get_window_label(window_type: AppWindow) -> String {
    window_type.to_string()
}

#[cfg(test)]
mod tests {

    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};

    mod test_position_relative_to_editor_window {
        use crate::window_controls::utils::is_pos_within_editor_window;

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
