use super::{
    rules::{rule_base::RuleResults, RuleBase, RuleType, SwiftLinterProps},
    syntax_tree::SwiftSyntaxTree,
    utils::XcodeText,
    TextRange,
};
use crate::{
    app_handle,
    platform::macos::{get_textarea_content, GetVia},
    utils::messaging::ChannelList,
    window_controls::config::AppWindow,
};
use tauri::Manager;

#[derive(Clone, Debug)]
pub struct EditorWindowProps {
    /// The reference to the AXUIElement of the editor window.
    pub window_uid: usize,

    /// The process identifier for the window's editor application.
    pub pid: i32,
}

pub struct CodeDocument {
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

    // The currently selected text range in the text field.
    selected_text_range: Option<TextRange>,

    // A treesitter syntax tree
    syntax_tree: SwiftSyntaxTree,
}

impl CodeDocument {
    pub fn new(editor_window_props: &EditorWindowProps) -> Self {
        Self {
            rules: vec![],
            editor_window_props: editor_window_props.clone(),
            text: None,
            file_path: None,
            selected_text_range: None,
            syntax_tree: SwiftSyntaxTree::new(),
        }
    }

    pub fn selected_text_range(&self) -> &Option<TextRange> {
        &self.selected_text_range
    }

    pub fn syntax_tree(&self) -> &SwiftSyntaxTree {
        &self.syntax_tree
    }

    pub fn editor_window_props(&self) -> &EditorWindowProps {
        &self.editor_window_props
    }

    pub fn text_content(&self) -> &Option<XcodeText> {
        &self.text
    }

    pub fn file_path(&self) -> &Option<String> {
        &self.file_path
    }

    pub fn update_doc_properties(
        &mut self,
        new_content_string: &String,
        file_path: &Option<String>,
    ) {
        let new_content = XcodeText::from_str(new_content_string);
        let is_file_path_updated = self.is_file_path_updated(file_path);
        let is_file_text_updated = self.is_file_text_updated(&new_content);

        if !is_file_path_updated && !is_file_text_updated {
            // Return early if the file path and text did not change
            return;
        }

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

        self.file_path = file_path.clone();
        self.syntax_tree.parse(&new_content);
        self.text = Some(new_content);
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
        let _ = app_handle().emit_to(
            &AppWindow::CodeOverlay.to_string(),
            &ChannelList::RuleResults.to_string(),
            &rule_results,
        );

        // Send to Main window
        let _ = app_handle().emit_to(
            &AppWindow::Content.to_string(),
            &ChannelList::RuleResults.to_string(),
            &rule_results,
        );
    }

    pub fn set_selected_text_range(&mut self, text_range: &TextRange, double_check: bool) {
        // Check if content changed at same time - this is needed for the case that selected text range
        // is being delivered before text content change message
        if double_check {
            if let (Ok(content_text), Some(text)) = (
                get_textarea_content(&GetVia::Pid(self.editor_window_props.pid)),
                self.text.as_ref(),
            ) {
                let content_text_u16 = &XcodeText::from_str(&content_text);
                self.selected_text_range = Some(text_range.clone());

                if content_text_u16 != text {
                    self.syntax_tree.parse(content_text_u16);
                    self.text = Some(content_text_u16.clone());
                }
            }
        } else {
            self.selected_text_range = Some(text_range.clone());
        }
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

#[cfg(test)]
pub mod tests {
    use crate::core_engine::TextRange;

    use super::{CodeDocument, EditorWindowProps};

    pub fn get_code_documention_mock_object(
        default_code_snippet: Option<String>,
        default_file_path: Option<String>,
        default_selected_text_range: Option<TextRange>,
    ) -> CodeDocument {
        let code_snippet;
        if default_code_snippet.is_some() {
            code_snippet = default_code_snippet.unwrap();
        } else {
            code_snippet = r#"
            open class SystemWideElement: UIElement {
                fileprivate convenience init() {
                    self.init(AXUIElementCreateSystemWide())
                }
            
                open func elementAtPosition(_ x: Float, _ y: Float) throws -> UIElement? {
                    return try super.elementAtPosition(x, y)
                }
            }"#
            .to_string();
        }

        let selected_text_range;
        if default_selected_text_range.is_some() {
            selected_text_range = default_selected_text_range.unwrap();
        } else {
            selected_text_range = TextRange {
                index: 110, // this is right before `self.init(AXUIElementCreateSystemWide())`
                length: 0,
            };
        }

        let file_path;
        if default_file_path.is_some() {
            file_path = default_file_path;
        } else {
            file_path = None;
        }

        let editor_window = EditorWindowProps {
            window_uid: 1,
            pid: 1,
        };

        let mut code_doc = CodeDocument::new(&editor_window);
        code_doc.update_doc_properties(&code_snippet, &file_path);
        code_doc.set_selected_text_range(&selected_text_range, false);

        code_doc
    }
}
