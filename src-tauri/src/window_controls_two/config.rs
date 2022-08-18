use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(EnumIter, Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AppWindow {
    Settings,
    Analytics,
    Widget,
    Content,
    Repair,
    CodeOverlay,
    None,
}

impl std::fmt::Display for AppWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub mod default_properties {
    use super::*;

    pub fn url(window: &AppWindow) -> String {
        match window {
            AppWindow::Settings => {
                format!("{}{}", r"\", AppWindow::Settings.to_string().to_lowercase())
            }

            AppWindow::Analytics => format!(
                "{}{}",
                r"\",
                AppWindow::Analytics.to_string().to_lowercase()
            ),
            AppWindow::Widget => {
                format!("{}{}", r"\", AppWindow::Widget.to_string().to_lowercase())
            }

            AppWindow::Content => {
                format!("{}{}", r"\", AppWindow::Content.to_string().to_lowercase())
            }
            AppWindow::Repair => {
                format!("{}{}", r"\", AppWindow::Repair.to_string().to_lowercase())
            }
            AppWindow::None => "".to_string(),
            AppWindow::CodeOverlay => {
                format!(
                    "{}{}",
                    r"\",
                    AppWindow::CodeOverlay.to_string().to_lowercase()
                )
            }
        }
    }

    pub fn title(window: &AppWindow) -> String {
        // ACHTUNG! Before changing the titles, check if these are used elsewhere ... at least in
        // src-tauri/src/ax_interaction/app/callbacks I am using hard coded title strings because I
        // I could not get it to work to match with the strings here.
        match window {
            AppWindow::Settings => "CodeAlpha - Settings".to_string(),
            AppWindow::Analytics => "CodeAlpha - Analytics".to_string(),
            AppWindow::Widget => "CodeAlpha - Widget".to_string(),
            AppWindow::Content => "CodeAlpha - Guide".to_string(),
            AppWindow::Repair => "CodeAlpha - Explanation".to_string(),
            AppWindow::None => "".to_string(),
            AppWindow::CodeOverlay => "CodeAlpha - CodeOverlay".to_string(),
        }
    }

    pub fn size(window: &AppWindow) -> (f64, f64) {
        match window {
            AppWindow::Settings => (800.0, 600.0),
            AppWindow::Analytics => (1280.0, 786.0),
            AppWindow::Widget => (48.0, 48.0),
            AppWindow::Content => (322.0, 398.0),
            AppWindow::Repair => (513.0, 500.0),
            AppWindow::None => (0.0, 0.0),
            AppWindow::CodeOverlay => (0.0, 0.0),
        }
    }

    pub fn is_resizable(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::Repair => false,
            AppWindow::None => false,
            AppWindow::CodeOverlay => false,
        }
    }
    pub fn is_transparent(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => false,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::Repair => true,
            AppWindow::None => false,
            AppWindow::CodeOverlay => true,
        }
    }

    pub fn has_decorations(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::Repair => false,
            AppWindow::None => false,
            AppWindow::CodeOverlay => false,
        }
    }

    pub fn is_visible(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::Repair => true,
            AppWindow::None => true,
            AppWindow::CodeOverlay => false,
        }
    }

    pub fn is_always_on_top(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => false,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::Repair => true,
            AppWindow::None => false,
            AppWindow::CodeOverlay => true,
        }
    }

    pub fn skip_taskbar(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::Repair => true,
            AppWindow::None => true,
            AppWindow::CodeOverlay => true,
        }
    }
}