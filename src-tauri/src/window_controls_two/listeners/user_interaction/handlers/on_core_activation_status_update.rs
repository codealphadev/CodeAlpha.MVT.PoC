use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core_engine::events::models::CoreActivationStatusMessage, window_controls_two::WindowManager,
};

pub fn on_core_activation_status_update(
    window_manager: &Arc<Mutex<WindowManager>>,
    activation_msg: &CoreActivationStatusMessage,
) -> Option<()> {
    let mut window_manager = window_manager.lock();

    window_manager.set_is_core_engine_active(activation_msg.engine_active);

    let editor_windows = window_manager.editor_windows().lock();
    let _editor_window = editor_windows.get(&window_manager.focused_editor_window()?)?;

    // Depending on the activation status, we either show or hide the CodeOverlay window.
    if activation_msg.engine_active {
        todo!("show_code_overlay(&widget_props.app_handle, editor_window.textarea_position(true), editor_window.textarea_size(),");
    } else {
        todo!("editor_window.hide_widget_routine();");
    }

    Some(())
}
