use std::sync::Arc;

use super::{
    features::{
        BracketHighlight, CoreEngineTrigger, DocsGenerator, Feature, FeatureBase, SwiftFormatter,
    },
    rules::{rule_base::RuleResults, RuleBase, RuleType, SwiftLinterProps},
    syntax_tree::SwiftSyntaxTree,
    utils::XcodeText,
    TextRange,
};
use crate::{
    app_handle,
    ax_interaction::models::editor::EditorShortcutPressedMessage,
    utils::{geometry::LogicalFrame, messaging::ChannelList},
    window_controls::config::AppWindow,
};
use parking_lot::Mutex;
use tauri::Manager;

#[derive(Clone, Debug)]
pub struct EditorWindowProps {
    /// The unique identifier is generated the moment we 'detect' a previously unknown editor window.
    pub id: uuid::Uuid,

    /// The reference to the AXUIElement of the editor window.
    pub uielement_hash: usize,

    /// The process identifier for the window's editor application.
    pub pid: i32,

    pub viewport_frame: LogicalFrame,

    // Range of the code document for which we can get bounds using the AX API
    pub visible_text_range: TextRange,
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

    syntax_tree: SwiftSyntaxTree,

    /// The module that manages the generation of documentation for this code document.
    docs_generator: Arc<Mutex<DocsGenerator>>,

    features: Vec<Feature>,
}

impl CodeDocument {
    pub fn new(editor_window_props: &EditorWindowProps) -> Self {
        let pid = editor_window_props.pid;
        let docs_generator_arc = Arc::new(Mutex::new(DocsGenerator::new(pid)));
        DocsGenerator::start_listener_window_control_events(&app_handle(), &docs_generator_arc);

        let mut code_document = Self {
            app_handle: app_handle(),
            rules: vec![],
            features: vec![],
            editor_window_props: editor_window_props.clone(),
            text: None,
            file_path: None,
            selected_text_range: None,
            syntax_tree: SwiftSyntaxTree::new(),
            docs_generator: docs_generator_arc,
        };

        code_document.init_features();

        code_document
    }

    pub fn init_features(&mut self) {
        self.features = [
            Feature::Formatter(SwiftFormatter::new()),
            Feature::BracketHighlighting(BracketHighlight::new()),
        ]
        .to_vec();
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

    pub fn selected_text_range(&self) -> &Option<TextRange> {
        &self.selected_text_range
    }

    pub fn update_editor_window_viewport(&mut self, viewport_frame: LogicalFrame) {
        self.editor_window_props.viewport_frame = viewport_frame;
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

        self.text = Some(new_content);
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

        let parsed_successfully = self.syntax_tree.parse(&new_content);

        // Update text content in features
        self.docs_generator.lock().update_content(&new_content);
    }

    pub fn process_rules(&mut self) {
        for rule in &mut self.rules {
            rule.run();
        }
    }

    pub fn update_docs_gen_annotation_visualization(&mut self) {
        self.docs_generator.lock().update_visualization();
    }

    pub fn deactivate_features(&mut self) {
        self.docs_generator.lock().clear_docs_generation_tasks();
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

        self.docs_generator
            .lock()
            .update_selected_text_range(&text_range);

        for feature in &mut self.features {
            feature.compute(&CoreEngineTrigger::OnTextSelectionChange);
        }
    }

    pub fn on_save(&mut self, shortcut: &EditorShortcutPressedMessage) {
        for feature in &mut self.features {
            feature.compute(&CoreEngineTrigger::OnShortcutPressed(shortcut.clone()));
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
