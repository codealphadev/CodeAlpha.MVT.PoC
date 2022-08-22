#![allow(dead_code)]

use unicode_segmentation::UnicodeSegmentation;

/// A file containing utilities that are used/shared across all modules of the project

pub fn slice_string(string: &str, start: usize, length: usize) -> Option<String> {
    let graphemes = string.graphemes(true).collect::<Vec<&str>>();
    if graphemes.len() < start + length {
        return None;
    }
    Some(graphemes[start..(start + length)].join("").to_string())
}

#[cfg(test)]
mod tests_string_slice {
    use super::slice_string;

    fn test_slice_string(string: &str, start: usize, length: usize, expected: Option<&str>) {
        assert_eq!(
            slice_string(string, start, length),
            if expected.is_some() {
                Some(expected.unwrap().to_string())
            } else {
                None
            }
        );
    }

    #[test]
    fn utf8() {
        test_slice_string("Hello, World!", 3, 4, Some("lo, "));
    }

    #[test]
    fn unicode() {
        test_slice_string("H©स्lo ,👮🏻‍♀️ дorld!", 2, 9, Some("स्lo ,👮🏻‍♀️ дo"));
    }

    #[test]
    fn unicode_out_of_range() {
        test_slice_string("H©llo , дorld!", 3, 20, None);
    }

    #[test]
    fn zero_range() {
        test_slice_string("H©llo , дorld!", 3, 0, Some(""));
    }
}

pub mod geometry {

    use cocoa::appkit::CGPoint;
    use core_graphics::geometry::CGSize;
    use core_graphics_types::geometry::CGRect;
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
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
                x: rect.origin.x,
                y: rect.origin.y,
            }
        }

        pub fn as_tauri_LogicalPosition(&self) -> tauri::LogicalPosition<f64> {
            tauri::LogicalPosition {
                x: self.x,
                y: self.y,
            }
        }

        pub fn as_CGPoint(&self) -> CGPoint {
            CGPoint {
                x: self.x,
                y: self.y,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
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
                width: rect.size.width,
                height: rect.size.height,
            }
        }

        pub fn as_tauri_LogicalSize(&self) -> tauri::LogicalSize<f64> {
            tauri::LogicalSize {
                width: self.width,
                height: self.height,
            }
        }

        pub fn as_CGSize(&self) -> CGSize {
            CGSize {
                width: self.width,
                height: self.height,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
    #[ts(export, export_to = "bindings/geometry/")]
    pub struct LogicalFrame {
        pub origin: LogicalPosition,
        pub size: LogicalSize,
    }

    impl LogicalFrame {
        pub fn new(origin: LogicalPosition, size: LogicalSize) -> Self {
            Self { origin, size }
        }

        pub fn from_CGRect(rect: &CGRect) -> Self {
            Self {
                origin: LogicalPosition::from_CGRect(rect),
                size: LogicalSize::from_CGRect(rect),
            }
        }

        pub fn as_CGRect(&self) -> CGRect {
            CGRect {
                origin: self.origin.as_CGPoint(),
                size: self.size.as_CGSize(),
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
        AXEventApp,
        AXEventXcode,
        BracketHighlightResults,
        EventInputDevice,
        EventRuleExecutionState,
        EventTrackingAreas,
        EventUserInteractions,
        EventWindowControls,
        EventDocsGeneration,
        RuleResults,
    }
    impl fmt::Display for ChannelList {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ChannelList::AXEventApp => write!(f, "AXEventApp"),
                ChannelList::AXEventXcode => write!(f, "AXEventXcode"),
                ChannelList::BracketHighlightResults => write!(f, "BracketHighlightResults"),
                ChannelList::EventInputDevice => write!(f, "EventInputDevice"),
                ChannelList::EventRuleExecutionState => write!(f, "EventRuleExecutionState"),
                ChannelList::EventTrackingAreas => write!(f, "EventTrackingAreas"),
                ChannelList::EventUserInteractions => write!(f, "EventUserInteractions"),
                ChannelList::EventWindowControls => write!(f, "EventWindowControls"),
                ChannelList::EventDocsGeneration => write!(f, "EventDocsGeneration"),
                ChannelList::RuleResults => write!(f, "RuleResults"),
            }
        }
    }
}
