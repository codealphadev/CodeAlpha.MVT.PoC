use std::thread;

use accessibility_sys::{
    kAXApplicationActivatedNotification, kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification, kAXApplicationShownNotification, kAXErrorNoValue,
    kAXFocusedUIElementChangedNotification, kAXMainWindowChangedNotification,
    kAXValueChangedNotification, kAXWindowCreatedNotification, kAXWindowDeminiaturizedNotification,
    kAXWindowMiniaturizedNotification, kAXWindowMovedNotification, kAXWindowResizedNotification,
};

use accessibility::{AXAttribute, AXObserver, AXUIElement, Error};
use core_foundation::runloop::CFRunLoop;

use super::callback_replit_notifications;
use crate::ax_interaction::{
    models::editor::EditorAppClosedMessage, AXEventReplit, ReplitObserverState,
};

static SUPPORTED_BROWSERS: &[&str] = &["com.google.Chrome", "com.apple.Safari"];
static SUPPORTED_FILETYPES: &[&str] = &["swift", "ts", "js", "txt"];

static OBSERVER_REGISTRATION_DELAY_IN_MILLIS: u64 = 2000;
static OBSERVER_NOTIFICATIONS: &'static [&'static str] = &[
    kAXApplicationActivatedNotification,
    kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification,
    kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification,
    kAXMainWindowChangedNotification,
    kAXValueChangedNotification,
    kAXWindowCreatedNotification,
    kAXWindowDeminiaturizedNotification,
    kAXWindowMiniaturizedNotification,
    kAXWindowMovedNotification,
    kAXWindowResizedNotification,
];

pub fn register_observer_replit(
    known_replit_editors: &mut Vec<(String, AXUIElement)>,
    app_handle: &tauri::AppHandle,
) {
    for browser in SUPPORTED_BROWSERS {
        if let Ok(new_editors) = new_replit_editors(known_replit_editors, browser) {
            for replit_ui_element in new_editors {
                let _ = create_observer_and_add_notifications(&replit_ui_element, &app_handle);
            }
        }
    }

    // Determine which of the supported browsers are not currently open
    let mut closed_browsers: Vec<String> = Vec::new();
    for browser in SUPPORTED_BROWSERS {
        if let Err(_) = AXUIElement::application_with_bundle(browser) {
            closed_browsers.push(browser.to_string());
        }
    }

    for browser in closed_browsers {
        // Remove all editors from browsers which are closed
        known_replit_editors.retain(|editor| {
            if editor.0 == browser {
                AXEventReplit::EditorAppClosed(EditorAppClosedMessage {
                    editor_name: "Replit".to_string(),
                    pid: editor.1.pid().unwrap().try_into().unwrap(),
                    browser: Some(editor.0.clone()),
                })
                .publish_to_tauri(&app_handle);
                return false;
            }
            return true;
        });
    }

    // println!("{:?}", known_replit_editors);
}

fn new_replit_editors(
    known_replit_editors: &mut Vec<(String, AXUIElement)>,
    browser_name: &str,
) -> Result<Vec<AXUIElement>, Error> {
    let browser_ui_element = AXUIElement::application_with_bundle(browser_name)?;
    let browser_windows = browser_ui_element.attribute(&AXAttribute::children())?;

    let mut new_replit_editors: Vec<AXUIElement> = Vec::new();
    for window in &browser_windows {
        if let Ok(window_title) = window.attribute(&AXAttribute::title()) {
            // Check if the currently activated tab in the editor is a Replit tab
            if !window_title.to_string().contains("Replit") {
                continue;
            }

            // check if the Replit tab shows an editor window - indicated by a file extension in the tab's title
            let mut replit_window_has_editor_open = false;
            for file_suffix in SUPPORTED_FILETYPES {
                let match_str = format!(".{}", file_suffix);

                if window_title.to_string().contains(&match_str) {
                    replit_window_has_editor_open = true;
                }
            }
            if !replit_window_has_editor_open {
                continue;
            }

            // Check if the Replit window is already known
            let valid_replit_editor_window_already_known =
                (&known_replit_editors).iter().find(|elem| {
                    // if the Replit window is already known, we don't need to create a new observer
                    *window == elem.1
                });

            if valid_replit_editor_window_already_known.is_some() {
                continue;
            } else {
            }

            // If we reach this point, we have found a new valid Replit editor window
            known_replit_editors.push((browser_name.to_string(), window.clone()));
            new_replit_editors.push(window.clone());
        }
    }

    Ok(new_replit_editors)
}

// This function is called to create a new observer and add the notifications to it.
// The list of notifications is managed at the top of the file in a static variable.
fn create_observer_and_add_notifications(
    replit_ui_element: &AXUIElement,
    app_handle: &tauri::AppHandle,
) -> Result<(), Error> {
    let pid = replit_ui_element.pid().unwrap();
    let window_title = replit_ui_element
        .attribute(&AXAttribute::title())?
        .to_string();
    let app_handle_move_copy = app_handle.clone();

    thread::spawn(move || {
        // 0. Delay observer registration on macOS, because there is a good chance no
        // notifications will be received despite seemingly successful observer registration
        thread::sleep(std::time::Duration::from_millis(
            OBSERVER_REGISTRATION_DELAY_IN_MILLIS,
        ));

        // 1. Create AXObserver
        let replit_observer = AXObserver::new(pid, callback_replit_notifications);

        // 2. Find the Replit window's UIElement --> we can't move AXUIElement to different thread
        //    because Send is not implemented.
        if let Ok(_) = ui_element_window_by_title(pid, &window_title) {
            println!("Found Replit window: {}", window_title);
            let ui_element = AXUIElement::application(pid); // <- TODO
            if let Ok(mut replit_observer) = replit_observer {
                // 2. Start AXObserver before adding notifications
                replit_observer.start();

                // 3. Add notifications
                for notification in OBSERVER_NOTIFICATIONS.iter() {
                    let _ = replit_observer.add_notification(
                        notification,
                        &ui_element,
                        ReplitObserverState {
                            app_handle: app_handle_move_copy.clone(),
                            window_list: Vec::new(),
                        },
                    );
                }

                // 4. Kick of RunLoop on this thread
                CFRunLoop::run_current();
            }
        }
    });

    Ok(())
}

fn ui_element_window_by_title(pid: i32, title: &str) -> Result<AXUIElement, Error> {
    let app_ui_element = AXUIElement::application(pid);
    let app_windows = app_ui_element.attribute(&AXAttribute::children())?;

    for window in &app_windows {
        if let Ok(window_title) = window.attribute(&AXAttribute::title()) {
            if window_title.to_string() == title {
                return Ok(window.clone());
            }
        }
    }

    Err(Error::Ax(kAXErrorNoValue))
}
