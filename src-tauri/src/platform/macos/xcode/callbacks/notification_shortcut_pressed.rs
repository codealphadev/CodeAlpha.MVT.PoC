use accessibility::{AXUIElement, AXUIElementAttributes, Error};

use crate::{
    platform::macos::{
        get_focused_window,
        models::editor::{EditorShortcutPressedMessage, ModifierKey},
        xcode::XCodeObserverState,
        AXEventXcode,
    },
    utils::assert_or_error_trace,
};

pub fn notification_key_press_save(
    uielement: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    assert_or_error_trace(
        uielement.role()?.to_string() == "AXMenuItem",
        &format!(
            "notification_key_press_save() called with AXUIElement of type {}; expected AXMenuItem",
            uielement.role()?.to_string()
        ),
    );

    let cmd_title = uielement.title()?;
    let cmd_modifier_option = uielement.menu_item_cmd_modifier()?.to_i64();
    let cmd_char = uielement.menu_item_cmd_char()?;

    if let Some(cmd_modifier_int) = cmd_modifier_option {
        let mut cmd_modifier = ModifierKey::Unknown;
        match cmd_modifier_int {
            0 => {
                cmd_modifier = ModifierKey::Cmd;
            }
            1 => {
                cmd_modifier = ModifierKey::ShiftCmd;
            }
            2 => {
                cmd_modifier = ModifierKey::OptionCmd;
            }
            4 => {
                cmd_modifier = ModifierKey::CtrlCmd;
            }
            _ => {}
        }
        let window_uid = get_focused_window().map_err(|_| Error::NotFound)?;

        AXEventXcode::EditorShortcutPressed(EditorShortcutPressedMessage {
            window_uid,
            modifier: cmd_modifier,
            key: cmd_char.to_string(),
            menu_item_title: cmd_title.to_string(),
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);
    }

    Ok(())
}
