use serde::{Deserialize, Serialize};
use tauri::async_runtime::block_on;
/// A file containing utilities that are used/shared across all modules of the project

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
                ChannelList::RuleResults => write!(f, "RuleResults"),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintlifyResponse {
    docstring: Option<String>,
    feedbackId: String,
    position: String,
    preview: String,
    shouldShowFeedback: bool,
    shouldShowShare: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintlifyRequest {
    apiKey: String,
    code: String,
    context: Option<String>,
}

pub fn get_mintlify_documentation(
    code: String,
    context: Option<String>,
) -> Option<MintlifyResponse> {
    let handle = mintlify_documentation(code, context);
    let mintlify_response: Result<MintlifyResponse, reqwest::Error> = block_on(handle);

    if let Ok(mintlify_response) = mintlify_response {
        Some(mintlify_response)
    } else {
        None
    }
}

async fn mintlify_documentation(
    code: String,
    context: Option<String>,
) -> Result<MintlifyResponse, reqwest::Error> {
    let req_body = MintlifyRequest {
        apiKey: "-RWsev7z_qgP!Qinp_8cbmwgP9jg4AQBkfz".to_string(),
        code,
        context,
    };

    let response = reqwest::Client::new()
        .post("https://europe-west1-codealpha-analyze-text-dev.cloudfunctions.net/analyze-code")
        .json(&req_body)
        .send()
        .await?;
    let parsed_response = response.json().await?;
    Ok(parsed_response)
}

#[cfg(test)]
mod tests_mintlify {

    use super::get_mintlify_documentation;

    #[test]
    fn test_get_mintlify_documentation() {
        let resp = get_mintlify_documentation(
            "print(\"Hello World\")".to_string(),
            Some("print(\"Hello World\")".to_string()),
        );
        assert!(resp.is_some());
    }
}
