use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{
    core_engine::{
        core_engine::{CoreEngineError, WindowUid},
        features::CoreEngineTrigger,
        CodeDocument, CoreEngine, EditorWindowProps,
    },
    platform::macos::models::editor::EditorTextareaContentChangedMessage,
};

pub fn on_text_content_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    content_changed_msg: &EditorTextareaContentChangedMessage,
) -> Result<(), CoreEngineError> {
    let core_engine = &mut core_engine_arc.lock();

    let core_engine_active_status = core_engine.engine_active();

    {
        let code_documents = &mut core_engine.code_documents().lock();

        check_if_code_doc_needs_to_be_created(
            code_documents,
            content_changed_msg.pid,
            content_changed_msg.window_uid,
        );

        let code_doc = code_documents
            .get_mut(&content_changed_msg.window_uid)
            .ok_or(CoreEngineError::CodeDocNotFound(
                content_changed_msg.window_uid,
            ))?;

        code_doc.update_doc_properties(
            &content_changed_msg.content,
            &content_changed_msg.file_path_as_str,
        );

        // Checking if the engine is active. If not, it returns.
        if !core_engine_active_status {
            return Ok(());
        }

        code_doc.process_rules();
        code_doc.compute_rule_visualizations();
    }

    core_engine.run_features(
        content_changed_msg.window_uid,
        &CoreEngineTrigger::OnTextContentChange,
    )
}

pub fn check_if_code_doc_needs_to_be_created(
    code_documents: &mut HashMap<WindowUid, CodeDocument>,
    editor_pid: i32,
    editor_window_uid: WindowUid,
) -> bool {
    let new_code_doc = CodeDocument::new(&EditorWindowProps {
        window_uid: editor_window_uid,
        pid: editor_pid,
    });

    // check if code document is already contained in list of documents
    if (*code_documents).get(&editor_window_uid).is_none() {
        (*code_documents).insert(editor_window_uid, new_code_doc);
        true
    } else {
        false
    }
}
