pub use config::default_properties;
pub use config::AppWindow;
pub use editor_window::EditorWindow;
pub use widget_window::WidgetWindow;
pub use window_controls::WindowControls;

pub mod actions;
mod config;
pub mod content_window;
mod editor_window;
mod widget_window;
mod window_controls;
