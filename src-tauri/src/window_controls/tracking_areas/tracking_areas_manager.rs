use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    app_handle,
    platform::macos::models::input_device::{ClickType, MouseButton, MouseClickMessage},
    utils::geometry::LogicalPosition,
    window_controls::{
        config::AppWindow,
        events::{
            models::{
                TrackingAreaClickedMessage, TrackingAreaEnteredMessage, TrackingAreaExitedMessage,
            },
            EventWindowControls,
        },
        models::{TrackingAreaClickedOutsideMessage, TrackingAreaMouseOverMessage},
        utils::{get_position, get_size, get_window_level, is_visible},
    },
};

use super::{
    listeners::{input_devices_listener, tracking_area_listener},
    TrackingArea, TrackingEventType, TrackingEvents,
};

#[derive(Clone, Debug)]
pub struct TrackingAreasManager {
    pub app_handle: tauri::AppHandle,
    tracking_areas: Vec<(TrackingArea, Option<std::time::Instant>)>,
    previous_mouse_position: Option<(f64, f64)>,
}

#[derive(Debug)]
struct TrackingEvent {
    tracking_area: TrackingArea,
    event_type: TrackingEventType,
    mouse_position_local: LogicalPosition,
    button: MouseButton,
    click_type: ClickType,
    duration_in_area_ms: Option<u64>,
}

impl TrackingAreasManager {
    pub fn new() -> Self {
        Self {
            app_handle: app_handle(),
            tracking_areas: Vec::new(),
            previous_mouse_position: None,
        }
    }

    pub fn add_tracking_areas(&mut self, tracking_areas: Vec<TrackingArea>) {
        let mut new_tracking_areas = Vec::new();
        for tracking_area in tracking_areas {
            new_tracking_areas.push((tracking_area, None));
        }
        self.tracking_areas.append(&mut new_tracking_areas);
    }

    pub fn remove_tracking_areas(&mut self, tracking_areas: Vec<uuid::Uuid>) {
        self.tracking_areas
            .retain(|(tracking_area, _)| !tracking_areas.contains(&tracking_area.id));
    }

    pub fn reset_tracking_areas(&mut self) {
        self.tracking_areas.clear();
    }

    pub fn update_tracking_areas(&mut self, tracking_areas: Vec<TrackingArea>) {
        for updated_tracking_area in tracking_areas.iter() {
            for tracking_area in self.tracking_areas.iter_mut() {
                if tracking_area.0.id == updated_tracking_area.id {
                    tracking_area.0.update(updated_tracking_area);
                }
            }
        }
    }

    pub fn replace_tracking_areas(&mut self, tracking_areas: Vec<TrackingArea>) {
        let mut new_tracking_areas: Vec<(TrackingArea, Option<std::time::Instant>)> = Vec::new();
        for tracking_area in tracking_areas {
            new_tracking_areas.push((tracking_area, None));
        }

        self.tracking_areas = new_tracking_areas;
    }

