use std::sync::{Arc, Mutex};

use accessibility::AXUIElementAttributes;
use core_foundation::string::CFString;
use tauri::Manager;

use crate::{
    ax_interaction::{generate_axui_element_hash, get_textarea_uielement},
    core_engine::{
        events::{
            models::{CoreActivationStatusMessage, SearchQueryMessage},
            EventUserInteraction,
        },
        rules::{search_and_replace::SearchRuleProps, RuleType},
        CoreEngine,
    },
    utils::messaging::ChannelList,
};

pub fn register_listener_user_interactions(
    app_handle: &tauri::AppHandle,
    core_engine: &Arc<Mutex<CoreEngine>>,
) {
    let core_engine_move_copy = (core_engine).clone();
    app_handle.listen_global(ChannelList::EventUserInteractions.to_string(), move |msg| {
        let event_user_interaction: EventUserInteraction =
            serde_json::from_str(&msg.payload().unwrap()).unwrap();

        match event_user_interaction {
            EventUserInteraction::SearchQuery(msg) => {
                on_search_query_by_user(&core_engine_move_copy, &msg);
            }
            EventUserInteraction::CoreActivationStatus(msg) => {
                on_core_activation_status_update(&core_engine_move_copy, &msg);
            }
            EventUserInteraction::None => {}
        }
    });
}

fn on_search_query_by_user(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    search_query_msg: &SearchQueryMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    let code_documents = &mut *(match core_engine.code_documents().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    // get PID from any open XCode editor window
    let pid = if let Some(random_code_doc) = code_documents.iter().find(|(_, _)| true) {
        random_code_doc.1.editor_window_props().pid
    } else {
        return;
    };

    // Fetch latest content from editor
    let textarea_uielement = if let Some(uielement) = get_textarea_uielement(pid) {
        uielement
    } else {
        return;
    };

    let textarea_content = if let Ok(content) = textarea_uielement.value() {
        let content_str = content.downcast::<CFString>();

        if let Some(cf_str) = content_str {
            cf_str.to_string()
        } else {
            return;
        }
    } else {
        return;
    };

    // Determine editor window hash to trigger rule on correct window
    let window_uielement = if let Ok(uielement) = textarea_uielement.window() {
        uielement
    } else {
        return;
    };

    let window_hash = generate_axui_element_hash(&window_uielement);

    // Run rule and update visualization
    if let Some(code_doc_of_active_editor_window) = code_documents
        .iter_mut()
        .find(|(_, code_doc)| code_doc.editor_window_props().uielement_hash == window_hash)
    {
        // Update properties of all rules
        for rule in code_doc_of_active_editor_window.1.rules_mut() {
            match rule {
                RuleType::SearchRule(search_rule) => {
                    search_rule.update_properties(SearchRuleProps {
                        search_str: Some(search_query_msg.query.clone()),
                        content: Some(textarea_content.clone()),
                    })
                }
                RuleType::_SwiftLinter(_) => {}
            }
        }

        code_doc_of_active_editor_window.1.process_rules();
        code_doc_of_active_editor_window
            .1
            .compute_rule_visualizations();
    }
}

fn on_core_activation_status_update(
    core_engine_arc: &Arc<Mutex<CoreEngine>>,
    core_activation_status_msg: &CoreActivationStatusMessage,
) {
    let core_engine = &mut *(match core_engine_arc.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    });

    core_engine.set_engine_active(core_activation_status_msg.engine_active);
}
