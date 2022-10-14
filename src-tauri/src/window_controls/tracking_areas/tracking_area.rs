use serde::{Deserialize, Serialize};

use crate::{
    core_engine::EditorWindowUid,
    utils::geometry::{LogicalFrame, LogicalPosition},
    window_controls::{config::AppWindow, utils::get_position},
};

/// A TrackingArea can subscribe to any number of TrackingEventTypes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TrackingEvents {
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
pub enum TrackingEventSubscriber {
    AppWindow(AppWindow),
    Backend,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TrackingAreaType {
    Generic,              // A generic tracking area that can be used for any purpose.
    AppWindow(AppWindow), // A tracking area that is used to track the AppWindows (CodeOverlay, Explain, Main, Widget, etc.).
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TrackingArea {
    pub id: uuid::Uuid,
    pub window_uid: EditorWindowUid, // Set to '0' if not associated with an editor window.
    pub area_type: TrackingAreaType,
    pub app_window: AppWindow, // The associated AppWindow - it's the window which receives notifications.
    pub rectangle: LogicalFrame, // In coordinates relative to the containing app window.
    pub events: TrackingEvents,
    pub subscriber: Vec<TrackingEventSubscriber>,
}

impl TrackingArea {
    pub fn update(&mut self, updated_tracking_area: &TrackingArea) {
        if updated_tracking_area.id != self.id {
            return;
        }
        self.rectangle = updated_tracking_area.rectangle.clone();
        self.events = updated_tracking_area.events.clone();
        self.subscriber = updated_tracking_area.subscriber.clone();
    }

    pub fn rect_as_global(&self) -> LogicalFrame {
        self.rectangle.to_global(&self.global_origin())
    }

    pub fn global_origin(&self) -> LogicalPosition {
        get_position(self.app_window).expect("TrackingArea: Failed to get window position")
    }
}