    pub fn track_mouse_position(&mut self, mouse_x: f64, mouse_y: f64) -> Option<()> {
        self.previous_mouse_position = Some((mouse_x, mouse_y));

        // `Option<u64>` contains the duration in millis for how long the mouse has been in this tracking area.
        let mut tracking_events: Vec<TrackingEvent> = Vec::new();

        for tracking_area in self.tracking_areas.iter_mut() {
            if !(is_visible(tracking_area.0.app_window).ok() == Some(true)) {
                continue;
            }

            if tracking_area
                .0
                .rect_as_global()
                .contains_point(mouse_x, mouse_y)
            {
                if let Some(tracking_start) = tracking_area.1 {
                    // Case: TrackingArea was already entered before.
                    if is_blocked_by_other_app_window(tracking_area.0.app_window, mouse_x, mouse_y)
                    {
                        // Case: Mouse is still inside the tracking area, but an app window opens above it.
                        // We publish a MouseExited event to the tracking area.
                        tracking_events.push(TrackingEvent {
                            tracking_area: tracking_area.0.clone(),
                            event_type: TrackingEventType::MouseExited,
                            duration_in_area_ms: Some(tracking_start.elapsed().as_millis() as u64),
                            mouse_position_local: LogicalPosition {
                                x: mouse_x,
                                y: mouse_y,
                            }
                            .to_local(&tracking_area.0.global_origin()),
                            button: MouseButton::None,
                            click_type: ClickType::None,
                        });
                        tracking_area.1 = None;
                        continue;
                    } else {
                        tracking_events.push(TrackingEvent {
                            tracking_area: tracking_area.0.clone(),
                            event_type: TrackingEventType::MouseOver,
                            duration_in_area_ms: Some(tracking_start.elapsed().as_millis() as u64),
                            mouse_position_local: LogicalPosition {
                                x: mouse_x,
                                y: mouse_y,
                            }
                            .to_local(&tracking_area.0.global_origin()),
                            button: MouseButton::None,
                            click_type: ClickType::None,
                        });
                    }
                } else {
                    // Case: TrackingArea was not entered before, start tracking the time spent in the area.
                    if is_blocked_by_other_app_window(tracking_area.0.app_window, mouse_x, mouse_y)
                    {
                        continue;
                    } else {
                        tracking_area.1 = Some(std::time::Instant::now());
                        tracking_events.push(TrackingEvent {
                            tracking_area: tracking_area.0.clone(),
                            event_type: TrackingEventType::MouseEntered,
                            duration_in_area_ms: None,
                            mouse_position_local: LogicalPosition {
                                x: mouse_x,
                                y: mouse_y,
                            }
                            .to_local(&tracking_area.0.global_origin()),
                            button: MouseButton::None,
                            click_type: ClickType::None,
                        });
                    }
                }
            } else {
                // Case: Mouse is not inside the tracking area.
                if let Some(tracking_start) = tracking_area.1 {
                    // Case: TrackingArea was entered before, now the mouse is not inside it anymore,
                    // publish a MouseExited event to the tracking area.
                    tracking_area.1 = None;
                    tracking_events.push(TrackingEvent {
                        tracking_area: tracking_area.0.clone(),
                        event_type: TrackingEventType::MouseExited,
                        duration_in_area_ms: Some(tracking_start.elapsed().as_millis() as u64),
                        mouse_position_local: LogicalPosition {
                            x: mouse_x,
                            y: mouse_y,
                        }
                        .to_local(&tracking_area.0.global_origin()),
                        button: MouseButton::None,
                        click_type: ClickType::None,
                    });
                }
            }
        }

        self.publish_tracking_state(&tracking_events);

        Some(())
    }

    pub fn track_mouse_click(&mut self, click_msg: &MouseClickMessage) -> Option<()> {
        self.previous_mouse_position =
            Some((click_msg.cursor_position.x, click_msg.cursor_position.y));

        // `Option<u128>` contains the duration in millis for how long the mouse has been in this tracking area.
        let mut tracking_results: Vec<TrackingEvent> = Vec::new();

        for tracking_area in self.tracking_areas.iter() {
            if !(is_visible(tracking_area.0.app_window).ok() == Some(true)) {
                continue;
            }

            if is_blocked_by_other_app_window(
                tracking_area.0.app_window,
                click_msg.cursor_position.x,
                click_msg.cursor_position.y,
            ) {
                continue;
            }

            if tracking_area
                .0
                .rect_as_global()
                .contains_point(click_msg.cursor_position.x, click_msg.cursor_position.y)
            {
                if let Some(tracking_start) = tracking_area.1 {
                    tracking_results.push(TrackingEvent {
                        tracking_area: tracking_area.0.clone(),
                        event_type: TrackingEventType::MouseClicked,
                        duration_in_area_ms: Some(tracking_start.elapsed().as_millis() as u64),
                        mouse_position_local: LogicalPosition {
                            x: click_msg.cursor_position.x,
                            y: click_msg.cursor_position.y,
                        }
                        .to_local(&tracking_area.0.global_origin()),
                        button: click_msg.button.clone(),
                        click_type: click_msg.click_type.clone(),
                    });
                } else {
                    tracking_results.push(TrackingEvent {
                        tracking_area: tracking_area.0.clone(),
                        event_type: TrackingEventType::MouseClicked,
                        duration_in_area_ms: None,
                        mouse_position_local: LogicalPosition {
                            x: click_msg.cursor_position.x,
                            y: click_msg.cursor_position.y,
                        }
                        .to_local(&tracking_area.0.global_origin()),
                        button: click_msg.button.clone(),
                        click_type: click_msg.click_type.clone(),
                    });
                }
            } else {
                // Check if tracking area subscribed to MouseClickedOutside event.
                if Self::evaluate_event_subscriptions(
                    &TrackingEventType::MouseClickedOutside,
                    &tracking_area.0.events,
                ) {
                    tracking_results.push(TrackingEvent {
                        tracking_area: tracking_area.0.clone(),
                        event_type: TrackingEventType::MouseClickedOutside,
                        duration_in_area_ms: None,
                        mouse_position_local: LogicalPosition {
                            x: click_msg.cursor_position.x,
                            y: click_msg.cursor_position.y,
                        }
                        .to_local(&tracking_area.0.global_origin()),
                        button: click_msg.button.clone(),
                        click_type: click_msg.click_type.clone(),
                    });
                }
            }
        }

        self.publish_tracking_state(&tracking_results);

        Some(())
    }

