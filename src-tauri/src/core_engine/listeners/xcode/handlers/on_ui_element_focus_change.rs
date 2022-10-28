use parking_lot::Mutex;
use std::sync::Arc;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::models::editor::{EditorUIElementFocusedMessage, FocusedUIElement},
};

pub fn on_editor_focused_uielement_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    uielement_focus_changed_msg: &EditorUIElementFocusedMessage,
) -> Result<(), CoreEngineError> {
    if uielement_focus_changed_msg.focused_ui_element != FocusedUIElement::Textarea {
        return Ok(());
    }

    let pid = uielement_focus_changed_msg
        .pid
        .ok_or(CoreEngineError::MissingContext(
            "PID missing from focus_changed_msg".to_string(),
        ))?;

    let window_uid =
        uielement_focus_changed_msg
            .window_uid
            .ok_or(CoreEngineError::MissingContext(
                "WindowUID missing from focus_changed_msg".to_string(),
            ))?;

    let core_engine = &mut core_engine_arc.lock();
    core_engine.add_code_document(pid, window_uid);

    _ = core_engine.handle_trigger(window_uid, CoreEngineTrigger::OnTextSelectionChange);
    _ = core_engine.handle_trigger(window_uid, CoreEngineTrigger::OnTextContentChange);

    Ok(())
}
