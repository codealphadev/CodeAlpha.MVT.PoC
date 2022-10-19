use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

use crate::{
    core_engine::{
        core_engine::{CoreEngineError, EditorWindowUid},
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

    let text_changed;
    {
        let swift_parser = core_engine.swift_parser();
        let code_documents = &mut core_engine.code_documents().lock();

        check_if_code_doc_needs_to_be_created(
            code_documents,
            content_changed_msg.pid,
            content_changed_msg.window_uid,
            swift_parser,
        );

        let code_doc = code_documents
            .get_mut(&content_changed_msg.window_uid)
            .ok_or(CoreEngineError::CodeDocNotFound(
                content_changed_msg.window_uid,
            ))?;

        text_changed = code_doc.update_doc_properties(
            &content_changed_msg.content,
            &content_changed_msg.file_path_as_str,
        );
    }

    if text_changed {
        core_engine.run_features(
            content_changed_msg.window_uid,
            &CoreEngineTrigger::OnTextContentChange,
        )?
    }
    Ok(())
}

pub fn check_if_code_doc_needs_to_be_created(
    code_documents: &mut HashMap<EditorWindowUid, CodeDocument>,
    editor_pid: i32,
    editor_window_uid: EditorWindowUid,
    swift_parser: Arc<Mutex<tree_sitter::Parser>>,
) {
    let new_code_doc = CodeDocument::new(
        &EditorWindowProps {
            window_uid: editor_window_uid,
            pid: editor_pid,
        },
        swift_parser,
    );

    // check if code document is already contained in list of documents
    if (*code_documents).get(&editor_window_uid).is_none() {
        (*code_documents).insert(editor_window_uid, new_code_doc);
    }
}
