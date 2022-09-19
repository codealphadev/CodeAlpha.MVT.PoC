use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    EventField,
};
use lazy_static::lazy_static;
use objc::{msg_send, runtime::Class, sel, sel_impl};
use parking_lot::Mutex;
use rdev::{simulate, EventType};
use std::time::Duration;
use throttle::Throttle;

use crate::{
    app_handle,
    platform::macos::{models::input_device::MouseMovedMessage, EventInputDevice},
    utils::geometry::{LogicalPosition, LogicalSize},
};

use super::{
    get_xcode_editor_textarea,
    models::input_device::{ClickType, MouseButton, MouseClickMessage},
    EventViewport, GetVia, XcodeError,
};

lazy_static! {
    static ref CODE_DOC_ORIGIN: Mutex<Option<LogicalPosition>> = Mutex::new(None);
}

lazy_static! {
    static ref SCROLL_THROTTLE: Mutex<Throttle> =
        Mutex::new(Throttle::new(Duration::from_millis(8), 1));
}

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
    notification_xcode_textarea_scrolled();
}

fn did_code_doc_origin_change(code_doc_origin: &LogicalPosition) -> bool {
    let mut code_doc_origin_lock = CODE_DOC_ORIGIN.lock();
    let code_doc_origin_changed = match *code_doc_origin_lock {
        Some(ref origin) => origin != code_doc_origin,
        None => true,
    };
    *code_doc_origin_lock = Some(*code_doc_origin);
    code_doc_origin_changed
}

fn notification_xcode_textarea_scrolled() -> Option<()> {
    SCROLL_THROTTLE.try_lock()?.accept().ok()?;

    let xcode_editor_textarea = get_xcode_editor_textarea().ok()?;

    if xcode_editor_textarea.is_some() {
        let event = EventViewport::new_xcode_viewport_update_minimal(&GetVia::Current).ok()?;

        let did_update = match &event {
            EventViewport::XcodeViewportUpdate(msg) => {
                did_code_doc_origin_change(&msg.code_document_frame_properties.dimensions.origin)
            }
        };

        if did_update {
            event.publish_to_tauri(&app_handle());
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

    let (button, click_type) = map_mouse_buttons(event_type);

    if let (Some(button), Some(click_type)) = (button, click_type) {
        EventInputDevice::MouseClick(MouseClickMessage {
            button,
            click_type,
            cursor_position,
        })
        .publish_to_tauri(&app_handle());
    }
}

pub fn send_event_mouse_wheel(delta: LogicalSize) -> Result<bool, XcodeError> {
    if get_xcode_editor_textarea()?.is_some() {
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

pub fn pressed_mouse_buttons() -> Option<MouseClickMessage> {
    let ns_event = Class::get("NSEvent").unwrap();

    let mouse_button_id: CGEventType = unsafe { msg_send![ns_event, pressedMouseButtons] };
    let mouse_position = enigo::Enigo::mouse_location();

    let (button, click_type) = map_mouse_buttons(mouse_button_id);

    if let (Some(button), Some(click_type)) = (button, click_type) {
        return Some(MouseClickMessage {
            button,
            click_type,
            cursor_position: LogicalPosition {
                x: mouse_position.0 as f64,
                y: mouse_position.1 as f64,
            },
        });
    }

    None
}

fn map_mouse_buttons(mouse_button: CGEventType) -> (Option<MouseButton>, Option<ClickType>) {
    let mut button: Option<MouseButton> = None;
    let mut click_type: Option<ClickType> = None;

    match mouse_button {
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

    (button, click_type)
}
