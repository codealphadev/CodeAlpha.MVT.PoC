use std::time::Instant;

use crate::{
    ax_interaction::models::editor::FocusedUIElement, window_controls::editor_window::EditorWindow,
};

use super::WidgetWindow;

pub enum ShowHide {
    Show,
    Hide,
    Continue,
}

pub fn validate_decision_tree_show_hide_widget(
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
