use parking_lot::Mutex;
use std::sync::Arc;

use crate::{
    ax_interaction::{
        get_selected_text_range, get_textarea_content, get_textarea_file_path, get_viewport_frame,
        models::editor::{EditorUIElementFocusedMessage, FocusedUIElement},
        GetVia,
    },
    core_engine::CoreEngine,
};

use super::check_if_code_doc_needs_to_be_created;

pub fn on_editor_focused_uielement_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    uielement_focus_changed_msg: &EditorUIElementFocusedMessage,
) -> Option<()> {
    if uielement_focus_changed_msg.focused_ui_element != FocusedUIElement::Textarea {
        return None;
    }

    let core_engine = &mut core_engine_arc.lock();

    let core_engine_active_status = core_engine.engine_active();

    let code_documents = &mut core_engine.code_documents().lock();

    let pid = uielement_focus_changed_msg.pid?;

    _ = check_if_code_doc_needs_to_be_created(
        code_documents,
        uielement_focus_changed_msg.window_id?,
        uielement_focus_changed_msg.pid?,
        uielement_focus_changed_msg.ui_elem_hash?,
    );

    let code_doc = code_documents.get_mut(&uielement_focus_changed_msg.ui_elem_hash?)?;

    // Update rule properties
    let content_str = get_textarea_content(&GetVia::Pid(pid)).ok()?;
    let file_path = get_textarea_file_path(&GetVia::Pid(pid)).ok();
    code_doc.update_doc_properties(&content_str, &file_path);

    let selected_text_range = get_selected_text_range(&GetVia::Pid(pid)).ok();
    if let Some(selected_text_range) = selected_text_range {
        code_doc.set_selected_text_range(selected_text_range.index, selected_text_range.length);
    }

    // Checking if the engine is active. If not, it returns.
    if !core_engine_active_status {
        return None;
    }

    code_doc.update_editor_window_viewport(get_viewport_frame(&GetVia::Current).ok()?);

    Some(())
}
