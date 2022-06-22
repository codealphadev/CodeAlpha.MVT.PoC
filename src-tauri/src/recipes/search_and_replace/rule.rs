use super::utils::types::MatchRange;
use super::RuleMatch;

pub struct SearchRule {
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
        while let Some((left_str, rest_str)) = mut_content_str.split_once(&search_str) {
            let char_count_search_str = search_str.to_string().chars().count();
            let char_count_left_str = left_str.to_string().chars().count();

            rule_matches.push(RuleMatch {
                range: MatchRange {
                    index: char_count_left_str + removed_chars,
                    length: char_count_search_str,
                },
                matched: search_str.to_string(),
                rectangles: Vec::new(),
            });

            removed_chars += char_count_left_str + char_count_search_str;
            mut_content_str = rest_str.to_string();
        }

        self.rule_matches = Some(rule_matches);
    }

    pub fn compute_match_boundaries(&mut self, editor_app_pid: i32) {
        if let Some(matches) = &mut self.rule_matches {
            for single_match in matches.iter_mut() {
                (*single_match).update_rectangles(editor_app_pid);
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
        rule.run(&content_str, &search_str);

        if let Some(matches) = rule.rule_matches {
            println!("{:#?}", matches);
        } else {
            assert!(false);
        }
    }
}
