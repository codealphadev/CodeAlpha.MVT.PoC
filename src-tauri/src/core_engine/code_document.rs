use tauri::Manager;

use crate::{
    ax_interaction::{set_selected_text_range, update_xcode_editor_content},
    utils::messaging::ChannelList,
    window_controls::config::AppWindow,
};

use super::{
    formatter::get_format_swift_file,
    rules::{
        RuleBase, RuleResults, RuleType, SearchRule, SearchRuleProps, SwiftLinterProps,
        SwiftLinterRule, TextPosition, TextRange,
    },
    syntax_tree::SwiftSyntaxTree,
};

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
    text: String,

    /// The file path of the loaded code document. If it is none, then the code document
    /// loaded its contents purely through the AX API from a textarea that is not linked
    /// to a file on disk.
    file_path: Option<String>,

    /// The syntax tree of the loaded code document.
    swift_syntax_tree: SwiftSyntaxTree,

    selected_text_range: Option<TextRange>,
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
            text: "".to_string(),
            file_path: None,
            swift_syntax_tree: SwiftSyntaxTree::new(),
            selected_text_range: None,
        }
    }

    pub fn editor_window_props(&self) -> &EditorWindowProps {
        &self.editor_window_props
    }

    pub fn update_doc_properties(&mut self, text: &String, file_path: &Option<String>) {
        let text_updated = if text != &self.text { true } else { false };
        let file_path_updated = self.file_path_updated(file_path);

        if file_path_updated {
            self.swift_syntax_tree.reset();
            self.file_path = file_path.clone();
        }

        if text_updated {
            self.text = text.clone();
        }

        // rerun syntax tree parser
        self.swift_syntax_tree.parse(&self.text);
        let new_tree = self.swift_syntax_tree.get_tree();
        let new_content = self.text.clone();

        for rule in self.rules_mut() {
            match rule {
                RuleType::SearchRule(rule) => rule.update_properties(SearchRuleProps {
                    search_str: None,
                    content: Some(text.clone()),
                }),
                RuleType::SwiftLinter(rule) => rule.update_properties(SwiftLinterProps {
                    file_path_as_str: file_path.clone(),
                    linter_config: None,
                    swift_syntax_tree: new_tree.clone(),
                    file_content: Some(new_content.clone()),
                }),
            }
        }
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

    pub fn set_selected_text_range(&mut self, index: usize, length: usize) {
        self.selected_text_range = Some(TextRange { length, index });
    }

    pub fn on_save(&mut self) {
        if let (Some(file_path), Some(selected_text_range)) =
            (self.file_path.clone(), self.selected_text_range.clone())
        {
            let formatted_content_option =
                get_format_swift_file(file_path, selected_text_range.clone(), self.text.clone());
            if let Some(formatted_content) = formatted_content_option {
                if formatted_content.content == self.text.clone() {
                    return;
                }
                // Update content
                if let Ok(_) = update_xcode_editor_content(
                    self.editor_window_props.pid,
                    &formatted_content.content,
                ) {
                } else {
                    assert!(false, "Could not update Xcode editor content");
                }

                // TODO: Restore scroll position

                // Restore cursor position
                // Keep cursor on same line as before or end of file
                let new_index = get_new_cursor_index(
                    &self.text,
                    &formatted_content.content,
                    selected_text_range.index,
                );

                if let Ok(_) = set_selected_text_range(
                    self.editor_window_props.pid,
                    new_index,
                    selected_text_range.length,
                ) {
                } else {
                    assert!(false, "Could not set cursor position");
                }
            }
        }
    }

    pub fn rules_mut(&mut self) -> &mut Vec<RuleType> {
        &mut self.rules
    }

    fn file_path_updated(&self, file_path_new: &Option<String>) -> bool {
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
}

fn get_new_cursor_index(old_content: &String, formatted_content: &String, index: usize) -> usize {
    let mut new_index = formatted_content.len();
    if let Some(text_position) = TextPosition::from_TextIndex(old_content, index) {
        if let Some(text_index) = text_position.as_TextIndex_stay_on_line(formatted_content, true) {
            new_index = text_index;
        }
    }

    new_index
}
