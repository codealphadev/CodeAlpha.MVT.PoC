use accessibility::{AXUIElement, AXUIElementAttributes};
use accessibility_sys::pid_t;
use core_foundation::string::CFString;
use tauri::{ClipboardManager, Error};

use crate::{
    ax_interaction::{get_selected_text_range, set_selected_text_range, GetVia},
    core_engine::TextRange,
};

/// A function that pastes the clipboard content into the textarea of the Xcode editor.
/// If some text is provided as part of the arguments, then this is pasted into the textarea while
/// the old clipboard content is preserved and restored after the paste operation.
///
/// It takes a `pid_t` (process ID) and a `String` (the text to paste), and then it pastes the text into
/// the code text area of the editor window of the application with the given process ID
///
/// Arguments:
///
/// * `app_handle`: Tauri App Handle.
/// * `pid`: The process ID of the application containing the code text area.
/// * `text`: optional - The text to paste into the text area. If provided, the clipboard will be preserved.
///
/// Might returns a Tauri Error.
pub fn paste_clipboard_text(
    app_handle: &tauri::AppHandle,
    pid: pid_t,
    text: Option<&String>,
    add_linebreak_with_enter: bool,
) -> Result<(), Error> {
    let mut clipboard = app_handle.clipboard_manager();
    let preserve_old_clipboard = clipboard.read_text()?;

    if let Some(text) = text {
        let text_to_paste = if add_linebreak_with_enter {
            format!("{}\n", text)
        } else {
            text.clone()
        };
        clipboard.write_text(text_to_paste)?;
    }

    let app_ui_element = AXUIElement::application(pid);
    let _ = perform_paste_xcode_ax_api(&app_ui_element);

    if let Some(text) = preserve_old_clipboard {
        tauri::async_runtime::spawn(async move {
            use std::time;
            let delay = time::Duration::from_millis(150);
            tokio::time::sleep(delay).await;

            let _ = clipboard.write_text(text);
        });
    }

    Ok(())
}

/// A function that pastes the clipboard content at a specific text selection in the textarea of the Xcode editor.
/// It replaces the text in the given range with the given text, and then, optionally, restores the cursor position
/// to where it was before the replacement.
///
/// If some text is provided as part of the arguments, then this is pasted into the textarea while
/// the old clipboard content is preserved and restored after the paste operation.
///
/// Arguments:
///
/// * `app_handle`: the tauri app handle
/// * `pid`: The process ID of the application containing the code text area.
/// * `range`: The range of text to replace.
/// * `text`: The text to replace the range with. If this is None, the text from the clipboard will be
/// used.
/// * `restore_cursor`: if true, the cursor position will be restored after the text is replaced
pub fn replace_range_with_clipboard_text(
    app_handle: &tauri::AppHandle,
    pid: pid_t,
    range: &TextRange,
    text: Option<&String>,
    restore_cursor: bool,
) {
    let mut preserved_cursor_position: Option<TextRange> = None;

    if restore_cursor {
        if let Ok(range) = get_selected_text_range(&GetVia::Pid(pid)) {
            preserved_cursor_position = Some(range);
        } else {
            // Case: an error was thrown while attempting to obtain the cursor position
            return;
        };
    }

    if set_selected_text_range(&range, &GetVia::Pid(pid)).is_ok() {
        let _ = paste_clipboard_text(&app_handle, pid, text, true);
    }

    if restore_cursor {
        if let Some(range) = preserved_cursor_position {
            let _ = set_selected_text_range(&range, &GetVia::Pid(pid));
        }
    }
}

/// "Find the 'Paste' menu item in the 'Edit' menu, and press it."
///
/// The function starts by asserting that the element passed in is an application. Then it loops through
/// the application's children, looking for the menu bar. Once it finds the menu bar, it loops through
/// the menu bar's children, looking for the 'Edit' menu. Once it finds the 'Edit' menu, it loops
/// through the 'Edit' menu's children, looking for the 'Paste' menu item. Once it finds the 'Paste'
/// menu item, it loops through the 'Paste' menu item's children, looking for the 'Paste' menu. Once it
/// finds the 'Paste' menu, it loops through the 'Paste' menu's children, looking for the 'Paste' menu
/// item. Once it finds the 'Paste' menu item, it saves it in a variable. Then, if the variable is not
/// `None`, it 'presses' it.
///
/// Arguments:
///
/// * `element`: The AXUIElement that represents the application.
///
/// Returns:
///
/// A Result<(), accessibility::Error>
fn perform_paste_xcode_ax_api(element: &AXUIElement) -> Result<(), accessibility::Error> {
    assert!(element.role()? == "AXApplication");

    let mut paste_ui_element: Option<AXUIElement> = None;
    let app_children = element.children()?;
    for app_child in app_children.iter() {
        if app_child.role()? == "AXMenuBar" {
            let menu_bar_children = app_child.children()?;
            for menu_bar_child in menu_bar_children.iter() {
                if menu_bar_child.title()? == "Edit" {
                    let edit_children = menu_bar_child.children()?;
                    for edit_child in edit_children.iter() {
                        if edit_child.role()? == "AXMenu" {
                            let edit_menu_children = edit_child.children()?;
                            for edit_menu_child in edit_menu_children.iter() {
                                if edit_menu_child.title()? == "Paste" {
                                    let paste_children = edit_menu_child.children()?;
                                    for paste_child in paste_children.iter() {
                                        if paste_child.role()? == "AXMenu" {
                                            let paste_menu_children = paste_child.children()?;
                                            for paste_menu_child in paste_menu_children.iter() {
                                                if paste_menu_child.title()? == "Paste" {
                                                    paste_ui_element =
                                                        Some(paste_menu_child.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(paste_ui_element) = paste_ui_element {
        paste_ui_element.perform_action(&CFString::from_static_string("AXPress"))?;
    }

    Ok(())
}
