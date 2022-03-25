use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;

use crate::utils::xcode_twin::XCodeTwin;

#[tauri::command]
fn search_and_replace(search_str: String, replace_str: String, state: tauri::State<'_, XCodeTwin>) {
    if let Some(current_content) = state.get_state_xcode_editor_content() {
        let content_str = current_content.content;
        let content_str = content_str.replace(&search_str, &replace_str);

        state.update_xcode_editor_content(&content_str);
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("xcode-state-plugin")
        .invoke_handler(tauri::generate_handler![search_and_replace])
        .build()
}
