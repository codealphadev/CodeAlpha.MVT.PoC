use std::process::Command;

use crate::{
    ax_interaction::get_textarea_uielement,
    core_engine::rules::{
        utils::{
            ax_utils::get_char_range_of_line,
            types::{CharRange, MatchRange},
        },
        RuleBase, RuleMatch, RuleName, RuleResults,
    },
};

use super::types::{LintAlert, LintLevel, LintResults};

pub struct SwiftLinterProps {
    pub file_path_as_str: Option<String>,
    pub linter_config: Option<String>,
}

pub struct SwiftLinterRule {
    rule_type: RuleName,
    rule_matches: Option<Vec<RuleMatch>>,
    file_path_updated: bool,
    file_path_as_str: Option<String>,
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
        if !self.linter_config_updated && !self.file_path_updated {
            // nothing changed, no need to reprocess, return cached results
            return self.rule_results();
        }

        let textarea_uielement =
            if let Some(textarea_uielement) = get_textarea_uielement(self.editor_app_pid) {
                textarea_uielement
            } else {
                return None;
            };

        if let Some(linter_results) = self.lint_swift_file() {
            let mut rule_matches = Vec::new();

            for lint_alert in linter_results.lints {
                let char_range_for_line = if let Some(char_range_for_line) =
                    get_char_range_of_line(lint_alert.line as i64 - 1, &textarea_uielement)
                {
                    char_range_for_line
                } else {
                    continue;
                };

                let rule_match = RuleMatch::new(
                    RuleName::SwiftLinter,
                    MatchRange {
                        string: "todo!()".to_string(),
                        range: CharRange {
                            index: char_range_for_line.index + lint_alert.column,
                            length: 1,
                        },
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
    pub fn new(editor_app_pid: i32) -> Self {
        Self {
            rule_matches: None,
            file_path_updated: false,
            file_path_as_str: None,
            linter_config: Some("--quiet".to_string()),
            linter_config_updated: false,
            rule_type: RuleName::SwiftLinter,
            editor_app_pid,
        }
    }

    pub fn update_properties(&mut self, properties: SwiftLinterProps) {
        self.update_file_path(properties.file_path_as_str);
        self.update_linter_config(properties.linter_config);
    }

    fn update_file_path(&mut self, file_path_as_str: Option<String>) {
        if let Some(file_path) = file_path_as_str {
            if self.file_path_as_str.is_none()
                || self.file_path_as_str.as_ref().unwrap() != &file_path
            {
                self.file_path_as_str = Some(file_path);
                self.file_path_updated = true;
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

        LintAlert {
            file_path: parts[0].to_string(),
            line: parts[1].parse::<usize>().unwrap(),
            column: parts[2].parse::<usize>().unwrap(),
            level: if parts[3].trim() == "error" {
                LintLevel::Error
            } else {
                LintLevel::Warning
            },
            message: last_parts.join(":").to_string(),
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
        let editor_app_pid = 2763;
        let mut rule = SwiftLinterRule::new(editor_app_pid);
        rule.update_properties(SwiftLinterProps {
            file_path_as_str: Some(file_path_as_str.to_string()),
            linter_config: None,
        });
        rule.run();

        if let Some(matches) = rule.rule_matches {
            println!("{:#?}", matches);
        } else {
            assert!(false, "No rule matches!");
        }
    }
}
