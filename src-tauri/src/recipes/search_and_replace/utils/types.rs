#[derive(Debug, Clone)]
pub struct MatchRectangle {
    pub origin: tauri::LogicalPosition<f64>,
    pub size: tauri::LogicalSize<f64>,
}

#[derive(Debug, Clone)]
pub struct CharRange {
    pub index: usize,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct MatchRange {
    pub string: String,
    pub range: CharRange,
}
