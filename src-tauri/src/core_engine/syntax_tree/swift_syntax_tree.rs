use std::{
    fs::create_dir_all,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};
use tree_sitter::{Node, Parser, Point, Tree};

use crate::core_engine::rules::TextRange;

use super::{swift_codeblock::SwiftCodeBlock, SwiftCodeBlockType};

pub struct SwiftSyntaxTree {
    tree_sitter_parser: Parser,
    tree_sitter_tree: Option<Tree>,
    content: Option<String>,
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

    pub fn reset(&mut self) {
        self.tree_sitter_tree = None;
        self.content = None;
    }

    pub fn parse(&mut self, content: &String) -> bool {
        // If there already exists a tree, we are updating it with the new content.
        // We assume the content is an updated version of the content parsed before.

        let updated_tree: Option<Tree>;
        if let (Some(_), Some(_)) = (&mut self.tree_sitter_tree, &self.content) {
            // // Skip if the content is the same as before.
            // if old_content == content {
            //     return true;
            // }

            // // Determine the edits made to the code document.
            // let mut input_edits: Vec<InputEdit> = detect_input_edits(old_content, content);
            // println!("{:?}", input_edits);

            // // Sort input_edits by start_byte in descending order before applying them to the tree.
            // input_edits.sort_by(|a, b| b.start_byte.cmp(&a.start_byte));

            // // Apply the sorted edits to the old tree.
            // for edit in input_edits.iter() {
            //     old_tree.edit(edit);
            // }

            // updated_tree = self.tree_sitter_parser.parse(content, Some(old_tree));
            println!("Parsing old tree");
            updated_tree = self.tree_sitter_parser.parse(content, None);
        } else {
            updated_tree = self.tree_sitter_parser.parse(content, None);
            println!("Parsing new tree");
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

    pub fn get_selected_codeblock_node(
        &self,
        selected_text_range: &TextRange,
    ) -> Option<SwiftCodeBlock> {
        // 1. Determine the node that the curser currently is on
        let mut currently_selected_node = None;
        if let (Some(syntax_tree), Some(text_content)) =
            (self.tree_sitter_tree.as_ref(), self.content.as_ref())
        {
            if let Some((selected_text_range_start_pos, _)) =
                selected_text_range.as_StartEndTextPosition(&text_content)
            {
                currently_selected_node = syntax_tree.root_node().named_descendant_for_point_range(
                    Point {
                        row: selected_text_range_start_pos.row,
                        column: selected_text_range_start_pos.column,
                    },
                    Point {
                        row: selected_text_range_start_pos.row,
                        column: selected_text_range_start_pos.column,
                    },
                );
            }
        }

        // 2. Find the nearest codeblock node
        if let (Some(mut node), Some(content)) =
            (currently_selected_node.clone(), self.content.as_ref())
        {
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

    use super::SwiftSyntaxTree;
    use pretty_assertions::assert_eq;
    use rand::Rng;
    use tree_sitter::Point;

    #[test]
    #[ignore]
    fn test_tree_reset() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new();

        let original_string = "let";
        let replace_string = "var";
        let code_original = format!(
            "{} apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"",
            original_string
        );
        let code_updated = format!(
            "{} apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"",
            replace_string
        );

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
                    child.utf8_text(code_original.as_bytes()).unwrap()
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
                child.utf8_text(code_updated.as_bytes()).unwrap()
            );
        }
    }

    #[test]
    #[ignore]
    fn test_tree_sitter_logging() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new();

        _ = swift_syntax_tree.start_logging(&prepare_treesitter_logging());

        let code = "var apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"";
        swift_syntax_tree.parse(&code.to_string());
        let updated_code = "let apples = 3\nlet appleSummary = \"I have \\(apples) apples.\"\nlet appleSummary2 = \"I have \\(apples) apples.\"";
        swift_syntax_tree.parse(&updated_code.to_string());
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
        let text = "let x = 1; console.log(x);\n";
        //                |------------------------>| <- end column is zero on row 1
        //                                            <- end byte is one past the last byte (27), as they are also zero-based
        let mut swift_syntax_tree = SwiftSyntaxTree::new();
        swift_syntax_tree.parse(&text.to_string());

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        // println!("{:#?}", root_node.start_position());
        // println!("{:#?}", root_node.end_position());

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), 27);
        assert_eq!(root_node.start_position(), Point { row: 0, column: 0 });
        assert_eq!(root_node.end_position(), Point { row: 1, column: 0 });
    }

    #[test]
    fn test_start_end_point_end_no_newline_char() {
        let text = "let x = 1; console.log(x);";
        //                |------------------------>| <- end column is one past the last char (26)
        //                |------------------------>| <- end byte is one past the last byte (26), as they are also zero-based
        let mut swift_syntax_tree = SwiftSyntaxTree::new();
        swift_syntax_tree.parse(&text.to_string());

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        // println!("{:#?}", root_node.start_position());
        // println!("{:#?}", root_node.end_position());

        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), 26);
        assert_eq!(root_node.start_position(), Point { row: 0, column: 0 });
        assert_eq!(root_node.end_position(), Point { row: 0, column: 26 });
    }

    #[test]
    fn test_start_end_point_with_UTF16_chars() {
        let mut swift_syntax_tree = SwiftSyntaxTree::new();

        let utf16_str = "// 😊\n";
        let utf8_str = "let x = 1; console.log(x);";
        let text = format!("{}{}", utf16_str, utf8_str);

        swift_syntax_tree.parse(&text.to_string());

        let root_node = swift_syntax_tree.tree().unwrap().root_node();

        // println!("Start Pos: {:#?}", root_node.start_position());
        // println!("End Pos: {:#?}", root_node.end_position());
        // println!("Start Byte: {:#?}", root_node.start_byte());
        // println!("End Byte: {:#?}", root_node.end_byte());

        assert_eq!(root_node.start_byte(), 0);
        let byte_count = utf8_str.bytes().count() + utf16_str.bytes().count(); // 26 (utf8) + 8 (utf16, emoji is 4 bytes)
        assert_eq!(root_node.end_byte(), byte_count);
        assert_eq!(root_node.start_position(), Point { row: 0, column: 0 });
        assert_eq!(root_node.end_position(), Point { row: 1, column: 26 });
    }
}
