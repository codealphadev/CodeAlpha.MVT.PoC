use std::{
    fs::create_dir_all,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
use tree_sitter::{Node, Parser, Tree};

use crate::core_engine::{
    rules::{TextPosition, TextRange},
    utils::XcodeText,
};

use super::swift_codeblock::SwiftCodeBlock;

pub struct SwiftSyntaxTree {
    tree_sitter_parser: Parser,
    tree_sitter_tree: Option<Tree>,
    content: Option<XcodeText>,
    logging_folder: Option<PathBuf>,
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
            logging_folder: None,
        }
    }

    pub fn _reset(&mut self) {
        self.tree_sitter_tree = None;
        self.content = None;
    }

    pub fn parse(&mut self, content: &XcodeText) -> bool {
        let updated_tree = self.tree_sitter_parser.parse_utf16(content, None);

        if updated_tree.is_some() {
            self.tree_sitter_tree = updated_tree;
            self.content = Some(content.to_owned());
            return true;
        } else {
            return false;
        }
    }

    pub fn get_selected_code_node(&self, selected_text_range: &TextRange) -> Option<Node> {
        if let (Some(syntax_tree), Some(text_content)) =
            (self.tree_sitter_tree.as_ref(), self.content.as_ref())
        {
            if let Some((start_position, _)) =
                selected_text_range.as_StartEndTextPosition(text_content)
            {
                let node = syntax_tree.root_node().named_descendant_for_point_range(
                    TextPosition {
                        row: start_position.row,
                        column: start_position.column,
                    }
                    .as_TSPoint(),
                    TextPosition {
                        row: start_position.row,
                        column: start_position.column,
                    }
                    .as_TSPoint(),
                );

                return node;
            }
        }
        None
    }

    pub fn get_selected_codeblock_node(
        &self,
        selected_text_range: &TextRange,
    ) -> Option<SwiftCodeBlock> {
        if let (Some(mut node), Some(content)) = (
            self.get_selected_code_node(selected_text_range),
            self.content.as_ref(),
        ) {
            loop {
                if let Ok(codeblock_node) = SwiftCodeBlock::new(node, content) {
                    return Some(codeblock_node);
                }

                if let Some(parent) = node.parent() {
                    node = parent;
                } else {
                    break;
                }
            }
        }

        None
    }

    #[allow(dead_code)]
    pub fn tree(&self) -> Option<&Tree> {
        self.tree_sitter_tree.as_ref()
    }

    #[allow(dead_code)]
    pub fn start_logging(&mut self, path: &PathBuf) -> std::io::Result<()> {
        let html_header: &[u8] = b"<!DOCTYPE html>\n<style>svg { width: 100%; }</style>\n\n";

        create_dir_all(&path)?;

        let mut html_file_path = path.to_str().unwrap().to_string();
        html_file_path.push_str("tree.html");

        let mut html_file = std::fs::File::create(html_file_path)?;

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

        // let mut dot_file_path = path.to_str().unwrap().to_string();
        // dot_file_path.push_str("tree.dot");

        // let mut dot_file = std::fs::File::create(dot_file_path)?;
        // dot_file.write(html_header)?;
        // let mut dot_process = Command::new("dot")
        //     .arg("-Tsvg")
        //     .stdin(Stdio::piped())
        //     .stdout(dot_file)
        //     .spawn()
        //     .expect("Failed to run Dot");
        // let dot_stdin = dot_process
        //     .stdin
        //     .take()
        //     .expect("Failed to open stdin for Dot");
        self.tree_sitter_parser.print_dot_graphs(&dot_stdin);

        Ok(())
    }
}

impl Drop for SwiftSyntaxTree {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        if let Some(logging_folder) = &self.logging_folder {
            println!("{:?}", logging_folder);
        }
        self.tree_sitter_parser.stop_printing_dot_graphs();
    }
}

#[cfg(test)]
mod tests_SwiftSyntaxTree {

    use std::path::PathBuf;

    use crate::core_engine::{
        rules::TextPosition,
        utils::{utf16_bytes_count, utf16_treesitter_point_to_position, XcodeText},
    };

    use super::SwiftSyntaxTree;
    use pretty_assertions::assert_eq;
    use rand::Rng;

    #[test]
    #[ignore]
    fn test_tree_reset() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new();

