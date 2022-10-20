use parking_lot::Mutex;
use std::sync::Arc;

use crate::{
    core_engine::{core_engine::CoreEngineError, features::CoreEngineTrigger, CoreEngine},
    platform::macos::{
        get_selected_text_range, get_textarea_content, get_textarea_file_path,
        models::editor::{EditorUIElementFocusedMessage, FocusedUIElement},
        GetVia,
    },
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

    let content_str = get_textarea_content(&GetVia::Pid(pid))?;
    let file_path = get_textarea_file_path(&GetVia::Pid(pid)).ok();
    let selected_text_range = get_selected_text_range(&GetVia::Pid(pid))?;

    _ = core_engine.run_features(
        window_uid,
        CoreEngineTrigger::OnTextSelectionChange,
        None,
        None,
        Some(&selected_text_range),
    );

    _ = core_engine.run_features(
        window_uid,
        CoreEngineTrigger::OnTextContentChange,
        Some(&content_str),
        file_path.as_ref(),
        None,
    );

    Ok(())
}
