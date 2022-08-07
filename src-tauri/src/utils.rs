#![allow(unused)]
/// A file containing utilities that are used/shared across all modules of the project

pub mod geometry {

    use core_graphics_types::geometry::CGRect;
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
    #[ts(export, export_to = "bindings/geometry/")]
    pub struct LogicalPosition {
        /// Vertical axis value.
        pub x: f64,
        /// Horizontal axis value.
        pub y: f64,
    }

    impl LogicalPosition {
        pub fn new(x: f64, y: f64) -> Self {
            Self { x, y }
        }

        pub fn from_tauri_LogicalPosition(pos: &tauri::LogicalPosition<f64>) -> Self {
            Self { x: pos.x, y: pos.y }
        }

        pub fn from_CGRect(rect: &CGRect) -> Self {
            Self {
                x: rect.origin.x as f64,
                y: rect.origin.y as f64,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
    #[ts(export, export_to = "bindings/geometry/")]
    pub struct LogicalSize {
        /// Width.
        pub width: f64,
        /// Height.
        pub height: f64,
    }

    impl LogicalSize {
        pub fn new(width: f64, height: f64) -> Self {
            Self { width, height }
        }

        pub fn from_tauri_LogicalSize(size: &tauri::LogicalSize<f64>) -> Self {
            Self {
                width: size.width,
                height: size.height,
            }
        }

        pub fn from_CGRect(rect: &CGRect) -> Self {
            Self {
                width: rect.size.width as f64,
                height: rect.size.height as f64,
            }
        }
    }
}

pub mod messaging {
    use std::fmt;

    use serde::{Deserialize, Serialize};
    use ts_rs::TS;

    #[derive(Clone, Debug, Serialize, Deserialize, TS)]
    #[ts(export)]
    pub enum ChannelList {
        EventUserInteractions,
        EventRuleExecutionState,
        EventWindowControls,
        RuleResults,
        AXEventApp,
        AXEventReplit,
        AXEventXcode,
        EventInputDevice,
    }
    impl fmt::Display for ChannelList {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ChannelList::EventUserInteractions => write!(f, "EventUserInteractions"),
                ChannelList::EventRuleExecutionState => write!(f, "EventRuleExecutionState"),
                ChannelList::EventWindowControls => write!(f, "EventWindowControls"),
                ChannelList::RuleResults => write!(f, "RuleResults"),
                ChannelList::AXEventApp => write!(f, "AXEventApp"),
                ChannelList::AXEventReplit => write!(f, "AXEventReplit"),
                ChannelList::AXEventXcode => write!(f, "AXEventXcode"),
                ChannelList::EventInputDevice => write!(f, "EventInputDevice"),
            }
        }
    }
}
