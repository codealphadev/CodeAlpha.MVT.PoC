pub use config::default_properties;
pub use config::special_default_position_for_content_window;
pub use config::AppWindow;
pub use content_window::cmd_open_content_window;
pub use content_window::cmd_resize_content_window;
pub use content_window::cmd_toggle_content_window;
pub use content_window::ContentWindow;
pub use editor_window::EditorWindow;
pub use state_management::WindowStateManager;
pub use utils::*;
pub use widget_window::WidgetWindow;

pub mod actions;
mod config;
mod content_window;
mod editor_window;
mod state_management;
mod utils;
mod widget_window;
