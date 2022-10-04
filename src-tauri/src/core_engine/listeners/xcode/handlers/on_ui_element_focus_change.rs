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

use super::check_if_code_doc_needs_to_be_created;

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

    let mut text_changed;
    {
        let code_documents = &mut core_engine.code_documents().lock();

        check_if_code_doc_needs_to_be_created(code_documents, pid, window_uid);

        let code_doc = code_documents
            .get_mut(&window_uid)
            .ok_or(CoreEngineError::CodeDocNotFound(window_uid))?;

        // Update code document properties
        let content_str = get_textarea_content(&GetVia::Pid(pid))?;
        let file_path = get_textarea_file_path(&GetVia::Pid(pid)).ok();
        text_changed = code_doc.update_doc_properties(&content_str, &file_path);

        let selected_text_range = get_selected_text_range(&GetVia::Pid(pid))?;
        text_changed = text_changed || code_doc.set_selected_text_range(&selected_text_range, true);
    }

    core_engine.run_features(window_uid, &CoreEngineTrigger::OnTextSelectionChange)?;
    if text_changed {
        core_engine.run_features(window_uid, &CoreEngineTrigger::OnTextContentChange)?;
    }
    Ok(())
}
