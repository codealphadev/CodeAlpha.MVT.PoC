use accessibility::{AXUIElement, AXUIElementAttributes, Error};

use crate::ax_interaction::{
    currently_focused_window, generate_axui_element_hash,
    models::editor::{EditorShortcutPressedMessage, ModifierKey},
    AXEventXcode, XCodeObserverState,
};

pub fn notification_key_press_save(
    uielement: &AXUIElement,
    xcode_observer_state: &mut XCodeObserverState,
) -> Result<(), Error> {
    assert_eq!(uielement.role()?, "AXMenuItem");

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
        let ui_elem_hash = generate_axui_element_hash(&currently_focused_window()?);

        AXEventXcode::EditorShortcutPressed(EditorShortcutPressedMessage {
            modifier: cmd_modifier,
            key: cmd_char.to_string(),
            menu_item_title: cmd_title.to_string(),
            ui_elem_hash,
        })
        .publish_to_tauri(&xcode_observer_state.app_handle);
    }

    Ok(())
}
