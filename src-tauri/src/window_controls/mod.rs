pub use config::default_properties;
pub use config::AppWindow;
pub use editor_window::EditorWindow;
pub use state_management::WindowStateManager;
pub use widget_window::WidgetWindow;

pub mod actions;
mod config;
pub mod content_window;
mod editor_window;
mod state_management;
mod widget_window;
