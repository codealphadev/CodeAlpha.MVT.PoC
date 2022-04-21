pub use dimension_calculations::prevent_misalignement_of_content_and_widget;
pub use dimension_calculations::POSITIONING_OFFSET_X;
pub use dimension_calculations::POSITIONING_OFFSET_Y;
pub use widget_window::WidgetWindow;

mod decision_tree_show_hide_widget;
mod dimension_calculations;
mod handler_ax_events_app;
mod handler_ax_events_xcode;
mod listeners;
mod widget_window;
