use serde::{Deserialize, Serialize};
use strum::EnumIter;

use tauri::Manager;
use ts_rs::TS;

use crate::app_handle;

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
    pub fn hidden_on_temporary_hide() -> Vec<AppWindow> {
        let considered_windows = vec![
            AppWindow::Widget,
            AppWindow::CodeOverlay,
            AppWindow::Explain,
        ];

        let code_overlay_window = app_handle()
            .get_window(&AppWindow::CodeOverlay.to_string())
            .expect("Could not get CodeOverlay window");
        let code_overlay_window_monitor = code_overlay_window
            .current_monitor()
            .expect("Error when attempting to get current monitor for CodeOverlay window")
            .expect("Could not get current monitor for CodeOverlay window");

        let mut hidden_windows = vec![];
        for window in considered_windows {
            if Self::check_if_on_same_screen(window, &code_overlay_window_monitor) == Some(true) {
                hidden_windows.push(window);
            }
        }

        hidden_windows
    }

    pub fn hiddon_on_zoom_level_change() -> Vec<AppWindow> {
        vec![AppWindow::CodeOverlay, AppWindow::Explain]
    }

    pub fn hidden_on_click_widget() -> Vec<AppWindow> {
        vec![AppWindow::Main]
    }

    pub fn shown_on_focus_gained(hidden_app_windows: Option<Vec<AppWindow>>) -> Vec<AppWindow> {
        let considered_windows = vec![AppWindow::Widget, AppWindow::CodeOverlay];

        let mut shown_windows = vec![];

        if let Some(hidden_app_windows) = hidden_app_windows {
            for window in hidden_app_windows {
                if considered_windows.contains(&window) {
                    shown_windows.push(window);
                }
            }

            shown_windows
        } else {
            considered_windows
        }
    }

    pub fn shown_on_click_widget() -> Vec<AppWindow> {
        vec![AppWindow::Main]
    }

    fn check_if_on_same_screen(app_window: AppWindow, monitor: &tauri::Monitor) -> Option<bool> {
        let tauri_window = app_handle().get_window(&app_window.to_string())?;
        let tauri_window_monitor = tauri_window.current_monitor().ok()??;

        Some(monitor.position() == tauri_window_monitor.position())
    }
}

pub mod default_properties {
    use convert_case::{Case, Casing};

    use super::*;

    pub fn url(window: &AppWindow) -> String {
        format!("/{}", window.to_string().to_case(Case::Kebab))
    }

    pub fn title(window: &AppWindow) -> String {
        // ACHTUNG! Before changing the titles, check if these are used elsewhere ... at least in
        // src-tauri/src/ax_interaction/app/callbacks I am using hard coded title strings because I
        // I could not get it to work to match with the strings here.
        match window {
            AppWindow::Settings => "Pretzl - Settings".to_string(),
            AppWindow::Analytics => "Pretzl - Analytics".to_string(),
            AppWindow::Widget => "Pretzl - Widget".to_string(),
            AppWindow::Explain => "Pretzl - Explanation".to_string(),
            AppWindow::CodeOverlay => "Pretzl - CodeOverlay".to_string(),
            AppWindow::Main => "Pretzl - Main".to_string(),
        }
    }

    pub fn size(window: &AppWindow) -> (f64, f64) {
        match window {
            AppWindow::Settings => (800.0, 600.0),
            AppWindow::Analytics => (1280.0, 786.0),
            AppWindow::Widget => (48.0, 48.0),
            AppWindow::Explain => (430.0, 500.0),
            AppWindow::CodeOverlay => (1.0, 1.0),
            AppWindow::Main => (352.0, 200.0),
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
