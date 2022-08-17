use std::process::Command;

use crate::{
    ax_interaction::get_textarea_uielement,
    core_engine::rules::{
        rule_match::RuleMatchProps,
        utils::{ax_utils::get_text_range_of_line, text_types::TextRange, types::MatchRange},
        RuleBase, RuleMatch, RuleMatchCategory, RuleName, RuleResults,
    },
};

use super::types::{LintAlert, LintLevel, LintResults};

pub struct SwiftLinterProps {
    pub file_path_as_str: Option<String>,
    pub linter_config: Option<String>,
    pub file_content: Option<String>,
}

pub struct SwiftLinterRule {
    rule_type: RuleName,
    rule_matches: Option<Vec<RuleMatch>>,
    file_path_updated: bool,
    file_path_as_str: Option<String>,
    file_content: Option<String>,
    file_content_updated: bool,
    linter_config: Option<String>,
    linter_config_updated: bool,
    editor_app_pid: i32,
}

impl RuleBase for SwiftLinterRule {
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
        if !self.linter_config_updated && !self.file_path_updated && !self.file_content_updated {
            // nothing changed, no need to reprocess, return cached results
            return self.rule_results();
        }

        let textarea_uielement =
            if let Some(textarea_uielement) = get_textarea_uielement(self.editor_app_pid) {
                textarea_uielement
            } else {
                return None;
            };

        // Process all found linter results
        if let Some(linter_results) = self.lint_swift_file() {
            let mut rule_matches = Vec::new();

            for lint_alert in linter_results.lints.iter().enumerate() {
                let char_range_for_line = if let Some(char_range_for_line) =
                    get_text_range_of_line(lint_alert.1.line - 1, &textarea_uielement)
                {
                    char_range_for_line
                } else {
                    continue;
                };

                let rule_match = RuleMatch::new(
                    RuleName::SwiftLinter,
                    MatchRange {
                        string: "unknown yet".encode_utf16().collect::<Vec<u16>>(),
                        range: TextRange {
                            index: char_range_for_line.index + lint_alert.1.column,
                            length: 1,
                        },
                    },
                    RuleMatchProps {
                        identifier: lint_alert.1.identifier.clone(),
                        description: lint_alert.1.message.clone(),
                        category: RuleMatchCategory::from_lint_level(lint_alert.1.level.clone()),
                    },
                );

                rule_matches.push(rule_match);
            }

            rule_matches.sort_by(|a, b| {
                a.match_range()
                    .range
                    .index
                    .cmp(&b.match_range().range.index)
            });

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

impl SwiftLinterRule {
    pub fn _new(editor_app_pid: i32) -> Self {
        Self {
            rule_matches: None,
            file_path_updated: false,
            file_path_as_str: None,
            linter_config: Some("--quiet".to_string()),
            linter_config_updated: false,
            rule_type: RuleName::SwiftLinter,
            file_content: None,
            file_content_updated: false,
            editor_app_pid,
        }
    }

    pub fn update_properties(&mut self, properties: SwiftLinterProps) {
        self.update_file_path(properties.file_path_as_str);
        self.update_linter_config(properties.linter_config);
        self.update_file_content(properties.file_content);
    }

    fn update_file_path(&mut self, file_path_as_str: Option<String>) {
        if let Some(file_path) = file_path_as_str {
            if self.file_path_as_str.is_none()
                || self.file_path_as_str.as_ref().unwrap() != &file_path
            {
                self.file_path_as_str = Some(file_path);
                self.file_path_updated = true;
            } else {
                self.file_path_updated = false;
            }
        }
    }

    fn update_linter_config(&mut self, linter_config: Option<String>) {
        if let Some(linter_config) = linter_config {
            if self.linter_config.is_none()
                || self.linter_config.as_ref().unwrap() != &linter_config
            {
                self.linter_config = Some(linter_config);
                self.linter_config_updated = true;
            } else {
                self.linter_config_updated = false;
            }
        }
    }

    fn update_file_content(&mut self, file_content: Option<String>) {
        if let Some(file_content) = file_content {
            // Update content if it has changed
            if self.file_content.is_none() || self.file_content.as_ref().unwrap() != &file_content {
                self.file_content = Some(file_content);
                self.file_content_updated = true;
            } else {
                self.file_content_updated = false;
            }
        }
    }

    fn lint_swift_file(&self) -> Option<LintResults> {
        if let Some(file_path) = &self.file_path_as_str {
            let output = Command::new("swiftlint")
                .arg(file_path)
                .arg("--quiet")
                .output()
                .expect("failed to execute process");

            let mut lints = Vec::new();
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let lint = Self::parse_lint_line(line);
                lints.push(lint);
            }

            Some(LintResults { lints })
        } else {
            None
        }
    }

    fn parse_lint_line(line: &str) -> LintAlert {
        let parts: Vec<&str> = line.split(":").collect();
        let (_, last_parts) = parts.split_at(4);

        let last_parts = last_parts.join(":").to_string();
        let message_parts: Vec<&str> = last_parts.split("(").collect();
        let (message, identifier_str) = message_parts.split_at(1);

        let mut identifier = identifier_str.join("(").to_string();
        identifier.pop();

        LintAlert {
            file_path: parts[0].to_string(),
            line: parts[1].parse::<usize>().unwrap(),
            column: parts[2].parse::<usize>().unwrap(),
            level: if parts[3].trim() == "error" {
                LintLevel::Error
            } else {
                LintLevel::Warning
            },
            message: message.join("(").to_string(),
            identifier: identifier.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core_engine::rules::RuleBase;

    use super::{SwiftLinterProps, SwiftLinterRule};

    #[test]
    #[ignore]
    fn test_swift_linter() {
        let file_path_as_str = "/Users/adam/codealpha/code/adam-test/Shared/ContentView.swift";
        let mut rule = SwiftLinterRule::_new(12345);
        rule.update_properties(SwiftLinterProps {
            file_path_as_str: Some(file_path_as_str.to_string()),
            linter_config: None,
            file_content: None,
        });
        rule.run();

        if let Some(matches) = rule.rule_matches {
            println!("{:#?}", matches);
        } else {
            assert!(false, "No rule matches!");
        }
    }
}
