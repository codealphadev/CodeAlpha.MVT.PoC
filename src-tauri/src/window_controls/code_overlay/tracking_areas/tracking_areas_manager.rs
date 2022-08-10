use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::{
    ax_interaction::{
        models::input_device::{ClickType, MouseButton, MouseClickMessage, MouseMovedMessage},
        EventInputDevice,
    },
    utils::messaging::ChannelList,
    window_controls::{
        actions::{get_position, get_size},
        config::AppWindow,
        events::{
            models::{
                TrackingAreaClickedMessage, TrackingAreaEnteredMessage, TrackingAreaExitedMessage,
            },
            EventWindowControls,
        },
    },
};

use super::{EventTrackingArea, TrackingArea, TrackingEvent, TrackingEventSubscription};

pub struct TrackingAreasManager {
    pub app_handle: tauri::AppHandle,
    tracking_areas: Vec<(TrackingArea, Option<std::time::Instant>)>,
    previous_mouse_position: Option<(f64, f64)>,
}

impl TrackingAreasManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        Self {
            app_handle: app_handle.clone(),
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

    pub fn track_mouse_position(&mut self, mouse_x: f64, mouse_y: f64) {
        self.previous_mouse_position = Some((mouse_x, mouse_y));

        // `Option<u128>` contains the duration in millis for how long the mouse has been in this tracking area.
        let mut tracking_results: Vec<(TrackingArea, TrackingEvent, Option<u64>)> = Vec::new();

        for tracking_area in self.tracking_areas.iter_mut() {
            if tracking_area
                .0
                .rectangles
                .iter()
                .any(|rectangle| rectangle.contains_point(mouse_x, mouse_y))
            {
                println!("Mouse is in tracking area: {:?}", tracking_area.0.id);
                if let Some(tracking_start) = tracking_area.1 {
                    // Case: TrackingArea was already entered before.
                    if check_overlap_with_other_app_windows(mouse_x, mouse_y) {
                        // Case: Mouse is still inside the tracking area, but an app window opens above it.
                        // We publish a MouseExited event to the tracking area.
                        tracking_results.push((
                            tracking_area.0.clone(),
                            TrackingEvent::MouseExited,
                            Some(tracking_start.elapsed().as_millis() as u64),
                        ));
                        tracking_area.1 = None;
                        continue;
                    } else {
                        tracking_results.push((
                            tracking_area.0.clone(),
                            TrackingEvent::MouseMoved,
                            Some(tracking_start.elapsed().as_millis() as u64),
                        ));
                    }
                } else {
                    // Case: TrackingArea was not entered before, start tracking the time spent in the area.
                    if check_overlap_with_other_app_windows(mouse_x, mouse_y) {
                        continue;
                    } else {
                        tracking_area.1 = Some(std::time::Instant::now());
                        tracking_results.push((
                            tracking_area.0.clone(),
                            TrackingEvent::MouseEntered,
                            None,
                        ));
                    }
                }
            } else {
                // Case: Mouse is not inside the tracking area.
                if let Some(tracking_start) = tracking_area.1 {
                    // Case: TrackingArea was entered before, now the mouse is not inside it anymore,
                    // publish a MouseExited event to the tracking area.
                    tracking_area.1 = None;
                    tracking_results.push((
                        tracking_area.0.clone(),
                        TrackingEvent::MouseExited,
                        Some(tracking_start.elapsed().as_millis() as u64),
                    ));
                }
            }
        }

        self.publish_tracking_state(&tracking_results);
    }

    pub fn track_mouse_click(&mut self, mouse_x: f64, mouse_y: f64) {
        self.previous_mouse_position = Some((mouse_x, mouse_y));

        if check_overlap_with_other_app_windows(mouse_x, mouse_y) {
            return;
        }

        // `Option<u128>` contains the duration in millis for how long the mouse has been in this tracking area.
        let mut tracking_results: Vec<(TrackingArea, TrackingEvent, Option<u64>)> = Vec::new();

        for tracking_area in self.tracking_areas.iter() {
            if tracking_area
                .0
                .rectangles
                .iter()
                .any(|rectangle| rectangle.contains_point(mouse_x, mouse_y))
            {
                println!("Mouse clicking tracking area: {:?}", tracking_area.0.id);

                if let Some(tracking_start) = tracking_area.1 {
                    tracking_results.push((
                        tracking_area.0.clone(),
                        TrackingEvent::MouseClicked,
                        Some(tracking_start.elapsed().as_millis() as u64),
                    ));
                } else {
                    tracking_results.push((
                        tracking_area.0.clone(),
                        TrackingEvent::MouseClicked,
                        None,
                    ));
                }
            }
        }

        self.publish_tracking_state(&tracking_results);
    }

