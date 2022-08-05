use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use accessibility::AXUIElementAttributes;
use core_foundation::string::CFString;
use tauri::Manager;

use crate::{
    ax_interaction::{
        get_file_path_from_window, get_textarea_uielement,
        models::editor::{
            EditorShortcutPressedMessage, EditorTextareaContentChangedMessage,
            EditorTextareaScrolledMessage, EditorTextareaSelectedTextChangedMessage,
            EditorTextareaZoomedMessage, EditorUIElementFocusedMessage,
            EditorWindowDestroyedMessage, EditorWindowMovedMessage, EditorWindowResizedMessage,
            FocusedUIElement, ModifierKey,
        },
        AXEventXcode,
    },
    core_engine::{core_engine::UIElementHash, CodeDocument, CoreEngine, EditorWindowProps},
    utils::messaging::ChannelList,
};

pub fn register_listener_xcode(
    app_handle: &tauri::AppHandle,
    core_engine: &Arc<Mutex<CoreEngine>>,
) {
    let core_engine_move_copy = core_engine.clone();
    app_handle.listen_global(ChannelList::AXEventXcode.to_string(), move |msg| {
        let axevent_xcode: AXEventXcode = serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match axevent_xcode {
            AXEventXcode::EditorWindowMoved(msg) => {
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
            AXEventXcode::EditorTextareaSelectedTextChanged(msg) => {
                on_editor_textarea_selected_text_changed(&core_engine_move_copy, &msg);
            }
            AXEventXcode::EditorAppClosed(_) => {
                on_close_editor_app(&core_engine_move_copy);
            }
            AXEventXcode::EditorWindowCreated(_) => {
                // We don't do anything because we don't keep track of open windows, here we are only
                // interested in the displayed document
            }
            AXEventXcode::EditorWindowDestroyed(msg) => {
                on_editor_window_destroyed(&core_engine_move_copy, &msg);
            }
            AXEventXcode::EditorUIElementFocused(msg) => {
                on_editor_focused_uielement_changed(&core_engine_move_copy, &msg);
            }
            AXEventXcode::EditorShortcutPressed(msg) => {
                on_editor_shortcut_pressed(&core_engine_move_copy, &msg);
            }
            _ => {}
        }
    });
}

fn on_editor_shortcut_pressed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    msg: &EditorShortcutPressedMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    // Checking if the engine is active. If not, don't continue.
    if !core_engine.engine_active() {
        return;
    }

    match msg.modifier {
        ModifierKey::Cmd => match msg.key.as_str() {
            "S" => {
                let code_documents = &mut *(match core_engine.code_documents().lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                });

                if let Some(code_doc) = code_documents.get_mut(&msg.ui_elem_hash) {
                    code_doc.on_save();
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn on_editor_textarea_selected_text_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    msg: &EditorTextareaSelectedTextChangedMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let core_engine_active_status = core_engine.engine_active();

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&msg.ui_elem_hash) {
        code_doc.set_selected_text_range(msg.index, msg.length);
    }

    // Checking if the engine is active. If not, don't continue.
    if !core_engine_active_status {
        return;
    }
}

fn on_editor_textarea_content_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    content_changed_msg: &EditorTextareaContentChangedMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let core_engine_active_status = core_engine.engine_active();

    let app_handle = core_engine.app_handle.clone();

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let _ = check_if_code_doc_needs_to_be_created(
        &app_handle,
        code_documents,
        EditorWindowProps {
            id: content_changed_msg.id,
            pid: content_changed_msg.pid,
            uielement_hash: content_changed_msg.ui_elem_hash,
        },
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

fn on_editor_textarea_scrolled(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    scrolled_msg: &EditorTextareaScrolledMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&scrolled_msg.uielement_hash) {
        code_doc.compute_rule_visualizations();
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

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&zoomed_msg.uielement_hash) {
        code_doc.compute_rule_visualizations();
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

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&resized_msg.uielement_hash) {
        code_doc.compute_rule_visualizations();
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

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&moved_msg.uielement_hash) {
        code_doc.compute_rule_visualizations();
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

fn check_if_code_doc_needs_to_be_created(
    app_handle: &tauri::AppHandle,
    code_documents: &mut HashMap<UIElementHash, CodeDocument>,
    editor_window_props: EditorWindowProps,
) -> bool {
    let new_code_doc = CodeDocument::new(
        app_handle.clone(),
        EditorWindowProps {
            id: editor_window_props.id,
            pid: editor_window_props.pid,
            uielement_hash: editor_window_props.uielement_hash,
        },
    );

    // check if code document is already contained in list of documents
    if (*code_documents)
        .get(&editor_window_props.uielement_hash)
        .is_none()
    {
        (*code_documents).insert(editor_window_props.uielement_hash, new_code_doc);
        true
    } else {
        false
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

    let _ = &code_documents.remove(&destroyed_msg.uielement_hash);
}

fn on_editor_focused_uielement_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    uielement_focus_changed_msg: &EditorUIElementFocusedMessage,
) {
    if uielement_focus_changed_msg.focused_ui_element != FocusedUIElement::Textarea {
        return;
    }

    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let core_engine_active_status = core_engine.engine_active();

    let app_handle = core_engine.app_handle.clone();

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let textarea_uielement =
        if let Some(uielem) = get_textarea_uielement(uielement_focus_changed_msg.pid) {
            uielem
        } else {
            return;
        };

    // Update rule properties
    let content_str = if let Ok(content) = textarea_uielement.value() {
        if let Some(content_str) = content.downcast::<CFString>() {
            content_str.to_string()
        } else {
            return;
        }
    } else {
        return;
    };

    let file_path = if let Ok(uielem) = textarea_uielement.window() {
        if let Ok(file_path) = get_file_path_from_window(&uielem) {
            Some(file_path)
        } else {
            None
        }
    } else {
        None
    };

    _ = check_if_code_doc_needs_to_be_created(
        &app_handle,
        code_documents,
        EditorWindowProps {
            id: uielement_focus_changed_msg.window_id,
            pid: uielement_focus_changed_msg.pid,
            uielement_hash: uielement_focus_changed_msg.ui_elem_hash,
        },
    );

    if let Some(code_doc) = code_documents.get_mut(&uielement_focus_changed_msg.ui_elem_hash) {
        code_doc.update_doc_properties(&content_str, &file_path);

        // Checking if the engine is active. If not, it returns.
        if !core_engine_active_status {
            return;
        }

        code_doc.process_rules();
        code_doc.compute_rule_visualizations();
    }
}
