use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    EventField,
};
use rdev::{simulate, EventType};

use crate::{
    app_handle,
    ax_interaction::{models::input_device::MouseMovedMessage, EventInputDevice},
    utils::geometry::LogicalPosition,
};

use super::{
    fast_track_code_editor_scroll::fast_track_handle_text_editor_mousewheel_scroll,
    generate_axui_element_hash,
    internal::get_focused_uielement,
    is_focused_uielement_xcode_editor_textarea,
    models::input_device::{ClickType, MouseButton, MouseClickMessage},
    setup::{get_registered_ax_observer, ObserverType},
    GetVia, XcodeError,
};

pub fn subscribe_mouse_events() {
    match CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![
            CGEventType::MouseMoved,
            CGEventType::LeftMouseDown,
            CGEventType::LeftMouseUp,
            CGEventType::ScrollWheel,
        ],
        |_a, event_type, event| {
            match event_type {
                CGEventType::Null => {}
                CGEventType::LeftMouseDown => notification_mouse_click_event(event_type, event),
                CGEventType::LeftMouseUp => notification_mouse_click_event(event_type, event),
                CGEventType::RightMouseDown => notification_mouse_click_event(event_type, event),
                CGEventType::RightMouseUp => notification_mouse_click_event(event_type, event),
                CGEventType::MouseMoved => notification_mouse_move_event(event),
                CGEventType::LeftMouseDragged => notification_mouse_click_event(event_type, event),
                CGEventType::RightMouseDragged => notification_mouse_click_event(event_type, event),
                CGEventType::KeyDown => {}
                CGEventType::KeyUp => {}
                CGEventType::FlagsChanged => {}
                CGEventType::ScrollWheel => notification_mousewheel_event_wrapper(),
                CGEventType::TabletPointer => {}
                CGEventType::TabletProximity => {}
                CGEventType::OtherMouseDown => notification_mouse_click_event(event_type, event),
                CGEventType::OtherMouseUp => notification_mouse_click_event(event_type, event),
                CGEventType::OtherMouseDragged => notification_mouse_click_event(event_type, event),
                CGEventType::TapDisabledByTimeout => {}
                CGEventType::TapDisabledByUserInput => {}
            }
            None
        },
    ) {
        Ok(tap) => unsafe {
            let loop_source = tap
                .mach_port
                .create_runloop_source(0)
                .expect("Registering mouse event subscriber failed");
            let runloop = CFRunLoop::get_current();
            runloop.add_source(&loop_source, kCFRunLoopDefaultMode);
            tap.enable();
            CFRunLoop::run_current();
        },
        Err(_) => (println!("Error subscribing to mouse events")),
    }
}

fn notification_mousewheel_event_wrapper() {
    notification_mousewheel_event();
}
fn notification_mousewheel_event() -> Option<()> {
    // Check if we need to send a notification that a valid text editor field was scrolled.
    if let Some((editor_pid, _)) = get_registered_ax_observer(ObserverType::XCode) {
        // 1. Is focused app a valid editor?
        let focused_ui_element = get_focused_uielement(&GetVia::Current).ok()?;
        if editor_pid == focused_ui_element.pid().ok()? {
            // 2. Is focused app a valid text editor field?
            if is_focused_uielement_xcode_editor_textarea().ok()? {
                let text_editor_hash = generate_axui_element_hash(&focused_ui_element);
                fast_track_handle_text_editor_mousewheel_scroll(text_editor_hash);
            }
        }
    }

    Some(())
}

fn notification_mouse_move_event(event: &CGEvent) {
    let delta_x = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X);
    let delta_y = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y);
    let event_location = event.location();
    let cursor_position = LogicalPosition {
        x: event_location.x.round() as f64,
        y: event_location.y.round() as f64,
    };

    EventInputDevice::MouseMoved(MouseMovedMessage {
        move_delta_x: delta_x,
        move_delta_y: delta_y,
        cursor_position,
    })
    .publish_to_tauri(&app_handle());
}

fn notification_mouse_click_event(event_type: CGEventType, event: &CGEvent) {
    let event_location = event.location();
    let cursor_position = LogicalPosition {
        x: event_location.x.round() as f64,
        y: event_location.y.round() as f64,
    };

    let mut button: Option<MouseButton> = None;
    let mut click_type: Option<ClickType> = None;

    match event_type {
        CGEventType::LeftMouseDown => {
            button = Some(MouseButton::Left);
            click_type = Some(ClickType::Down);
        }
        CGEventType::LeftMouseUp => {
            button = Some(MouseButton::Left);
            click_type = Some(ClickType::Up);
        }
        CGEventType::LeftMouseDragged => {
            button = Some(MouseButton::Left);
            click_type = Some(ClickType::Drag);
        }
        CGEventType::RightMouseDown => {
            button = Some(MouseButton::Right);
            click_type = Some(ClickType::Down);
        }
        CGEventType::RightMouseUp => {
            button = Some(MouseButton::Right);
            click_type = Some(ClickType::Up);
        }
        CGEventType::RightMouseDragged => {
            button = Some(MouseButton::Right);
            click_type = Some(ClickType::Drag);
        }
        CGEventType::OtherMouseDown => {
            button = Some(MouseButton::Other);
            click_type = Some(ClickType::Down);
        }
        CGEventType::OtherMouseUp => {
            button = Some(MouseButton::Other);
            click_type = Some(ClickType::Up);
        }
        CGEventType::OtherMouseDragged => {
            button = Some(MouseButton::Other);
            click_type = Some(ClickType::Drag);
        }
        _ => {}
    }

    if let (Some(button), Some(click_type)) = (button, click_type) {
        EventInputDevice::MouseClick(MouseClickMessage {
            button,
            click_type,
            cursor_position,
        })
        .publish_to_tauri(&app_handle());
    }
}

pub fn send_event_mouse_wheel(delta: tauri::LogicalSize<f64>) -> Result<bool, XcodeError> {
    if is_focused_uielement_xcode_editor_textarea()? {
        let event_type = EventType::Wheel {
            delta_x: delta.width as i64,
            delta_y: delta.height as i64,
        };

        match simulate(&event_type) {
            Ok(()) => {
                return Ok(true);
            }
            Err(_) => {
                println!("We could not send {:?}", event_type);
            }
        }
    }
    Ok(false)
}
