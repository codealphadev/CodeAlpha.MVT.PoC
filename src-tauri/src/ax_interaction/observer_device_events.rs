use std::time::{Instant, SystemTime};

use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    EventField,
};
use parking_lot::Mutex;

use crate::{
    app_handle,
    ax_interaction::{
        focused_uielement_of_app, get_textarea_frame, models::input_device::MouseMovedMessage,
        EventInputDevice,
    },
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        models::editor_window::CodeOverlayDimensionsUpdateMessage, EventWindowControls,
    },
};

use lazy_static::lazy_static;
lazy_static! {
    static ref CORRECTION_EVENT_PUBLISHING_TIME: Mutex<Option<Instant>> = Mutex::new(None);
}

use super::{
    currently_focused_app, derive_xcode_textarea_dimensions,
    models::input_device::{ClickType, MouseButton, MouseClickMessage},
    setup::{get_registered_ax_observer, ObserverType},
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
                CGEventType::ScrollWheel => notification_mousewheel_event(),
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

fn was_editor_window_scrolled(editor_pid: i32) -> bool {
    if let Ok(focused_ui_element) = currently_focused_app() {
        if let Ok(focused_pid) = focused_ui_element.pid() {
            if focused_pid == editor_pid {
                return true;
            }
        }
    }
    return false;
}

fn notification_mousewheel_event() {
    if let Some((editor_pid, _)) = get_registered_ax_observer(ObserverType::XCode) {
        if !was_editor_window_scrolled(editor_pid) {
            return;
        }
        // TODO: fence that the mouse position is in a valid rectangle
        println!("Mouse wheel event");

        if let Some(mut locked_publishing_time) = CORRECTION_EVENT_PUBLISHING_TIME.try_lock() {
            execute_event(editor_pid);
            if locked_publishing_time.is_none() {
                // Case: This is the first scrolling event. It will be responsible for the final execution, after the last scrolling event has happened.
                // So we wait until the last scrolling event was more than 50 millis ago.
                tauri::async_runtime::spawn(async move {
                    loop {
                        let correction_event_timestamp;
                        {
                            correction_event_timestamp =
                                CORRECTION_EVENT_PUBLISHING_TIME.lock().clone();
                        }

                        if let Some(hide_until) = correction_event_timestamp {
                            // Is zero when hide_until is older than Instant::now()
                            let duration = hide_until.duration_since(Instant::now());

                            if duration.is_zero() {
                                // Scrolling has finished. Publish correction event.
                                *CORRECTION_EVENT_PUBLISHING_TIME.lock() = None;
                                // Sometimes, XCode handles the scroll event quickly, but sometimes it takes longer.
                                // Send multiple correction events at different delays for optimal handling.
                                execute_event(editor_pid);
                                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                                execute_event(editor_pid);
                                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                execute_event(editor_pid);

                                break;
                            }
                            tokio::time::sleep(duration).await;
                        } else {
                            tokio::time::sleep(std::time::Duration::from_millis(3)).await;
                        }
                    }
                });
            }

            // Refresh the time to publish final, correction event, since we are stil observing scrolling.
            (*locked_publishing_time)
                .replace(Instant::now() + std::time::Duration::from_millis(50));
        }
    }
}

fn execute_event(pid: i32) {
    tauri::async_runtime::spawn(async move {
        println!("{:?}", SystemTime::now());

        let textarea_uielement = focused_uielement_of_app(pid).ok().unwrap(); // TODO: handle error and streamline
        let code_doc_rect = get_textarea_frame(&textarea_uielement).ok().unwrap();
        let view_doc_rect = derive_xcode_textarea_dimensions(&textarea_uielement).unwrap();

        EventWindowControls::CodeOverlayDimensionsUpdate(CodeOverlayDimensionsUpdateMessage {
            code_viewport_rect: LogicalFrame {
                origin: LogicalPosition::from_tauri_LogicalPosition(&view_doc_rect.0),
                size: LogicalSize::from_tauri_LogicalSize(&view_doc_rect.1),
            },
            code_document_rect: code_doc_rect,
        })
        .publish_to_tauri(&app_handle());
    });
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
