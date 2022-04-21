/// This command contains too much accessibility logic - going to give us a harder time in the future. Need better design.
#[tauri::command]
pub fn cmd_search_and_replace(
    search_str: String,
    replace_str: String,
    // state: tauri::State<'_, WindowStateMachine>,
) {
    // let pid = state.last_known_xcode_app_pid.lock().unwrap();

    // if let Some(pid) = *pid {
    //     let content = get_xcode_editor_content(pid.try_into().unwrap());

    //     if let Ok(content) = content {
    //         if let Some(content_str) = content {
    //             let content_str = content_str.replace(&search_str, &replace_str);
    //             let _ = update_xcode_editor_content(pid.try_into().unwrap(), &content_str);
    //         }
    //     }
    // }
}
