use super::{syntax_tree::SwiftSyntaxTree, utils::XcodeText, TextRange};

#[derive(Clone, Debug)]
pub struct EditorWindowProps {
    /// The reference to the AXUIElement of the editor window.
    pub window_uid: usize,

    /// The process identifier for the window's editor application.
    pub pid: i32,
}

#[derive(Clone)]
pub struct CodeDocument {
    /// Properties of the editor window that contains this code document.
    editor_window_props: EditorWindowProps,

    /// The file path of the loaded code document. If it is none, then the code document
    /// loaded its contents purely through the AX API from a textarea that is not linked
    /// to a file on disk.
    file_path: Option<String>,

    // The currently selected text range in the text field.
    selected_text_range: Option<TextRange>,

    // A treesitter syntax tree
    syntax_tree: Option<SwiftSyntaxTree>,
}

impl CodeDocument {
    pub fn new(editor_window_props: &EditorWindowProps) -> Self {
        Self {
            editor_window_props: editor_window_props.clone(),
            file_path: None,
            selected_text_range: None,
            syntax_tree: None,
        }
    }

    pub fn selected_text_range(&self) -> &Option<TextRange> {
        &self.selected_text_range
    }

    pub fn syntax_tree(&self) -> Option<&SwiftSyntaxTree> {
        self.syntax_tree.as_ref()
    }

    pub fn editor_window_props(&self) -> &EditorWindowProps {
        &self.editor_window_props
    }

    pub fn text_content(&self) -> Option<&XcodeText> {
        Some(self.syntax_tree.as_ref()?.text_content())
    }

    pub fn file_path(&self) -> &Option<String> {
        &self.file_path
    }

    pub fn update_code_text(&mut self, syntax_tree: SwiftSyntaxTree, file_path: Option<String>) {
        self.syntax_tree = Some(syntax_tree);
        self.file_path = file_path;
    }

    pub fn update_selected_text_range(&mut self, selected_text_range: TextRange) {
        self.selected_text_range = Some(selected_text_range);
    }
}
            