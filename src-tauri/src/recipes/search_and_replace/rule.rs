use super::MatchRange;
use super::RuleMatch;

struct SearchRule {
    pub rule_matches: Option<Vec<RuleMatch>>,
}

impl SearchRule {
    pub fn new() -> Self {
        Self { rule_matches: None }
    }

    pub fn run(&mut self, content_str: &String, search_str: &str) {
        let mut mut_content_str = content_str.clone();

        let mut rule_matches = Vec::new();
        let mut removed_chars = 0;
        while let Some(start_index) = mut_content_str.find(&search_str) {
            let end_index = start_index + search_str.len();
            let matched = mut_content_str[start_index..end_index].to_string();
            let rectangles = Vec::new();
            rule_matches.push(RuleMatch {
                range: MatchRange {
                    index: start_index + removed_chars,
                    length: matched.len(),
                },
                matched,
                rectangles,
            });
            removed_chars += end_index;
            mut_content_str = mut_content_str[end_index..].to_string();
        }

        self.rule_matches = Some(rule_matches);
    }
}

#[cfg(test)]
mod tests {
    use crate::ax_interaction::xcode::get_xcode_editor_content;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_search_rule() {
        let content_str = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.".to_string();
        let search_str = "Lorem".to_string();
        let mut rule = SearchRule::new();
        rule.run(&content_str, &search_str);

        if let Some(mut matches) = rule.rule_matches {
            println!("{:#?}", matches);

            assert_eq!(matches.len(), 4);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_get_rectangles() {
        let editor_pid = 12538 as i32;
        if let Ok(editor_content_option) = get_xcode_editor_content(editor_pid) {
            if let Some(editor_content) = editor_content_option {
                let search_str = "text ever since ".to_string();
                let mut rule = SearchRule::new();
                rule.run(&editor_content, &search_str);
                // rule.update_match_rectangles_TBD();

                if let Some(mut matches) = rule.rule_matches {
                    for single_match in matches.iter_mut() {
                        (*single_match).update_rectangles(editor_pid);
                    }

                    assert_eq!(matches.len(), 1);
                } else {
                    assert!(false);
                }
            }
        }

        // Observed "odd" behavior:
        // - Word Wrap always draws the bounding around the whole text area (horizontally)
        // - When matching the last characters a string that wraps around lines, the rect always extents to the maximum end text area's extents
        // - can not detenct yet on which characters wordwrap appears
    }
}
