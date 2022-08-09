use std::{
    io::Write,
    process::{Command, Stdio},
};
use tree_sitter::{InputEdit, Parser, Tree};

use super::detect_input_edits;

pub struct SwiftSyntaxTree {
    tree_sitter_parser: Parser,
    tree_sitter_tree: Option<Tree>,
    content: Option<String>,
}

impl SwiftSyntaxTree {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_swift::language())
            .expect("Swift Language not found");

        Self {
            tree_sitter_parser: parser,
            tree_sitter_tree: None,
            content: None,
        }
    }

    pub fn reset(&mut self) {
        self.tree_sitter_tree = None;
        self.content = None;
    }

    pub fn parse(&mut self, content: &String) -> bool {
        // If there already exists a tree, we are updating it with the new content.
        // We assume the content is an updated version of the content parsed before.

        let updated_tree: Option<Tree>;
        if let (Some(old_tree), Some(old_content)) = (&mut self.tree_sitter_tree, &self.content) {
            // Determine the edits made to the code document.
            let mut input_edits: Vec<InputEdit> = detect_input_edits(old_content, content);

            // Sort input_edits by start_byte in descending order before applying them to the tree.
            input_edits.sort_by(|a, b| b.start_byte.cmp(&a.start_byte));

            // Apply the sorted edits to the old tree.
            for edit in input_edits.iter() {
                old_tree.edit(edit);
            }

            updated_tree = self.tree_sitter_parser.parse(content, Some(old_tree));
        } else {
            updated_tree = self.tree_sitter_parser.parse(content, None);
        }

        if updated_tree.is_some() {
            self.tree_sitter_tree = updated_tree;
            self.content = Some(content.to_owned());
            return true;
        } else {
            return false;
        }
    }

    pub fn get_tree_copy(&self) -> Option<Tree> {
        self.tree_sitter_tree.clone()
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree_sitter_tree.as_ref()
    }

    #[allow(dead_code)]
    fn start_logging(parser: &mut Parser, path: &str) -> std::io::Result<()> {
        let mut dot_file_path = path.to_string().clone();
        dot_file_path.push_str("tree.dot");

        let mut html_file_path = path.to_string().clone();
        html_file_path.push_str("tree.html");

        // let mut dot_file = std::fs::File::create(dot_file_path)?;
        // self.tree_sitter_parser.print_dot_graphs(&dot_file);

        let mut html_file = std::fs::File::create(html_file_path)?;

        let html_header: &[u8] = b"<!DOCTYPE html>\n<style>svg { width: 100%; }</style>\n\n";
        html_file.write(html_header)?;
        let mut dot_process = Command::new("dot")
            .arg("-Tsvg")
            .stdin(Stdio::piped())
            .stdout(html_file)
            .spawn()
            .expect("Failed to run Dot");
        let dot_stdin = dot_process
            .stdin
            .take()
            .expect("Failed to open stdin for Dot");

        parser.print_dot_graphs(&dot_stdin);

        Ok(())
    }
}

impl Drop for SwiftSyntaxTree {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        self.tree_sitter_parser.stop_printing_dot_graphs();
    }
}
