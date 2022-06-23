use super::utils::types::{CharRange, MatchRange};
use super::RuleMatch;

pub struct SearchRule {
    pub rule_matches: Option<Vec<RuleMatch>>,
    pub search_str: Option<String>,
    pub content: Option<String>,
}

impl SearchRule {
    pub fn new() -> Self {
        Self {
            rule_matches: None,
            search_str: None,
            content: None,
        }
    }

    pub fn run(&mut self) {
        if let (Some(content_str), Some(search_str)) =
            (self.content.as_ref(), self.search_str.as_ref())
        {
            let mut mut_content_str = content_str.clone();

            let mut rule_matches = Vec::new();

            let mut removed_chars = 0;
            while let Some((left_str, rest_str)) = mut_content_str.split_once(search_str) {
                let char_count_search_str = search_str.to_string().chars().count();
                let char_count_left_str = left_str.to_string().chars().count();

                rule_matches.push(RuleMatch {
                    match_range: MatchRange {
                        string: search_str.to_string(),
                        range: CharRange {
                            index: char_count_left_str + removed_chars,
                            length: char_count_search_str,
                        },
                    },
                    rectangles: Vec::new(),
                    line_matches: Vec::new(),
                });

                removed_chars += char_count_left_str + char_count_search_str;
                mut_content_str = rest_str.to_string();
            }

            self.rule_matches = Some(rule_matches);
        }
    }

    pub fn update_content(&mut self, content_str: &String) {
        self.content = Some(content_str.clone());
    }

    pub fn update_search_str(&mut self, search_str: &str) {
        self.search_str = Some(search_str.to_string());
    }

    pub fn compute_match_boundaries(
        &mut self,
        editor_app_pid: i32,
        editor_window_hash: Option<usize>,
    ) {
        if let Some(matches) = &mut self.rule_matches {
            for single_match in matches.iter_mut() {
                (*single_match).update_rectangles(editor_app_pid, editor_window_hash);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_rule() {
        let content_str = "//*\n//  AXSwift.h\n//  AXSwift\n//\n//  Created by Tyler Mandry on 10/18/15.\n//  Copyright © 2015 Tyler Mandry. All rights reserved.\n//\n\n#import <Cocoa/Cocoa.h>\n\n//! Project version number for AXSwift.\nFOUNDATION_EXPORT double AXSwiftVersionNumber;\n\n//! Project version string for AXSwift.\nFOUNDATION_EXPORT const unsigned char AXSwiftVersionString[];\n\n// In this header, you should import all the public headers of your framework using statements like\n// #import <AXSwift/PublicHeader.h>\ntext ever since \n".to_string();
        let search_str = "text ever since ".to_string();
        let mut rule = SearchRule::new();
        rule.update_content(&content_str);
        rule.update_search_str(&search_str);
        rule.run();

        if let Some(matches) = rule.rule_matches {
            println!("{:#?}", matches);
        } else {
            assert!(false);
        }
    }
}
