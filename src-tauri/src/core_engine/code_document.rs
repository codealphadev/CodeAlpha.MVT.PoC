use std::sync::{Arc, Mutex};

use super::{
    events::EventRuleExecutionState,
    features::BracketHighlight,
    features::DocsGenerator,
    formatter::format_swift_file,
    rules::{RuleBase, RuleResults, RuleType, SwiftLinterProps},
    utils::XcodeText,
    TextPosition, TextRange,
};
use crate::{
    ax_interaction::{
        derive_xcode_textarea_dimensions, get_dark_mode, get_textarea_uielement,
        get_xcode_editor_content, send_event_mouse_wheel, set_selected_text_range,
        update_xcode_editor_content,
    },
    core_engine::rules::get_bounds_of_first_char_in_range,
    utils::messaging::ChannelList,
    window_controls::config::AppWindow,
};
use tauri::Manager;

#[derive(Clone, Debug)]
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

    /// The list of rules that are applied to this code document.
    rules: Vec<RuleType>,

    /// The content of the loaded code document.
    text: Option<XcodeText>,

    /// The file path of the loaded code document. If it is none, then the code document
    /// loaded its contents purely through the AX API from a textarea that is not linked
    /// to a file on disk.
    file_path: Option<String>,

    selected_text_range: Option<TextRange>,

    bracket_highlight: BracketHighlight,

    /// The module that manages the generation of documentation for this code document.
    docs_generator: Arc<Mutex<DocsGenerator>>,

    dark_mode: Option<bool>,
}

impl CodeDocument {
    pub fn new(
        app_handle: tauri::AppHandle,
        editor_window_props: EditorWindowProps,
    ) -> CodeDocument {
        let editor_window_props_clone = editor_window_props.clone();

        let pid = editor_window_props.pid;
        let docs_generator_arc = Arc::new(Mutex::new(DocsGenerator::new(pid)));
        DocsGenerator::start_listener_window_control_events(&app_handle, &docs_generator_arc);
        // TODO: Log if dark mode detection is not working properly.
        let dark_mode = get_dark_mode(pid).ok();

        let created_doc = CodeDocument {
            app_handle,
            rules: vec![],
            editor_window_props,
            text: None,
            file_path: None,
            dark_mode,
            selected_text_range: None,
            bracket_highlight: BracketHighlight::new(editor_window_props_clone.pid),
            docs_generator: docs_generator_arc,
        };
        created_doc.notify_dark_mode();
        return created_doc;
    }

    pub fn update_doc_properties(
        &mut self,
        new_content_string: &String,
        file_path: &Option<String>,
    ) {
        let new_content = &XcodeText::from_str(new_content_string);
        let is_file_path_updated = self.is_file_path_updated(file_path);
        let is_file_text_updated = self.is_file_text_updated(new_content);

        if !is_file_path_updated && !is_file_text_updated {
            // Return early if the file path and text did not change
            return;
        }

        self.text = Some(new_content.clone());
        self.file_path = file_path.clone();

        // Update Rule Properties
        for rule in self.rules_mut() {
            match rule {
                RuleType::_SwiftLinter(rule) => rule.update_properties(SwiftLinterProps {
                    file_path_as_str: file_path.clone(),
                    linter_config: None,
                    file_content: Some(new_content.clone()),
                }),
            }
        }

        // Update text content in features
        self.bracket_highlight.update_content(&new_content);

        (*self.docs_generator.lock().unwrap()).update_content(&new_content);
    }

    pub fn process_rules(&mut self) {
        for rule in &mut self.rules {
            rule.run();
        }
    }

