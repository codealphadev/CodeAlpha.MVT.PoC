pub use on_app_activated::on_app_activated;
pub use on_close_editor_app::on_close_editor_app;
pub use on_destroyed_editor_window::on_editor_window_destroyed;
pub use on_move::on_editor_window_moved;
pub use on_resize::on_editor_window_resized;
pub use on_scroll::on_editor_textarea_scrolled;
pub use on_selected_text_changed::on_selected_text_changed;
pub use on_shortcut_pressed::on_editor_shortcut_pressed;
pub use on_text_content_changed::check_if_code_doc_needs_to_be_created;
pub use on_text_content_changed::on_text_content_changed;
pub use on_ui_element_focus_change::on_editor_focused_uielement_changed;
pub use on_zoom_editor_window::on_editor_textarea_zoomed;

mod on_app_activated;
mod on_close_editor_app;
mod on_destroyed_editor_window;
mod on_move;
mod on_resize;
mod on_scroll;
mod on_selected_text_changed;
mod on_shortcut_pressed;
mod on_text_content_changed;
mod on_ui_element_focus_change;
mod on_zoom_editor_window;
