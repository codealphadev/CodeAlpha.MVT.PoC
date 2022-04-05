use serde::{Deserialize, Serialize};

// This file contains the list of all the app windows and their initial sizes and features
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AppWindow {
    Settings,
    Analytics,
    Widget,
    Content,
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
        let master = format!("{}{}", r"\", window.to_string().to_lowercase());

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

            AppWindow::None => "".to_string(),
        }
    }

    pub fn title(window: &AppWindow) -> String {
        match window {
            AppWindow::Settings => "CodeAlpha - Settings".to_string(),
            AppWindow::Analytics => "CodeAlpha - Analytics".to_string(),
            AppWindow::Widget => "CodeAlpha".to_string(),
            AppWindow::Content => "CodeAlpha - Guide".to_string(),
            AppWindow::None => "".to_string(),
        }
    }

    pub fn size(window: &AppWindow) -> (f64, f64) {
        match window {
            AppWindow::Settings => (800.0, 600.0),
            AppWindow::Analytics => (1280.0, 786.0),
            AppWindow::Widget => (48.0, 48.0),
            AppWindow::Content => (322.0, 316.0),
            AppWindow::None => (0.0, 0.0),
        }
    }

    // If we tie windows together as parent/child, they will be moved together.
    // For now, only the content window is supposed to have the Widget as a parent.
    pub fn parent_window(window: &AppWindow) -> AppWindow {
        match window {
            AppWindow::Settings => AppWindow::None,
            AppWindow::Analytics => AppWindow::None,
            AppWindow::Widget => AppWindow::None,
            AppWindow::Content => AppWindow::Widget,
            AppWindow::None => AppWindow::None,
        }
    }

    pub fn is_resizable(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::None => false,
        }
    }
    pub fn is_transparent(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => false,
            AppWindow::Analytics => false,
            AppWindow::Widget => true,
            AppWindow::Content => true,
            AppWindow::None => false,
        }
    }

    pub fn has_decorations(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::None => false,
        }
    }

    pub fn is_visible(window: &AppWindow) -> bool {
        match window {
            AppWindow::Settings => true,
            AppWindow::Analytics => true,
            AppWindow::Widget => false,
            AppWindow::Content => false,
            AppWindow::None => true,
        }
    }
}