    fn publish_tracking_state(&self, tracking_results: &Vec<TrackingEvent>) {
        for TrackingEvent {
            tracking_area,
            duration_in_area_ms,
            event_type,
            mouse_position_local: mouse_position,
            button,
            click_type,
        } in tracking_results.iter()
        {
            if Self::evaluate_event_subscriptions(event_type, &tracking_area.events) {
                match event_type {
                    TrackingEventType::MouseEntered => {
                        EventWindowControls::TrackingAreaEntered(TrackingAreaEnteredMessage {
                            id: tracking_area.id,
                            window_uid: tracking_area.window_uid,
                            app_window: tracking_area.app_window,
                            mouse_position: *mouse_position,
                        })
                        .publish_tracking_area(&tracking_area.subscriber);
                    }
                    TrackingEventType::MouseExited => {
                        EventWindowControls::TrackingAreaExited(TrackingAreaExitedMessage {
                            id: tracking_area.id,
                            window_uid: tracking_area.window_uid,
                            duration_ms: duration_in_area_ms.map_or(0, |dur| dur),
                            app_window: tracking_area.app_window,
                            mouse_position: *mouse_position,
                        })
                        .publish_tracking_area(&tracking_area.subscriber);
                    }
                    TrackingEventType::MouseOver => {
                        EventWindowControls::TrackingAreaMouseOver(TrackingAreaMouseOverMessage {
                            id: tracking_area.id,
                            window_uid: tracking_area.window_uid,
                            duration_ms: duration_in_area_ms.map_or(0, |dur| dur),
                            app_window: tracking_area.app_window,
                            mouse_position: *mouse_position,
                        })
                        .publish_tracking_area(&tracking_area.subscriber);
                    }
                    TrackingEventType::MouseClicked => {
                        EventWindowControls::TrackingAreaClicked(TrackingAreaClickedMessage {
                            id: tracking_area.id,
                            window_uid: tracking_area.window_uid,
                            duration_ms: duration_in_area_ms.map_or(0, |dur| dur),
                            app_window: tracking_area.app_window,
                            mouse_position: *mouse_position,
                            button: button.clone(),
                            click_type: click_type.clone(),
                        })
                        .publish_tracking_area(&tracking_area.subscriber);
                    }
                    TrackingEventType::MouseClickedOutside => {
                        EventWindowControls::TrackingAreaClickedOutside(
                            TrackingAreaClickedOutsideMessage {
                                id: tracking_area.id,
                                window_uid: tracking_area.window_uid,
                                app_window: tracking_area.app_window,
                                mouse_position: *mouse_position,
                                button: button.clone(),
                                click_type: click_type.clone(),
                            },
                        )
                        .publish_tracking_area(&tracking_area.subscriber);
                    }
                }
            }
        }
    }

    fn evaluate_event_subscriptions(
        tracking_event_type: &TrackingEventType,
        subscriptions: &TrackingEvents,
    ) -> bool {
        match subscriptions {
            TrackingEvents::TrackingEventTypes(subscriptions) => subscriptions
                .iter()
                .any(|subscription| subscription == tracking_event_type),
            TrackingEvents::All => true,
            TrackingEvents::None => false,
        }
    }

    pub fn start_event_listeners(tracking_area_manager: &Arc<Mutex<Self>>) {
        tracking_area_listener(tracking_area_manager);
        input_devices_listener(tracking_area_manager);
    }
}

fn is_blocked_by_other_app_window(
    checked_app_window: AppWindow,
    mouse_x: f64,
    mouse_y: f64,
) -> bool {
    use strum::IntoEnumIterator;

    if !(is_visible(checked_app_window).ok() == Some(true)) {
        return false;
    }

    let window_level_checked_app_window =
        if let Some(window_level) = get_window_level(checked_app_window) {
            window_level
        } else {
            panic!("No window level for: AppWindow: {:?}", checked_app_window);
        };

    for app_window in AppWindow::iter() {
        if app_window == checked_app_window {
            continue;
        }

        if !(is_visible(app_window).ok() == Some(true)) {
            continue;
        }

        if let Some(window_level) = get_window_level(app_window) {
            // Only check if the window is above the overlay window.
            if window_level > window_level_checked_app_window {
                if let (Some(origin), Some(size)) = (get_position(app_window), get_size(app_window))
                {
                    if mouse_x >= origin.x
                        && mouse_x <= origin.x + size.width
                        && mouse_y >= origin.y
                        && mouse_y <= origin.y + size.height
                    {
                        return true;
                    }
                }
            }
        }
    }

    false
}
