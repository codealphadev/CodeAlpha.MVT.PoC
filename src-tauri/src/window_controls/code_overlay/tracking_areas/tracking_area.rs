use serde::{Deserialize, Serialize};

use crate::core_engine::types::MatchRectangle;

/// A TrackingArea can subscribe to any number of TrackingEvents.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TrackingEventSubscription {
    TrackingEvent(Vec<TrackingEvent>),
    All,
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TrackingEvent {
    MouseEntered,
    MouseExited,
    MouseMoved,
    MouseClicked,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackingArea {
    pub id: uuid::Uuid,
    pub rectangles: Vec<MatchRectangle>,
    pub event_subscriptions: TrackingEventSubscription,
}

impl TrackingArea {
    pub fn update(&mut self, updated_tracking_area: &TrackingArea) {
        if updated_tracking_area.id != self.id {
            return;
        }
        self.rectangles = updated_tracking_area.rectangles.clone();
        self.event_subscriptions = updated_tracking_area.event_subscriptions.clone();
    }
}