    fn publish_tracking_state(
        &self,
        tracking_results: &Vec<(TrackingArea, TrackingEvent, Option<u64>)>,
    ) {
        for tracking_area in tracking_results.iter() {
            if Self::evaluate_event_subscriptions(
                &tracking_area.1,
                &tracking_area.0.event_subscriptions,
            ) {
                match tracking_area.1 {
                    TrackingEvent::MouseEntered => {
                        EventWindowControls::TrackingAreaEntered(TrackingAreaEnteredMessage {
                            id: tracking_area.0.id,
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                    TrackingEvent::MouseExited => {
                        let duration_ms = if let Some(duration_ms) = tracking_area.2 {
                            duration_ms
                        } else {
                            0
                        };

                        EventWindowControls::TrackingAreaExited(TrackingAreaExitedMessage {
                            id: tracking_area.0.id,
                            duration_ms,
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                    TrackingEvent::MouseMoved => {
                        // We don't see a use case for this at the moment. Hovering is detecting by the entering message.
                    }
                    TrackingEvent::MouseClicked => {
                        let duration_ms = if let Some(duration_ms) = tracking_area.2 {
                            duration_ms
                        } else {
                            0
                        };

                        EventWindowControls::TrackingAreaClicked(TrackingAreaClickedMessage {
                            id: tracking_area.0.id,
                            duration_ms,
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                }
            }
        }
    }

    fn evaluate_event_subscriptions(
        tracking_event: &TrackingEvent,
        subscriptions: &TrackingEventSubscription,
    ) -> bool {
        match subscriptions {
            TrackingEventSubscription::TrackingEvent(subscriptions) => subscriptions
                .iter()
                .any(|subscription| subscription == tracking_event),
            TrackingEventSubscription::All => true,
            TrackingEventSubscription::None => false,
        }
    }

    pub fn start_listener_events_input_devices(
        app_handle: &tauri::AppHandle,
        tracking_area_manager: &Arc<Mutex<Self>>,
    ) {
        let tracking_area_manager_move_copy = (tracking_area_manager).clone();
        app_handle.listen_global(ChannelList::EventInputDevice.to_string(), move |msg| {
            // Only process mouse events if the CodeOverlay window is shown.
            if !check_code_overlay_visible() {
                return;
            }

            let event_input_device: EventInputDevice =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            match event_input_device {
                EventInputDevice::MouseMoved(msg) => {
                    Self::on_mouse_moved(&tracking_area_manager_move_copy, &msg);
                }
                EventInputDevice::MouseClick(msg) => {
                    Self::on_mouse_clicked(&tracking_area_manager_move_copy, &msg);
                }
            }
        });
    }

    pub fn start_listener_tracking_areas(
        app_handle: &tauri::AppHandle,
        tracking_area_manager: &Arc<Mutex<Self>>,
    ) {
        let tracking_area_manager_move_copy = (tracking_area_manager).clone();
        app_handle.listen_global(ChannelList::EventTrackingAreas.to_string(), move |msg| {
            let event_tracking_areas: EventTrackingArea =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            let tracking_area_manager = &mut *(match tracking_area_manager_move_copy.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            });

            match event_tracking_areas {
                EventTrackingArea::Add(msg) => {
                    tracking_area_manager.add_tracking_areas(msg);
                }
                EventTrackingArea::Remove(msg) => {
                    tracking_area_manager.remove_tracking_areas(msg);
                }
                EventTrackingArea::Reset() => {
                    tracking_area_manager.reset_tracking_areas();
                }
                EventTrackingArea::Update(msg) => {
                    tracking_area_manager.update_tracking_areas(msg);
                }
                EventTrackingArea::Replace(msg) => {
                    tracking_area_manager.replace_tracking_areas(msg);
                }
            }
        });
    }

    fn on_mouse_moved(tracking_area_manager_arc: &Arc<Mutex<Self>>, move_msg: &MouseMovedMessage) {
        let tracking_area_manager = &mut *(match tracking_area_manager_arc.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        });

        tracking_area_manager
            .track_mouse_position(move_msg.cursor_position.x, move_msg.cursor_position.y);
    }

    fn on_mouse_clicked(
        tracking_area_manager_arc: &Arc<Mutex<Self>>,
        click_msg: &MouseClickMessage,
    ) {
        let tracking_area_manager = &mut *(match tracking_area_manager_arc.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        });

        if click_msg.button == MouseButton::Left && click_msg.click_type == ClickType::Down {
            tracking_area_manager
                .track_mouse_click(click_msg.cursor_position.x, click_msg.cursor_position.y);
        }
    }
}

fn check_code_overlay_visible() -> bool {
    use crate::app_handle;

    if let Some(window) = app_handle().get_window(&AppWindow::CodeOverlay.to_string()) {
        if let Ok(visible) = window.is_visible() {
            if visible {
                return true;
            }
        }
    }

    false
}

fn check_overlap_with_other_app_windows(mouse_x: f64, mouse_y: f64) -> bool {
    use crate::app_handle;
    use strum::IntoEnumIterator;

    for app_window in AppWindow::iter() {
        if app_window == AppWindow::CodeOverlay {
            continue;
        }

        if let (Ok(origin), Ok(size)) = (
            get_position(&app_handle(), app_window),
            get_size(&app_handle(), app_window),
        ) {
            if mouse_x >= origin.x
                && mouse_x <= origin.x + size.width
                && mouse_y >= origin.y
                && mouse_y <= origin.y + size.height
            {
                return true;
            }
        }
    }

    false
}
