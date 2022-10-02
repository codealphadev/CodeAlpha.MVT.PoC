use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    app_handle,
    core_engine::EditorWindowUid,
    utils::geometry::{LogicalPosition, LogicalSize},
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
    listeners::{input_devices_listener, tracking_area_listener, xcode_listener},
    TrackingArea, TrackingEventSubscription, TrackingEventType,
};

#[derive(Clone, Debug)]
pub struct TrackingAreasManager {
    pub app_handle: tauri::AppHandle,
    tracking_areas: Vec<(TrackingArea, Option<std::time::Instant>)>,
    previous_mouse_position: Option<(f64, f64)>,
}

struct TrackingEvent {
    area: TrackingArea,
    event_type: TrackingEventType,
    mouse_position_local: LogicalPosition,
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
        println!("update_tracking_areas: {:?}", tracking_areas);
        for updated_tracking_area in tracking_areas.iter() {
            for tracking_area in self.tracking_areas.iter_mut() {
                if tracking_area.0.id == updated_tracking_area.id {
                    tracking_area.0.update(updated_tracking_area);
                }
            }
        }
    }

    pub fn move_tracking_areas(
        &mut self,
        move_distance: &LogicalSize,
        window_uid: EditorWindowUid,
    ) {
        for tracking_area in self.tracking_areas.iter_mut() {
            if tracking_area.0.editor_window_uid == window_uid {
                tracking_area.0.rectangle.origin.x += move_distance.width;
                tracking_area.0.rectangle.origin.y += move_distance.height;
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
            if tracking_area.0.rectangle.contains_point(mouse_x, mouse_y) {
                if let Some(tracking_start) = tracking_area.1 {
                    // Case: TrackingArea was already entered before.
                    if is_blocked_by_other_app_window(tracking_area.0.app_window, mouse_x, mouse_y)
                    {
                        // Case: Mouse is still inside the tracking area, but an app window opens above it.
                        // We publish a MouseExited event to the tracking area.
                        tracking_events.push(TrackingEvent {
                            area: tracking_area.0.clone(),
                            event_type: TrackingEventType::MouseExited,
                            duration_in_area_ms: Some(tracking_start.elapsed().as_millis() as u64),
                            mouse_position_local: LogicalPosition {
                                x: mouse_x,
                                y: mouse_y,
                            }
                            .to_local(&tracking_area.0.rectangle.origin),
                        });
                        tracking_area.1 = None;
                        continue;
                    } else {
                        tracking_events.push(TrackingEvent {
                            area: tracking_area.0.clone(),
                            event_type: TrackingEventType::MouseOver,
                            duration_in_area_ms: Some(tracking_start.elapsed().as_millis() as u64),
                            mouse_position_local: LogicalPosition {
                                x: mouse_x,
                                y: mouse_y,
                            }
                            .to_local(&tracking_area.0.rectangle.origin),
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
                            area: tracking_area.0.clone(),
                            event_type: TrackingEventType::MouseEntered,
                            duration_in_area_ms: None,
                            mouse_position_local: LogicalPosition {
                                x: mouse_x,
                                y: mouse_y,
                            }
                            .to_local(&tracking_area.0.rectangle.origin),
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
                        area: tracking_area.0.clone(),
                        event_type: TrackingEventType::MouseExited,
                        duration_in_area_ms: Some(tracking_start.elapsed().as_millis() as u64),
                        mouse_position_local: LogicalPosition {
                            x: mouse_x,
                            y: mouse_y,
                        }
                        .to_local(&tracking_area.0.rectangle.origin),
                    });
                }
            }
        }

        self.publish_tracking_state(&tracking_events);

        Some(())
    }

    pub fn track_mouse_click(&mut self, mouse_x: f64, mouse_y: f64) -> Option<()> {
        self.previous_mouse_position = Some((mouse_x, mouse_y));

        // `Option<u128>` contains the duration in millis for how long the mouse has been in this tracking area.
        let mut tracking_results: Vec<TrackingEvent> = Vec::new();

        for tracking_area in self.tracking_areas.iter() {
            if is_blocked_by_other_app_window(tracking_area.0.app_window, mouse_x, mouse_y) {
                continue;
            }

            if tracking_area.0.rectangle.contains_point(mouse_x, mouse_y) {
                if let Some(tracking_start) = tracking_area.1 {
                    tracking_results.push(TrackingEvent {
                        area: tracking_area.0.clone(),
                        event_type: TrackingEventType::MouseClicked,
                        duration_in_area_ms: Some(tracking_start.elapsed().as_millis() as u64),
                        mouse_position_local: LogicalPosition {
                            x: mouse_x,
                            y: mouse_y,
                        }
                        .to_local(&tracking_area.0.rectangle.origin),
                    });
                } else {
                    tracking_results.push(TrackingEvent {
                        area: tracking_area.0.clone(),
                        event_type: TrackingEventType::MouseClicked,
                        duration_in_area_ms: None,
                        mouse_position_local: LogicalPosition {
                            x: mouse_x,
                            y: mouse_y,
                        }
                        .to_local(&tracking_area.0.rectangle.origin),
                    });
                }
            } else {
                // Check if tracking area subscribed to MouseClickedOutside event.
                if Self::evaluate_event_subscriptions(
                    &TrackingEventType::MouseClickedOutside,
                    &tracking_area.0.event_subscriptions,
                ) {
                    tracking_results.push(TrackingEvent {
                        area: tracking_area.0.clone(),
                        event_type: TrackingEventType::MouseClickedOutside,
                        duration_in_area_ms: None,
                        mouse_position_local: LogicalPosition {
                            x: mouse_x,
                            y: mouse_y,
                        }
                        .to_local(&tracking_area.0.rectangle.origin),
                    });
                }
            }
        }

        self.publish_tracking_state(&tracking_results);

        Some(())
    }

    fn publish_tracking_state(&self, tracking_results: &Vec<TrackingEvent>) {
        for TrackingEvent {
            area,
            duration_in_area_ms,
            event_type,
            mouse_position_local: mouse_position,
        } in tracking_results.iter()
        {
            if Self::evaluate_event_subscriptions(event_type, &area.event_subscriptions) {
                match event_type {
                    TrackingEventType::MouseEntered => {
                        EventWindowControls::TrackingAreaEntered(TrackingAreaEnteredMessage {
                            id: area.id,
                            editor_window_uid: area.editor_window_uid,
                            app_window: area.app_window,
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                    TrackingEventType::MouseExited => {
                        EventWindowControls::TrackingAreaExited(TrackingAreaExitedMessage {
                            id: area.id,
                            editor_window_uid: area.editor_window_uid,
                            duration_ms: duration_in_area_ms.map_or(0, |dur| dur),
                            app_window: area.app_window,
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                    TrackingEventType::MouseOver => {
                        EventWindowControls::TrackingAreaMouseOver(TrackingAreaMouseOverMessage {
                            id: area.id,
                            editor_window_uid: area.editor_window_uid,
                            duration_ms: duration_in_area_ms.map_or(0, |dur| dur),
                            app_window: area.app_window,
                            mouse_position: *mouse_position,
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                    TrackingEventType::MouseClicked => {
                        EventWindowControls::TrackingAreaClicked(TrackingAreaClickedMessage {
                            id: area.id,
                            editor_window_uid: area.editor_window_uid,
                            duration_ms: duration_in_area_ms.map_or(0, |dur| dur),
                            app_window: area.app_window,
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                    TrackingEventType::MouseClickedOutside => {
                        EventWindowControls::TrackingAreaClickedOutside(
                            TrackingAreaClickedOutsideMessage {
                                id: area.id,
                                editor_window_uid: area.editor_window_uid,
                                app_window: area.app_window,
                            },
                        )
                        .publish_to_tauri(&self.app_handle);
                    }
                }
            }
        }
    }

    fn evaluate_event_subscriptions(
        tracking_event_type: &TrackingEventType,
        subscriptions: &TrackingEventSubscription,
    ) -> bool {
        match subscriptions {
            TrackingEventSubscription::TrackingEventTypes(subscriptions) => subscriptions
                .iter()
                .any(|subscription| subscription == tracking_event_type),
            TrackingEventSubscription::All => true,
            TrackingEventSubscription::None => false,
        }
    }

    pub fn start_event_listeners(tracking_area_manager: &Arc<Mutex<Self>>) {
        tracking_area_listener(tracking_area_manager);
        input_devices_listener(tracking_area_manager);
        xcode_listener(tracking_area_manager);
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
