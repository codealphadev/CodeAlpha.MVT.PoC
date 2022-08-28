use serde::{Deserialize, Serialize};

use crate::{core_engine::WindowUid, utils::geometry::LogicalFrame};

/// A TrackingArea can subscribe to any number of TrackingEventTypes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TrackingEventSubscription {
    TrackingEventTypes(Vec<TrackingEventType>),
    All,
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TrackingEventType {
    MouseEntered,
    MouseExited,
    MouseMoved,
    MouseClicked,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackingArea {
    pub id: uuid::Uuid,
    pub window_uid: WindowUid,
    pub rectangles: Vec<LogicalFrame>,
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
