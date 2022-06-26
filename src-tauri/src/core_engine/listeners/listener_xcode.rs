use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tauri::Manager;

use crate::{
    ax_interaction::{
        models::editor::{
            EditorTextareaContentChanged, EditorTextareaScrolledMessage,
            EditorTextareaZoomedMessage, EditorWindowCreatedMessage, EditorWindowDestroyedMessage,
            EditorWindowMovedMessage, EditorWindowResizedMessage,
        },
        AXEventXcode,
    },
    core_engine::{
        rules::{search_and_replace::SearchRule, RuleType},
        CodeDocument, CoreEngine, EditorWindowProps,
    },
    utils::messaging::ChannelList,
};

pub fn register_listener_xcode(
    app_handle: &tauri::AppHandle,
    core_engine: &Arc<Mutex<CoreEngine>>,
) {
    let core_engine_move_copy = (core_engine).clone();
    app_handle.listen_global(ChannelList::AXEventXcode.to_string(), move |msg| {
        let axevent_xcode: AXEventXcode = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match axevent_xcode {
            AXEventXcode::EditorWindowMoved(msg) => {
                // TODO: dont't handle this here, but in the frontend -> more efficient implementation
                on_editor_window_moved(&core_engine_move_copy, &msg);
            }
            AXEventXcode::EditorWindowResized(msg) => {
                on_editor_window_resized(&core_engine_move_copy, &msg);
            }
            AXEventXcode::EditorTextareaScrolled(msg) => {
                on_editor_textarea_scrolled(&core_engine_move_copy, &msg);
            }
            AXEventXcode::EditorTextareaZoomed(msg) => {
                on_editor_textarea_zoomed(&core_engine_move_copy, &msg);
            }
            AXEventXcode::EditorTextareaContentChanged(msg) => {
                on_editor_textarea_content_changed(&core_engine_move_copy, &msg);
            }
            AXEventXcode::EditorAppClosed(_) => {
                on_close_editor_app(&core_engine_move_copy);
            }
            AXEventXcode::EditorWindowCreated(msg) => {
                on_editor_window_created(&core_engine_move_copy, &msg);
            }
            AXEventXcode::EditorWindowDestroyed(msg) => {
                on_editor_window_destroyed(&core_engine_move_copy, &msg);
            }
            _ => {}
        }
    });
}

fn on_editor_textarea_content_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    content_changed_msg: &EditorTextareaContentChanged,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    // Checking if the engine is active and if the active feature is SearchAndReplace. If not, it
    // returns.
    if !core_engine.engine_active() || !(core_engine.active_feature() == RuleType::SearchAndReplace)
    {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&content_changed_msg.id) {
        code_doc.compute_search_and_replace_rule(Some(content_changed_msg.content.clone()), None);
        code_doc.compute_search_and_replace_rule_visualization();
    }
}

fn on_editor_textarea_scrolled(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    scrolled_msg: &EditorTextareaScrolledMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    // Checking if the engine is active and if the active feature is SearchAndReplace. If not, it
    // returns.
    if !core_engine.engine_active() || !(core_engine.active_feature() == RuleType::SearchAndReplace)
    {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&scrolled_msg.id) {
        code_doc.compute_search_and_replace_rule_visualization();
    }
}

fn on_editor_textarea_zoomed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    zoomed_msg: &EditorTextareaZoomedMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    // Checking if the engine is active and if the active feature is SearchAndReplace. If not, it
    // returns.
    if !core_engine.engine_active() || !(core_engine.active_feature() == RuleType::SearchAndReplace)
    {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&zoomed_msg.id) {
        code_doc.compute_search_and_replace_rule_visualization();
    }
}

fn on_editor_window_resized(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    resized_msg: &EditorWindowResizedMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    // Checking if the engine is active and if the active feature is SearchAndReplace. If not, it
    // returns.
    if !core_engine.engine_active() || !(core_engine.active_feature() == RuleType::SearchAndReplace)
    {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&resized_msg.id) {
        code_doc.compute_search_and_replace_rule_visualization();
    }
}

fn on_editor_window_moved(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    moved_msg: &EditorWindowMovedMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    // Checking if the engine is active and if the active feature is SearchAndReplace. If not, it
    // returns.
    if !core_engine.engine_active() || !(core_engine.active_feature() == RuleType::SearchAndReplace)
    {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&moved_msg.id) {
        code_doc.compute_search_and_replace_rule_visualization();
    }
}

fn on_close_editor_app(core_engine_arc: &Arc<Mutex<CoreEngine>>) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    *code_documents = HashMap::new();
}

fn on_editor_window_created(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    created_msg: &EditorWindowCreatedMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let new_code_doc = CodeDocument::new(
        core_engine.app_handle.clone(),
        EditorWindowProps {
            id: created_msg.id,
            uielement_hash: created_msg.ui_elem_hash,
            pid: created_msg.pid,
        },
        SearchRule::new(),
    );

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    // check if code document is already contained in list of documents
    if (*code_documents).get(&created_msg.id).is_none() {
        (*code_documents).insert(created_msg.id, new_code_doc);
    }
}

fn on_editor_window_destroyed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    destroyed_msg: &EditorWindowDestroyedMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let _ = &code_documents.remove(&destroyed_msg.id);
}
