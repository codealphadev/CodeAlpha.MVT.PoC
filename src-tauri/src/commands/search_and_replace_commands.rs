use crate::core_engine::events::{models::SearchQueryMessage, EventUserInteraction};

/// This command contains too much accessibility logic - going to give us a harder time in the future. Need better design.
#[tauri::command]
pub fn cmd_search_and_replace(app_handle: tauri::AppHandle, search_str: String) {
    println!("cmd_search_and_replace: search_str: {}", search_str);
    EventUserInteraction::SearchQuery(SearchQueryMessage { query: search_str })
        .publish_to_tauri(&app_handle);
}
