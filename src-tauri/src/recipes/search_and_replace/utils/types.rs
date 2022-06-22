#[derive(Debug)]
pub struct MatchRectangle {
    pub origin: tauri::LogicalPosition<f64>,
    pub size: tauri::LogicalSize<f64>,
}

#[derive(Debug)]
pub struct MatchRange {
    pub index: usize,
    pub length: usize,
}
