pub use events::*;
pub use observer_device_events::pressed_mouse_buttons;
pub use setup::setup_observers;
pub use simulated_scrolling::*;
pub use utils::*;

pub mod app;
pub mod menu;
pub mod permissions_check;
pub mod setup;
pub mod system_tray;
pub mod utils;
pub mod xcode;

mod events;
mod observer_device_events;
mod simulated_scrolling;
