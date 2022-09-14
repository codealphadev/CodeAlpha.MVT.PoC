pub use events::*;
pub use tracking_areas::*;
pub use window_manager::cmd_resize_window;
pub use window_manager::cmd_toggle_app_activation;
pub use window_manager::WindowManager;

pub mod config;
mod events;
mod listeners;
mod tracking_areas;
mod utils;
mod window_manager;
mod windows;
