use crate::{
    core_engine::types::{MatchRectangle, RuleMatchCategory},
    window_controls::events::{
        models::{
            TrackingAreaClickedMessage, TrackingAreaEnteredMessage, TrackingAreaExitedMessage,
        },
        EventWindowControls,
    },
};

enum TrackingEvent {
    MouseEntered,
    MouseExited,
    MouseMoved,
    MouseClicked,
}

pub struct TrackingArea {
    id: uuid::Uuid,
    category: RuleMatchCategory,
    rectangles: Vec<MatchRectangle>,
    when_entered: Option<std::time::Instant>,
    tracking_event: Option<TrackingEvent>,
}

impl TrackingArea {
    pub fn new(
        id: uuid::Uuid,
        category: RuleMatchCategory,
        rectangles: Vec<MatchRectangle>,
    ) -> Self {
        Self {
            id,
            category,
            rectangles,
            when_entered: None,
            tracking_event: None,
        }
    }
}

pub struct TrackingAreasManager {
    pub app_handle: tauri::AppHandle,
    tracking_areas: Vec<TrackingArea>,
    previous_mouse_position: Option<(i32, i32)>,
}

impl TrackingAreasManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        Self {
            app_handle: app_handle.clone(),
            tracking_areas: Vec::new(),
            previous_mouse_position: None,
        }
    }

    pub fn tracking_areas(&mut self) -> &mut Vec<TrackingArea> {
        &mut self.tracking_areas
    }

    pub fn update_tracking_areas(&mut self, tracking_areas: Vec<TrackingArea>) {
        self.tracking_areas = tracking_areas;
    }

    pub fn track_mouse_position(&mut self, mouse_x: i32, mouse_y: i32) {
        self.previous_mouse_position = Some((mouse_x, mouse_y));

        for tracking_area in self.tracking_areas.iter_mut() {
            if tracking_area
                .rectangles
                .iter()
                .any(|rectangle| rectangle.contains_point(mouse_x, mouse_y))
            {
                if tracking_area.when_entered.is_none() {
                    tracking_area.when_entered = Some(std::time::Instant::now());
                    tracking_area.tracking_event = Some(TrackingEvent::MouseEntered);
                } else {
                    tracking_area.tracking_event = Some(TrackingEvent::MouseMoved);
                }
            } else {
                if tracking_area.when_entered.is_some() {
                    tracking_area.when_entered = None;
                    tracking_area.tracking_event = Some(TrackingEvent::MouseExited);
                }
            }
        }

        self.publish_tracking_state();
        self.reset_tracking_events();
    }

    pub fn track_mouse_click(&mut self, mouse_x: i32, mouse_y: i32) {
        for tracking_area in self.tracking_areas.iter_mut() {
            if tracking_area
                .rectangles
                .iter()
                .any(|rectangle| rectangle.contains_point(mouse_x, mouse_y))
            {
                tracking_area.tracking_event = Some(TrackingEvent::MouseClicked);
            }
        }

        self.publish_tracking_state();
        self.reset_tracking_events();
    }

    fn reset_tracking_events(&mut self) {
        for tracking_area in self.tracking_areas.iter_mut() {
            tracking_area.tracking_event = None;
        }
    }

    fn publish_tracking_state(&mut self) {
        for tracking_area in self.tracking_areas.iter() {
            if tracking_area.tracking_event.is_some() {
                match tracking_area.tracking_event.as_ref().unwrap() {
                    TrackingEvent::MouseEntered => {
                        EventWindowControls::TrackingAreaEntered(TrackingAreaEnteredMessage {
                            id: tracking_area.id,
                            category: tracking_area.category.clone(),
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                    TrackingEvent::MouseExited => {
                        EventWindowControls::TrackingAreaExited(TrackingAreaExitedMessage {
                            id: tracking_area.id,
                            category: tracking_area.category.clone(),
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                    TrackingEvent::MouseMoved => {
                        // do nothing
                        println!(
                            "MouseMoved in tracking area: {:#?}",
                            tracking_area.rectangles
                        );
                    }
                    TrackingEvent::MouseClicked => {
                        EventWindowControls::TrackingAreaClicked(TrackingAreaClickedMessage {
                            id: tracking_area.id,
                            category: tracking_area.category.clone(),
                        })
                        .publish_to_tauri(&self.app_handle);
                    }
                }
            }
        }
    }
}
