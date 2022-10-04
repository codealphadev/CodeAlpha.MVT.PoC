use serde::{Deserialize, Serialize};
use strum::EnumIter;

use ts_rs::TS;

#[derive(EnumIter, Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, TS)]
#[ts(export)]
pub enum AppWindow {
    Settings,
    Analytics,
    Widget,
    Explain,
    CodeOverlay,
    Main,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum WindowLevel {
    CodeOverlay = 5,
    Widget = 6,
    Main = 7,
    FloatingCard = 10,
    ConfigWindow = 20,
}

impl std::fmt::Display for AppWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl AppWindow {
    pub fn hidden_on_core_engine_inactive() -> Vec<AppWindow> {
        vec![AppWindow::CodeOverlay, AppWindow::Explain]
    }

    pub fn hidden_on_focus_lost() -> Vec<AppWindow> {
        vec![
            AppWindow::Widget,
            AppWindow::CodeOverlay,
            AppWindow::Explain,
        ]
    }
    pub fn hiddon_on_zoom_level_change() -> Vec<AppWindow> {
        vec![AppWindow::CodeOverlay, AppWindow::Explain]
    }

    pub fn hidden_on_click_widget() -> Vec<AppWindow> {
        vec![AppWindow::Main]
    }

    pub fn shown_on_focus_gained() -> Vec<AppWindow> {
        vec![AppWindow::Widget, AppWindow::CodeOverlay]
    }

    pub fn shown_on_core_engine_activated() -> Vec<AppWindow> {
        vec![AppWindow::Widget, AppWindow::CodeOverlay]
    }

    pub fn shown_on_click_widget() -> Vec<AppWindow> {
        vec![AppWindow::Main]
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
            AppWindow::Explain => {
                format!("{}{}", r"\", AppWindow::Explain.to_string().to_lowercase())
            }
            AppWindow::CodeOverlay => {
                format!(
                    "{}{}",
                    r"\",
                    AppWindow::CodeOverlay.to_string().to_lowercase()
                )
            }
            AppWindow::Main => format!("{}{}", r"\", AppWindow::Main.to_string().to_lowercase()),
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
            AppWindow::Explain => "CodeAlpha - Explanation".to_string(),
            AppWindow::CodeOverlay => "CodeAlpha - CodeOverlay".to_string(),
            AppWindow::Main => "CodeAlpha - Main".to_string(),
        }
    }

    pub fn size(window: &AppWindow) -> (f64, f64) {
        match window {
            AppWindow::Settings => (800.0, 600.0),
            AppWindow::Analytics => (1280.0, 786.0),
            AppWindow::Widget => (48.0, 48.0),
            AppWindow::Explain => (368.0, 500.0),
            AppWindow::CodeOverlay => (1.0, 1.0),
            AppWindow::Main => (320.0, 200.0),
        }
    }

    pub fn is_resizable(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Explain => false,
            AppWindow::CodeOverlay => false,
            AppWindow::Main => false,
        }
    }
    pub fn is_transparent(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => false,
            AppWindow::Widget => true,
            AppWindow::Explain => true,
            AppWindow::CodeOverlay => true,
            AppWindow::Main => true,
        }
    }

    pub fn has_decorations(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Explain => false,
            AppWindow::CodeOverlay => false,
            AppWindow::Main => false,
        }
    }

    pub fn is_visible(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Explain => false,
            AppWindow::CodeOverlay => false,
            AppWindow::Main => false,
        }
    }

    pub fn is_always_on_top(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => false,
            AppWindow::Widget => true,
            AppWindow::Explain => true,
            AppWindow::CodeOverlay => true,
            AppWindow::Main => true,
        }
    }

    pub fn skip_taskbar(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => true,
            AppWindow::Explain => true,
            AppWindow::CodeOverlay => true,
            AppWindow::Main => true,
        }
    }
}
