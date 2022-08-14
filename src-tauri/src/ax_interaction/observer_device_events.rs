use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    EventField,
};

use crate::{
    app_handle,
    ax_interaction::{models::input_device::MouseMovedMessage, EventInputDevice},
    utils::geometry::LogicalPosition,
};

use super::models::input_device::{ClickType, MouseButton, MouseClickMessage};

pub fn subscribe_mouse_events() {
    match CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![
            CGEventType::MouseMoved,
            CGEventType::LeftMouseDown,
            CGEventType::LeftMouseUp,
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
                CGEventType::ScrollWheel => {}
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
        Err(_) => (assert!(false)),
    }
}

fn _notification_mousewheel_event(event: &CGEvent) {
    let delta_x = event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2);
    let delta_y = event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1);
    let event_location = event.location();
    let cursor_position = LogicalPosition {
        x: event_location.x.round() as f64,
        y: event_location.y.round() as f64,
    };

    println!(
        "Scroll x: {:?}, scroll y: {:?}, pos x: {:?}, pos y: {:?}",
        delta_x, delta_y, cursor_position.x, cursor_position.y
    );
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
