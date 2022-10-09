pub use events::*;
pub use observer_device_events::pressed_mouse_buttons;
pub use observer_device_events::send_event_mouse_wheel;
pub use setup::setup_observers;
pub use simulated_scrolling::*;
pub use utils::*;

pub mod app;
pub mod setup;
pub mod utils;
pub mod xcode;

mod events;
mod observer_device_events;
mod simulated_scrolling;