    fn notify_dark_mode(&self) {
        // Send to CodeOverlay window
        let _ = self.app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            &ChannelList::DarkModeUpdate.to_string(),
            self.dark_mode,
        );
    }

    pub fn process_bracket_highlight(&mut self) {
        self.bracket_highlight.generate_results();

        // Send to CodeOverlay window
        let _ = self.app_handle.emit_to(
            &AppWindow::CodeOverlay.to_string(),
            &ChannelList::BracketHighlightResults.to_string(),
            &self.bracket_highlight.get_results(),
        );
    }

    pub fn update_docs_gen_annotation_visualization(&mut self) {
        (*self.docs_generator.lock().unwrap()).update_visualization();
    }

    pub fn deactivate_features(&mut self) {
        (*self.docs_generator.lock().unwrap()).clear_docs_generation_tasks();
    }

    pub fn activate_features(&mut self) {
        // Nothing yet
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

    pub fn set_selected_text_range(&mut self, index: usize, length: usize) {
        let text_range = TextRange { length, index };
        self.selected_text_range = Some(text_range);

        (*self.docs_generator.lock().unwrap()).update_selected_text_range(&text_range);

        self.bracket_highlight
            .update_selected_text_range(&text_range);

        // Check if content changed, if so, process bracket highlight
        if let (Ok(Some(content_text)), Some(text)) = (
            get_xcode_editor_content(self.editor_window_props.pid),
            self.text.as_ref(),
        ) {
            let content_text_u16 = &XcodeText::from_str(&content_text);
            if content_text_u16 != text {
                let new_content_u16 = content_text_u16;
                self.bracket_highlight.update_content(new_content_u16);
                self.process_bracket_highlight();
            }
        }
    }

    pub fn on_save(&mut self) {
        let (old_text, file_path, selected_text_range) =
            if let (Some(text), Some(file_path), Some(selected_text_range)) = (
                &self.text,
                self.file_path.clone(),
                self.selected_text_range.clone(),
            ) {
                (text, file_path, selected_text_range)
            } else {
                return;
            };
        let formatted_content = if let Some(formatted_content) = format_swift_file(file_path) {
            formatted_content
        } else {
            return;
        };

        if formatted_content == *old_text {
            return;
        }

        // Get position of selected text
        let mut scroll_delta = None;
        if let Some(editor_textarea_ui_element) =
            get_textarea_uielement(self.editor_window_props.pid)
        {
            // Get the dimensions of the textarea viewport
            if let Ok(textarea_viewport) =
                derive_xcode_textarea_dimensions(&editor_textarea_ui_element)
            {
                if let Some(bounds_of_selected_text) = get_bounds_of_first_char_in_range(
                    &selected_text_range,
                    &editor_textarea_ui_element,
                ) {
                    scroll_delta = Some(tauri::LogicalSize {
                        width: textarea_viewport.0.x - bounds_of_selected_text.origin.x,
                        height: bounds_of_selected_text.origin.y - textarea_viewport.0.y,
                    });
                }
            }
        }

        // Update content
        let formatted_content_string =
            if let Ok(formatted_content_string) = String::from_utf16(&formatted_content) {
                formatted_content_string
            } else {
                return;
            };
        if let Ok(_) =
            update_xcode_editor_content(self.editor_window_props.pid, &formatted_content_string)
        {
        } else {
            return;
        };

        // Restore cursor position
        // At this point we only place the curser a the exact same ROW | COL as before the formatting.
        if let Ok(_) = set_selected_text_range(
            self.editor_window_props.pid,
            get_new_cursor_index(&old_text, &formatted_content, selected_text_range.index),
            selected_text_range.length,
        ) {}

        // Scroll to the same position as before the formatting
        let pid_move_copy = self.editor_window_props.pid;
        if let Some(scroll_delta) = scroll_delta {
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                if let Ok(true) = send_event_mouse_wheel(pid_move_copy, scroll_delta) {}
            });
        }

        // Notifiy the frontend that the file has been formatted successfully
        EventRuleExecutionState::SwiftFormatFinished().publish_to_tauri(&self.app_handle);
    }

    pub fn rules_mut(&mut self) -> &mut Vec<RuleType> {
        &mut self.rules
    }

    fn is_file_path_updated(&self, file_path_new: &Option<String>) -> bool {
        if let Some(file_path_old) = &self.file_path {
            if let Some(file_path_new) = file_path_new {
                if file_path_old != file_path_new {
                    true
                } else {
                    false
                }
            } else {
                true
            }
        } else {
            if let Some(_) = file_path_new {
                true
            } else {
                false
            }
        }
    }

    fn is_file_text_updated(&self, file_text_new: &XcodeText) -> bool {
        if let Some(file_text_old) = &self.text {
            if file_text_old != file_text_new {
                true
            } else {
                false
            }
        } else {
            true
        }
    }
}

fn get_new_cursor_index(
    old_content: &XcodeText,
    formatted_content: &XcodeText,
    index: usize,
) -> usize {
    let mut new_index = formatted_content.len();
    if let Some(text_position) = TextPosition::from_TextIndex(old_content, index) {
        if let Some(text_index) = text_position.as_TextIndex_stay_on_line(formatted_content, true) {
            new_index = text_index;
        }
    }

    new_index
}
