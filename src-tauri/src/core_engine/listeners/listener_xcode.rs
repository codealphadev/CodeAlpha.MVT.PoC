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
            EditorTextareaContentChangedMessage, EditorTextareaScrolledMessage,
            EditorTextareaZoomedMessage, EditorUIElementFocusedMessage, EditorWindowCreatedMessage,
            EditorWindowDestroyedMessage, EditorWindowMovedMessage, EditorWindowResizedMessage,
            FocusedUIElement,
        },
        AXEventXcode,
    },
    core_engine::{
        rules::{RuleType, SearchRuleProps, SwiftLinterProps},
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
            AXEventXcode::EditorUIElementFocused(msg) => {
                on_editor_focused_uielement_changed(&core_engine_move_copy, &msg);
            }
            _ => {}
        }
    });
}

fn on_editor_textarea_content_changed(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    content_changed_msg: &EditorTextareaContentChangedMessage,
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

    if let Some(code_doc) = code_documents.get_mut(&content_changed_msg.id) {
        // Update rule properties
        for rule in code_doc.rules_mut() {
            match rule {
                RuleType::SearchRule(search_rule) => {
                    search_rule.update_properties(SearchRuleProps {
                        search_str: None,
                        content: Some(content_changed_msg.content.clone()),
                    })
                }
                RuleType::SwiftLinter(swift_linter_rule) => {
                    swift_linter_rule.update_properties(SwiftLinterProps {
                        file_path_as_str: Some(content_changed_msg.file_path_as_str.clone()),
                        linter_config: None,
                    })
                }
            }
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

    if let Some(code_doc) = code_documents.get_mut(&scrolled_msg.id) {
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

    if let Some(code_doc) = code_documents.get_mut(&zoomed_msg.id) {
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

    if let Some(code_doc) = code_documents.get_mut(&resized_msg.id) {
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

    if let Some(code_doc) = code_documents.get_mut(&moved_msg.id) {
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

    // Checking if the engine is active. If not, it returns.
    if !core_engine.engine_active() {
        return;
    }

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    if let Some(code_doc) = code_documents.get_mut(&uielement_focus_changed_msg.window_id) {
        let textarea_uielement =
            if let Some(uielem) = get_textarea_uielement(code_doc.editor_window_props().pid) {
                uielem
            } else {
                return;
            };

        // Update rule properties
        for rule in code_doc.rules_mut() {
            match rule {
                // Attempt to get content from the textarea.
                RuleType::SearchRule(search_rule) => {
                    if let Ok(content) = textarea_uielement.value() {
                        let content_str = if let Some(content_str) = content.downcast::<CFString>()
                        {
                            content_str.to_string()
                        } else {
                            continue;
                        };

                        search_rule.update_properties(SearchRuleProps {
                            search_str: None,
                            content: Some(content_str),
                        })
                    }
                }
                RuleType::SwiftLinter(swift_linter_rule) => {
                    let window_uielem = if let Ok(uielem) = textarea_uielement.window() {
                        uielem
                    } else {
                        continue;
                    };

                    if let Ok(file_path) = get_file_path_from_window(&window_uielem) {
                        swift_linter_rule.update_properties(SwiftLinterProps {
                            file_path_as_str: Some(file_path),
                            linter_config: None,
                        })
                    }
                }
            }
        }

        code_doc.process_rules();
        code_doc.compute_rule_visualizations();
    }
}
