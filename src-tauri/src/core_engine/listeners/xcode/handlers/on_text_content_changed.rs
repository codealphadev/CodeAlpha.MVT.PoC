use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{
    ax_interaction::{
        get_viewport_frame, models::editor::EditorTextareaContentChangedMessage, GetVia,
    },
    core_engine::{
        core_engine::UIElementHash, CodeDocument, CoreEngine, EditorWindowProps, TextRange,
    },
};

pub fn on_text_content_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    content_changed_msg: &EditorTextareaContentChangedMessage,
) {
    let core_engine = &mut core_engine_arc.lock();

    let core_engine_active_status = core_engine.engine_active();

    let code_documents = &mut core_engine.code_documents().lock();

    check_if_code_doc_needs_to_be_created(
        code_documents,
        content_changed_msg.id,
        content_changed_msg.pid,
        content_changed_msg.ui_elem_hash,
    );

    if let Some(code_doc) = code_documents.get_mut(&content_changed_msg.ui_elem_hash) {
        code_doc.update_doc_properties(
            &content_changed_msg.content,
            &content_changed_msg.file_path_as_str,
        );

        // Checking if the engine is active. If not, it returns.
        if !core_engine_active_status {
            return;
        }

        code_doc.process_rules();
        code_doc.compute_rule_visualizations();
    }
}

pub fn check_if_code_doc_needs_to_be_created(
    code_documents: &HashMap<UIElementHash, CodeDocument>,
    editor_id: uuid::Uuid,
    editor_pid: i32,
    editor_window_hash: UIElementHash,
) -> bool {
    let new_code_doc = CodeDocument::new(&EditorWindowProps {
        id: editor_id,
        pid: editor_pid,
        uielement_hash: editor_window_hash,
        viewport_frame: get_viewport_frame(&GetVia::Pid(editor_pid))
            .expect("Could not get viewport frame."),
        visible_text_range: TextRange::new(0, 0),
    });

    // check if code document is already contained in list of documents
    if (*code_documents).get(&editor_window_hash).is_none() {
        (*code_documents).insert(editor_window_hash, new_code_doc);
        true
    } else {
        false
    }
}
