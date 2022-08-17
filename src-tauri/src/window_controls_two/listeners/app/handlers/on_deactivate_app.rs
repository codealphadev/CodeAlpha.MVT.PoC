use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    ax_interaction::{
        currently_focused_window, generate_axui_element_hash, models::app::AppDeactivatedMessage,
    },
    window_controls_two::WindowManager,
};

pub fn on_deactivate_app(
    window_manager: &Arc<Mutex<WindowManager>>,
    _deactivated_msg: &AppDeactivatedMessage,
) {
    window_manager.lock().set_is_app_focused(false);

    // Determine if we need to hide our app
    // If the focus now is on a known editor window, we keep showing our app.
    // Subsequently arriving events will determine elsewhere if we need to hide our app.
    if let Ok(focused_window) = currently_focused_window() {
        let window_hash = generate_axui_element_hash(&focused_window);
        if window_manager
            .lock()
            .editor_windows()
            .lock()
            .get(&window_hash)
            .is_none()
        {
            todo!("WidgetWindow::hide_widget_routine(&widget_window.app_handle)");
        }
    } else {
        todo!("WidgetWindow::hide_widget_routine(&widget_window.app_handle)");
    }
}
