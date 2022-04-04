#![allow(non_upper_case_globals)]

use std::thread;
use std::{ffi::c_void, mem};

use accessibility::{AXAttribute, AXObserver, AXUIElement, Error};
use accessibility_sys::{
    kAXFocusedUIElementChangedNotification, kAXWindowDeminiaturizedNotification,
    kAXWindowMiniaturizedNotification, kAXWindowMovedNotification, kAXWindowResizedNotification,
    AXObserverRef, AXUIElementRef,
};
use cocoa::appkit::CGPoint;
use core_foundation::runloop::CFRunLoop;
use core_foundation::{
    base::TCFType,
    string::{CFString, CFStringRef},
};
use core_graphics_types::geometry::CGSize;

use crate::axevents::models::{XCodeFocusElement, XCodeFocusStatusChange};
use crate::axevents::{
    models::{AppFocusState, AppInfo},
    Event,
};

pub struct TauriStateHandle {
    pub handle: tauri::AppHandle,
}

pub fn observer_focused_app(tauri_handle: TauriStateHandle) {
    let tauri_handle_move_copy = TauriStateHandle {
        handle: tauri_handle.handle.clone(),
    };
    thread::spawn(move || {
        let mut focused_app: Option<AXUIElement> = None;
        let mut xcode_app: Option<AXUIElement> = None;

        loop {
            // Register XCode observer
            // =======================
            // Happens only once when xcode is launched; is retriggered if xcode is restarted
            let app = currently_focused_app();
            if let Ok(currently_focused_app) = app {
                let registration_required_res =
                    is_xcode_observer_registration_required(currently_focused_app, &mut xcode_app);

                if let Ok(registration_required) = registration_required_res {
                    if registration_required {
                        if let Some(ref xcode_app) = xcode_app {
                            let _ = register_xcode_observer(xcode_app, &tauri_handle_move_copy);
                        }
                    }
                }
            }

            // Monitor which app is currently in focus
            // =======================================
            let app = currently_focused_app();
            if let Ok(currently_focused_app) = app {
                if let Some(ref previously_focused_app) = focused_app {
                    if *previously_focused_app != currently_focused_app {
                        callback_app_focus_state(
                            previously_focused_app,
                            &currently_focused_app,
                            &tauri_handle_move_copy,
                        );

                        focused_app = Some(currently_focused_app);
                    }
                } else {
                    focused_app = Some(currently_focused_app);
                }
            }

            thread::sleep(std::time::Duration::from_millis(150));
        }
    });
}

fn is_xcode_observer_registration_required(
    focused_app: AXUIElement,
    known_xcode_app: &mut Option<AXUIElement>,
) -> Result<bool, Error> {
    let focused_app_title = focused_app.attribute(&AXAttribute::title())?;

    // Check if focused app is XCode, skip if not
    if focused_app_title != "Xcode" {
        return Ok(false);
    }

    if let Some(xcode_app) = known_xcode_app {
        let xcode_app_identifier = xcode_app.attribute(&AXAttribute::identifier())?;
        let focused_app_identifier = focused_app.attribute(&AXAttribute::identifier())?;

        // Case: XCode has a new AXUIElement, telling us it was restarted
        if xcode_app_identifier != focused_app_identifier {
            *known_xcode_app = Some(focused_app);
            return Ok(true);
        }
    } else {
        // Case: XCode was never started while the program was running; it's UI element is not known yet
        *known_xcode_app = Some(focused_app);
        return Ok(true);
    }

    Ok(false)
}

fn register_xcode_observer(
    xcode_ui_element: &AXUIElement,
    tauri_apphandle: &TauriStateHandle,
) -> Result<(), Error> {
    assert_eq!(xcode_ui_element.attribute(&AXAttribute::title())?, "Xcode");

    let pid = xcode_ui_element.pid().unwrap();
    let tauri_handle_move_copy = tauri_apphandle.handle.clone();
    thread::spawn(move || {
        // 1. Create AXObserver
        let xcode_observer = AXObserver::new(pid, callback_xcode_events);
        let ui_element = AXUIElement::application(pid);

        if let Ok(mut xcode_observer) = xcode_observer {
            xcode_observer.start();

            // Add notification for "AXFocusedUIElementChanged"
            let _ = xcode_observer.add_notification(
                kAXFocusedUIElementChangedNotification,
                &ui_element,
                TauriStateHandle {
                    handle: tauri_handle_move_copy.clone(),
                },
            );

            // Add notification for "WindowMoved"
            let _ = xcode_observer.add_notification(
                kAXWindowMovedNotification,
                &ui_element,
                TauriStateHandle {
                    handle: tauri_handle_move_copy.clone(),
                },
            );

            // Add notification for "WindowResized"
            let _ = xcode_observer.add_notification(
                kAXWindowResizedNotification,
                &ui_element,
                TauriStateHandle {
                    handle: tauri_handle_move_copy.clone(),
                },
            );

            // Add notification for "WindowMiniaturized"
            let _ = xcode_observer.add_notification(
                kAXWindowMiniaturizedNotification,
                &ui_element,
                TauriStateHandle {
                    handle: tauri_handle_move_copy.clone(),
                },
            );

            // Add notification for "WindowDeminiaturized"
            let _ = xcode_observer.add_notification(
                kAXWindowDeminiaturizedNotification,
                &ui_element,
                TauriStateHandle {
                    handle: tauri_handle_move_copy.clone(),
                },
            );

            CFRunLoop::run_current();
        }
    });

    Ok(())
}

