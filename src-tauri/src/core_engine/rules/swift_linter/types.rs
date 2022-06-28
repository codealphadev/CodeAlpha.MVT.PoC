use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct LintResults {
    pub lints: Vec<LintAlert>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LintAlert {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub level: LintLevel,
    pub message: String,
}

#[derive(Debug, Serialize, Clone)]
pub enum LintLevel {
    Error,
    Warning,
}
