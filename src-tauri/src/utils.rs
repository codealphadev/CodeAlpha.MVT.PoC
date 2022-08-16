/// A file containing utilities that are used/shared across all modules of the project

pub mod grapheme {
    use unicode_segmentation::UnicodeSegmentation;

    pub fn grapheme_slice_string(string: &str, start: usize, length: usize) -> Option<String> {
        let mut i = 0;
        let mut sliced_string = String::new();
        let mut added_graphemes = 0;
        for g in string.graphemes(true) {
            if i >= start {
                sliced_string.push_str(g);
                added_graphemes += 1;
            }
            i += 1;
            if i >= start + length {
                break;
            }
        }

        if added_graphemes != length {
            return None;
        }
        return Some(sliced_string);
    }

    pub fn grapheme_vec(string: &str) -> Vec<&str> {
        string.graphemes(true).collect::<Vec<&str>>()
    }

    pub fn grapheme_count(string: &str) -> usize {
        string.graphemes(true).count()
    }
}

#[cfg(test)]
mod tests_grapheme {
    #[cfg(test)]
    mod slice_string {
        use super::super::grapheme::grapheme_slice_string;
        use pretty_assertions::assert_eq;

        fn test_fn(string: &str, start: usize, length: usize, expected: Option<&str>) {
            assert_eq!(
                grapheme_slice_string(string, start, length),
                if expected.is_some() {
                    Some(expected.unwrap().to_string())
                } else {
                    None
                }
            );
        }

        #[test]
        fn utf8() {
            test_fn("Hello, World!", 3, 4, Some("lo, "));
        }

        #[test]
        fn unicode() {
            test_fn("H©स्lo ,👮🏻‍♀️ дorld!", 2, 9, Some("स्lo ,👮🏻‍♀️ дo"));
        }

        #[test]
        fn unicode_out_of_range() {
            test_fn("H©llo , дorld!", 3, 12, None);
        }

        #[test]
        fn zero_range() {
            test_fn("H©llo , дorld!", 3, 0, Some(""));
        }
    }
}

pub mod geometry {

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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
    #[ts(export, export_to = "bindings/geometry/")]
    pub struct LogicalSize {
        /// Width.
        pub width: f64,
        /// Height.
        pub height: f64,
    }

    impl LogicalSize {
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