        let original_string = "let";
        let replace_string = "var";
        let code_original = format!(
            "{} apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"",
            original_string
        )
        .encode_utf16()
        .collect();
        let code_updated = format!(
            "{} apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"",
            replace_string
        )
        .encode_utf16()
        .collect();

        swift_syntax_tree.parse(&code_original);
        {
            let root = swift_syntax_tree.tree().unwrap().root_node();
            let mut cursor = root.walk();
            println!(
                "Root before - ID {:?}",
                swift_syntax_tree.tree().unwrap().root_node().id()
            );
            for child in root.children(&mut cursor) {
                println!(
                    "Child before - ID {:?}, Kind {:?}, Text {:?}",
                    child.id(),
                    child.kind(),
                    child.utf16_text(&code_original)
                );
            }
        }

        swift_syntax_tree.parse(&code_updated);
        let root = swift_syntax_tree.tree().unwrap().root_node();
        let mut cursor = root.walk();
        println!(
            "Root after - ID {:?}",
            swift_syntax_tree.tree().unwrap().root_node().id()
        );
        for child in root.children(&mut cursor) {
            println!(
                "Child after - ID {:?}, Kind {:?}, Text {:?}",
                child.id(),
                child.kind(),
                child.utf16_text(&code_original)
            );
        }
    }

    #[test]
    #[ignore]
    fn test_tree_sitter_logging() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new();

        _ = swift_syntax_tree.start_logging(&prepare_treesitter_logging());

        let code = "var apples = 3\nlet appleSummary = \"I have \\(apples) apples.\""
            .encode_utf16()
            .collect();
        swift_syntax_tree.parse(&code);
        let updated_code = "let apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"\nlet appleSummary2 = \"I have \\(apples) apples.\"".encode_utf16().collect();
        swift_syntax_tree.parse(&updated_code);
    }

    fn prepare_treesitter_logging() -> PathBuf {
        let mut rng = rand::thread_rng();
        let n1: u32 = rng.gen::<u32>();

        let content = format!("{}/", n1);
        let mut path_buf = std::env::temp_dir();
        path_buf.push(content);

        println!("{}", path_buf.to_str().unwrap());

        path_buf
    }

    #[test]
    fn test_start_end_point_end_newline_char() {
        let text = "let x = 1; console.log(x);\n".encode_utf16().collect();
        //                |------------------------>| <- end column is zero on row 1
        //                                            <- end byte is one past the last byte (27), as they are also zero-based
        let mut swift_syntax_tree = SwiftSyntaxTree::new();
        swift_syntax_tree.parse(&text);

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), utf16_bytes_count(&text));
        assert_eq!(
            utf16_treesitter_point_to_position(&root_node.start_position()),
            TextPosition { row: 0, column: 0 }
        );
        assert_eq!(
            utf16_treesitter_point_to_position(&root_node.end_position()),
            TextPosition { row: 1, column: 0 }
        );
    }

    #[test]
    fn test_start_end_point_end_no_newline_char() {
        let text = "let x = 1; console.log(x);".encode_utf16().collect();
        //                |------------------------>| <- end column is one past the last char (26)
        //                |------------------------>| <- end byte is one past the last byte (26), as they are also zero-based
        let mut swift_syntax_tree = SwiftSyntaxTree::new();
        swift_syntax_tree.parse(&text);

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), utf16_bytes_count(&text));
        assert_eq!(
            utf16_treesitter_point_to_position(&root_node.start_position()),
            TextPosition { row: 0, column: 0 }
        );
        assert_eq!(
            utf16_treesitter_point_to_position(&root_node.end_position()),
            TextPosition { row: 0, column: 26 }
        );
    }

    #[test]
    fn test_start_end_point_with_UTF16_chars() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new();

        let mut text = "// 😊\n".encode_utf16().collect::<XcodeText>();
        let mut utf8_str = "let x = 1; console.log(x);"
            .encode_utf16()
            .collect::<XcodeText>();
        text.append(&mut utf8_str);

        swift_syntax_tree.parse(&text);

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), utf16_bytes_count(&text));
        assert_eq!(
            utf16_treesitter_point_to_position(&root_node.start_position()),
            TextPosition { row: 0, column: 0 }
        );
        assert_eq!(
            utf16_treesitter_point_to_position(&root_node.end_position()),
            TextPosition { row: 1, column: 26 }
        );
    }
}