unsafe extern "C" fn callback_xcode_events(
    observer: AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    context: *mut c_void,
) {
    let _observer: AXObserver = TCFType::wrap_under_get_rule(observer);
    let element: AXUIElement = TCFType::wrap_under_get_rule(element);
    let notification = CFString::wrap_under_get_rule(notification);
    let context: *mut TauriStateHandle = mem::transmute(context);

    match notification.to_string().as_str() {
        kAXFocusedUIElementChangedNotification => {
            let _ = publish_xcode_focus_change(&element, &*context);
        }
        _other => {
            // println!("{:?}", other)
        }
    }
}

fn publish_xcode_focus_change(
    focused_element: &AXUIElement,
    tauri_state: &TauriStateHandle,
) -> Result<(), Error> {
    let role = focused_element.attribute(&AXAttribute::role())?;

    if role.to_string().as_str() == "AXTextArea" {
        // Get the frame of the parent UI element --> TextAreas don't contain the actual coordinates of the window
        let parent_ui_element = focused_element.attribute(&AXAttribute::parent())?;

        let size_ax_val = parent_ui_element.attribute(&AXAttribute::size())?;
        let pos_ax_val = parent_ui_element.attribute(&AXAttribute::position())?;

        let size = size_ax_val.get_value::<CGSize>()?;
        let origin = pos_ax_val.get_value::<CGPoint>()?;

        let focus_change = XCodeFocusStatusChange {
            focus_element_change: XCodeFocusElement::Editor,
            is_in_focus: true,
            ui_element_x: origin.x,
            ui_element_y: origin.y,
            ui_element_w: size.width,
            ui_element_h: size.height,
        };

        let event = Event::XCodeFocusStatusChange(focus_change);
        event.publish_to_tauri(tauri_state.handle.clone());
    } else {
        let focus_change = XCodeFocusStatusChange {
            focus_element_change: XCodeFocusElement::App,
            is_in_focus: true,
            ui_element_x: 0.0,
            ui_element_y: 0.0,
            ui_element_w: 0.0,
            ui_element_h: 0.0,
        };

        let event = Event::XCodeFocusStatusChange(focus_change);
        event.publish_to_tauri(tauri_state.handle.clone());
    }

    Ok(())
}

fn callback_app_focus_state(
    previous_app: &AXUIElement,
    current_app: &AXUIElement,
    tauri_state: &TauriStateHandle,
) {
    assert_ne!(previous_app, current_app);

    let current_app_title = current_app.attribute(&AXAttribute::title());
    let previous_app_title = previous_app.attribute(&AXAttribute::title());

    if let (Ok(current_app_title), Ok(previous_app_title)) = (current_app_title, previous_app_title)
    {
        let focus_state = AppFocusState {
            previous_app: AppInfo {
                bundle_id: "".to_string(),
                name: previous_app_title.to_string(),
                pid: 0,
                is_finished_launching: true,
            },
            current_app: AppInfo {
                bundle_id: "".to_string(),
                name: current_app_title.to_string(),
                pid: 0,
                is_finished_launching: true,
            },
        };

        let event = Event::AppFocusState(focus_state);
        event.publish_to_tauri(tauri_state.handle.clone());
    }
}

pub fn currently_focused_app() -> Result<AXUIElement, Error> {
    let system_wide_element = AXUIElement::system_wide();
    let focused_ui_element = system_wide_element.attribute(&AXAttribute::focused_uielement())?;
    let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;
    focused_window.attribute(&AXAttribute::parent())
}
