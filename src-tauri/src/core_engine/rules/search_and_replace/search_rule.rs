use crate::core_engine::rules::{
    rule_match::RuleMatchProps,
    utils::{text_types::TextRange, types::MatchRange},
    RuleBase, RuleMatch, RuleMatchCategory, RuleName, RuleResults,
};

pub struct SearchRuleProps {
    pub search_str: Option<String>,
    pub content: Option<String>,
}

pub struct SearchRule {
    rule_matches: Option<Vec<RuleMatch>>,
    search_str: Option<String>,
    content: Option<String>,

    search_str_updated: bool,
    content_updated: bool,
    rule_type: RuleName,
}

impl RuleBase for SearchRule {
    fn rule_type(&self) -> RuleName {
        self.rule_type.clone()
    }

    fn rule_matches(&self) -> Option<&Vec<RuleMatch>> {
        self.rule_matches.as_ref()
    }

    fn rule_results(&self) -> Option<RuleResults> {
        if let Some(rule_matches) = &self.rule_matches {
            Some(RuleResults {
                rule: self.rule_type(),
                results: rule_matches.clone(),
            })
        } else {
            None
        }
    }

    fn run(&mut self) -> Option<RuleResults> {
        if !self.search_str_updated && !self.content_updated {
            // nothing changed, no need to reprocess content, return cached results
            return self.rule_results();
        }

        if let (Some(content_str), Some(search_str)) =
            (self.content.as_ref(), self.search_str.as_ref())
        {
            let mut mut_content_str = content_str.clone();

            let mut rule_matches = Vec::new();

            let mut removed_chars = 0;
            while let Some((left_str, rest_str)) = mut_content_str.split_once(search_str) {
                let char_count_search_str = search_str.to_string().chars().count();
                let char_count_left_str = left_str.to_string().chars().count();

                rule_matches.push(RuleMatch::new(
                    RuleName::SearchAndReplace,
                    MatchRange {
                        string: search_str.to_string(),
                        range: TextRange {
                            index: char_count_left_str + removed_chars,
                            length: char_count_search_str,
                        },
                    },
                    RuleMatchProps {
                        identifier: "".to_string(),
                        description: "lint_alert.message".to_string(),
                        category: RuleMatchCategory::None,
                    },
                ));

                removed_chars += char_count_left_str + char_count_search_str;
                mut_content_str = rest_str.to_string();
            }

            self.rule_matches = Some(rule_matches);
        } else {
            self.rule_matches = None;
        }

        self.rule_results()
    }

    fn compute_rule_match_rectangles(&mut self, editor_app_pid: i32) -> Option<RuleResults> {
        if let Some(matches) = &mut self.rule_matches {
            for single_match in matches.iter_mut() {
                (*single_match).update_rectangles(editor_app_pid);
            }
        }

        self.rule_results()
    }
}

impl SearchRule {
    pub fn new() -> Self {
        Self {
            rule_matches: None,
            search_str: None,
            content: None,
            search_str_updated: false,
            content_updated: false,
            rule_type: RuleName::SearchAndReplace,
        }
    }

    pub fn update_properties(&mut self, properties: SearchRuleProps) {
        self.update_content(properties.content);
        self.update_search_str(properties.search_str);
    }

    fn update_content(&mut self, content_str: Option<String>) {
        if let Some(content_str) = content_str {
            // Update content if it has changed
            if self.content.is_none() || self.content.as_ref().unwrap() != &content_str {
                self.content = Some(content_str);
                self.content_updated = true;
            }
        }
    }

    fn update_search_str(&mut self, search_str: Option<String>) {
        if let Some(search_str) = search_str {
            // Update content if it has changed
            if self.search_str.is_none() || self.search_str.as_ref().unwrap() != &search_str {
                self.search_str = Some(search_str);
                self.search_str_updated = true;
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
        let search_str = "\ntext ever since".to_string();
        let mut rule = SearchRule::new();
        rule.update_properties(SearchRuleProps {
            search_str: Some(search_str),
            content: Some(content_str),
        });
        rule.run();

        if let Some(matches) = rule.rule_matches {
            println!("{:#?}", matches);
        } else {
            assert!(false, "No rule matches!");
        }
    }
}
