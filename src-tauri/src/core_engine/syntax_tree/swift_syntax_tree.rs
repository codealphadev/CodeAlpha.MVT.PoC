use std::{
    io::Write,
    process::{Command, Stdio},
};
use tree_sitter::{Parser, Tree};

pub struct SwiftSyntaxTree {
    tree_sitter_parser: Parser,
    tree_sitter_tree: Option<Tree>,
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
        }
    }

    pub fn reset(&mut self) {
        self.tree_sitter_tree = None;
    }

    pub fn parse(&mut self, source: &String) {
        if let Some(old_tree) = &self.tree_sitter_tree {
            self.tree_sitter_tree = self.tree_sitter_parser.parse(source, Some(old_tree));
        } else {
            self.tree_sitter_tree = self.tree_sitter_parser.parse(source, None);
        }
    }

    pub fn get_tree_copy(&self) -> Option<Tree> {
        self.tree_sitter_tree.clone()
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
