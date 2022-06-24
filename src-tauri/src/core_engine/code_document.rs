use super::rules::search_and_replace::SearchRule;

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

    /// Rule engine for basic search and replace
    search_and_replace_rule: SearchRule,
}

impl CodeDocument {
    pub fn new(
        app_handle: tauri::AppHandle,
        editor_window_props: EditorWindowProps,
        search_and_replace_rule: SearchRule,
    ) -> CodeDocument {
        CodeDocument {
            app_handle,
            editor_window_props,
            search_and_replace_rule,
        }
    }

    pub fn editor_window_props(&self) -> &EditorWindowProps {
        &self.editor_window_props
    }

    pub fn search_and_replace_rule(&self) -> &SearchRule {
        &self.search_and_replace_rule
    }

    pub fn compute_search_and_replace_rule(
        &mut self,
        content_str: Option<String>,
        search_str: Option<String>,
    ) {
        self.search_and_replace_rule.run(content_str, search_str);
    }

    pub fn compute_search_and_replace_rule_visualization(&mut self) {
        self.search_and_replace_rule.compute_match_boundaries(
            self.editor_window_props.pid,
            Some(self.editor_window_props.uielement_hash),
        );

        // Publish to Frontend using tauri app handle
    }
}
