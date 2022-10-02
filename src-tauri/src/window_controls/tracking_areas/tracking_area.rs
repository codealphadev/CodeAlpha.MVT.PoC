use serde::{Deserialize, Serialize};

use crate::{
    core_engine::EditorWindowUid,
    utils::geometry::{LogicalFrame, LogicalPosition},
    window_controls::{config::AppWindow, utils::get_position},
};

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
    MouseOver,
    MouseClicked,
    MouseClickedOutside,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TrackingArea {
    pub id: uuid::Uuid,
    pub editor_window_uid: EditorWindowUid, // Set to '0' if not associated with an editor window.
    pub app_window: AppWindow,
    pub rectangle: LogicalFrame, // In coordinates relative to the containing app window.
    pub event_subscriptions: TrackingEventSubscription,
}

impl TrackingArea {
    pub fn update(&mut self, updated_tracking_area: &TrackingArea) {
        if updated_tracking_area.id != self.id {
            return;
        }
        self.rectangle = updated_tracking_area.rectangle.clone();
        self.event_subscriptions = updated_tracking_area.event_subscriptions.clone();
    }

    pub fn eq_props(&self, other: &Self) -> bool {
        self.editor_window_uid == other.editor_window_uid
            && self.rectangle == other.rectangle
            && self.event_subscriptions == other.event_subscriptions
    }

    pub fn rect_as_global(&self) -> LogicalFrame {
        self.rectangle.to_global(&self.global_origin())
    }

    pub fn global_origin(&self) -> LogicalPosition {
        get_position(self.app_window).expect("TrackingArea: Failed to get window position")
    }
}
