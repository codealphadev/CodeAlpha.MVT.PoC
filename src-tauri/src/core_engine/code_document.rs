use tauri::Manager;

use crate::{utils::messaging::ChannelList, window_controls::config::AppWindow};

use super::rules::{RuleBase, RuleResults, RuleType, SearchRule, SwiftLinterRule};

pub struct EditorWindowProps {
    /// The unique identifier is generated the moment we 'detect' a previously unknown editor window.
    pub id: uuid::Uuid,

    /// The reference to the AXUIElement of the editor window.
    pub uielement_hash: usize,

    /// The process identifier for the window's editor application.
    pub pid: i32,
}

pub struct CodeDocument {
    pub app_handle: tauri::AppHandle,

    /// Properties of the editor window that contains this code document.
    editor_window_props: EditorWindowProps,

    rules: Vec<RuleType>,
}

impl CodeDocument {
    pub fn new(
        app_handle: tauri::AppHandle,
        editor_window_props: EditorWindowProps,
    ) -> CodeDocument {
        CodeDocument {
            app_handle,
            rules: vec![
                RuleType::SearchRule(SearchRule::new()),
                RuleType::SwiftLinter(SwiftLinterRule::new(editor_window_props.pid)),
            ],
            editor_window_props,
        }
    }

    pub fn editor_window_props(&self) -> &EditorWindowProps {
        &self.editor_window_props
    }

    pub fn process_rules(&mut self) {
        for rule in &mut self.rules {
            rule.run();
        }
    }

    pub fn compute_rule_visualizations(&mut self) {
        let mut rule_results = Vec::<RuleResults>::new();
        for rule in &mut self.rules {
            if let Some(rule_match_results) =
                rule.compute_rule_match_rectangles(self.editor_window_props.pid)
            {
                rule_results.push(rule_match_results);
            }
        }

        // Send to CodeOverlay window
        let _ = self.app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            &ChannelList::RuleResults.to_string(),
            &rule_results,
        );

        // Send to Main window
        let _ = self.app_handle.emit_to(
            &AppWindow::Content.to_string(),
            &ChannelList::RuleResults.to_string(),
            &rule_results,
        );
    }

    pub fn rules_mut(&mut self) -> &mut Vec<RuleType> {
        &mut self.rules
    }
}
