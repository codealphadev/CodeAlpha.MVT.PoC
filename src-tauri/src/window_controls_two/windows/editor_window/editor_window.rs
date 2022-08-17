use tauri::Manager;

use crate::{
    ax_interaction::models::editor::{EditorWindowCreatedMessage, FocusedUIElement},
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls_two::config::AppWindow,
};

#[derive(Debug)]
enum HorizontalBoundary {
    Left,
    Right,
}

#[derive(Debug)]
enum VerticalBoundary {
    Top,
    Bottom,
}

#[derive(Debug)]
pub struct EditorWindow {
    /// The unique identifier is generated the moment we 'detect' a previously unknown editor window.
    uielement_hash: usize,

    /// The application name of the editor this window belongs to. For XCode it is "Xcode".
    editor_name: String,

    /// The process identifier for the window's editor application.
    pid: i32,

    /// This enum holds the last type of the last focused ui element in this editor window.
    focused_ui_element: Option<FocusedUIElement>,

    /// The editor window's dimensions.
    window_position: LogicalPosition,
    window_size: LogicalSize,

    /// The Text Area is the ui element within xcode that is used for editing code
    /// When initially focusing an editor window the text area might not be visible,
    /// wherefore it's dimension might not be known.
    /// The coordinates of the textarea's origin are relative to the editor window.
    textarea_position: Option<LogicalPosition>,
    textarea_size: Option<LogicalSize>,

    /// Widget position to the editor's text area.
    widget_position: Option<LogicalPosition>,

    /// When the editor text area's size or position is updated, the widget_position
    /// is recalculated with respect to the boundaries. The boundaries are initially set to bottom|right
    /// but get updated each time the user moves the widget manually
    h_boundary: HorizontalBoundary,
    v_boundary: VerticalBoundary,
}

impl EditorWindow {
    pub fn new(created_msg: &EditorWindowCreatedMessage) -> Self {
        Self {
            uielement_hash: created_msg.ui_elem_hash,
            editor_name: created_msg.editor_name.clone(),
            pid: created_msg.pid,
            window_position: LogicalPosition::from_tauri_LogicalPosition(
                &created_msg.window_position,
            ),
            window_size: LogicalSize::from_tauri_LogicalSize(&created_msg.window_size),
            textarea_position: None,
            textarea_size: None,
            focused_ui_element: None,
            h_boundary: HorizontalBoundary::Right,
            v_boundary: VerticalBoundary::Bottom,
            widget_position: None,
        }
    }

    pub fn pid(&self) -> i32 {
        self.pid
    }

    pub fn editor_name(&self) -> &String {
        &self.editor_name
    }

    pub fn textarea_position(&self, as_global_position: bool) -> Option<LogicalPosition> {
        if as_global_position {
            Some(self.transform_local_position_to_global_position(self.textarea_position?))
        } else {
            Some(self.textarea_position?)
        }
    }

    pub fn textarea_size(&self) -> Option<LogicalSize> {
        self.textarea_size
    }

    pub fn window_size(&self) -> LogicalSize {
        self.window_size
    }

    pub fn window_position(&self) -> LogicalPosition {
        self.window_position
    }

    pub fn widget_position(&self, as_global_position: bool) -> Option<LogicalPosition> {
        if as_global_position {
            Some(self.transform_local_position_to_global_position(self.widget_position?))
        } else {
            Some(self.widget_position?)
        }
    }

    pub fn focused_ui_element(&self) -> Option<&FocusedUIElement> {
        self.focused_ui_element.as_ref()
    }

    pub fn update_window_dimensions(
        &mut self,
        window_position: LogicalPosition,
        window_size: LogicalSize,
        textarea_position_global: Option<LogicalPosition>,
        textarea_size: Option<LogicalSize>,
    ) {
        // Transforming the global position of the textarea to a local position.
        let textarea_position = if let Some(textarea_position) = textarea_position_global {
            Some(self.transform_global_position_to_local_position(
                textarea_position,
                Some(window_position),
            ))
        } else {
            None
        };

        // Calculate the change of the window dimensions
        let (diff_pos, diff_size) = self.calculate_dimension_change(
            window_position,
            window_size,
            textarea_position,
            textarea_size,
        );

        // Update textarea dimension
        self.update_textarea_dimensions(diff_size, textarea_position, textarea_size);

        // Update widget position
        if textarea_position.is_some() {
            self.calc_widget_pos_by_respecting_boundaries(diff_pos, diff_size);
        }

        self.window_position = window_position;
        self.window_size = window_size;
    }

    pub fn get_monitor(&self, app_handle: tauri::AppHandle) -> Option<LogicalFrame> {
        // Because we've seen the OS calls to get a window's screen position and size are not
        // reliable, we determine the window by calculating which of the windows has the largest
        // overlap area with the editor window.

        let widget_window = app_handle.get_window(&AppWindow::Widget.to_string())?;
        let available_monitors = widget_window.available_monitors().ok()?;

        // Compute the overlapping area between all available monitors and the editor window's outer size
        let mut overlap_areas: Vec<(f64, LogicalFrame)> = Vec::new();
        for monitor in available_monitors {
            let scale_factor = monitor.scale_factor();
            let position = LogicalPosition::from_tauri_LogicalPosition(
                &monitor.position().to_logical::<f64>(scale_factor),
            );
            let size = LogicalSize::from_tauri_LogicalSize(
                &monitor.size().to_logical::<f64>(scale_factor),
            );

            if let Some(intersection_area) = intersection_area(
                LogicalFrame {
                    origin: position,
                    size,
                },
                LogicalFrame {
                    origin: self.window_position,
                    size: self.window_size,
                },
            ) {
                overlap_areas.push((
                    intersection_area,
                    LogicalFrame {
                        origin: position,
                        size,
                    },
                ));
            }
        }

        // get the item of overlap_areas with the largest area
        if let Some((_, monitor)) = overlap_areas
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        {
            Some(*monitor)
        } else {
            None
        }
    }

    pub fn update_focused_ui_element(
        &mut self,
        focused_ui_element: &FocusedUIElement,
        textarea_position_global: Option<LogicalPosition>,
        textarea_size: Option<LogicalSize>,
    ) {
        // Transforming the global position of the textarea to a local position.
        let textarea_position = if let Some(textarea_position) = textarea_position_global {
            Some(self.transform_global_position_to_local_position(textarea_position, None))
        } else {
            None
        };

        if textarea_position.is_some() {
            self.textarea_position = textarea_position;
        }

        if textarea_size.is_some() {
            self.textarea_size = textarea_size;
        }

        self.focused_ui_element = Some(focused_ui_element.clone());
    }

    fn calculate_dimension_change(
        &self,
        window_position: LogicalPosition,
        window_size: LogicalSize,
        textarea_position: Option<LogicalPosition>,
        textarea_size: Option<LogicalSize>,
    ) -> (LogicalSize, LogicalSize) {
        // Calculate the change of the dimensions
        let mut diff_pos = LogicalSize {
            width: window_position.x - self.window_position.x,
            height: window_position.y - self.window_position.y,
        };

        let mut diff_size = LogicalSize {
            width: window_size.width - self.window_size.width,
            height: window_size.height - self.window_size.height,
        };

        // If textarea dimensions are provided, use their change of the dimensions
        if let (
            Some(textarea_pos_new),
            Some(textarea_size_new),
            Some(textarea_pos_old),
            Some(textarea_size_old),
        ) = (
            textarea_position,
            textarea_size,
            self.textarea_position,
            self.textarea_size,
        ) {
            diff_pos = LogicalSize {
                width: textarea_pos_new.x - textarea_pos_old.x,
                height: textarea_pos_new.y - textarea_pos_old.y,
            };

            diff_size = LogicalSize {
                width: textarea_size_new.width - textarea_size_old.width,
                height: textarea_size_new.height - textarea_size_old.height,
            };
        }

        return (diff_pos, diff_size);
    }

    fn update_textarea_dimensions(
        &mut self,
        diff_size: LogicalSize,
        textarea_position: Option<LogicalPosition>,
        textarea_size: Option<LogicalSize>,
    ) -> Option<()> {
        if textarea_position.is_some() && textarea_size.is_some() {
            self.textarea_position = textarea_position;
            self.textarea_size = textarea_size;
        } else {
            // Case: valid updated textarea dimensions are provided;
            // Case: Deriving updated textarea dimensions from window dimension change;
            if self.textarea_size.is_some() && self.textarea_position.is_some() {
                self.textarea_size = Some(LogicalSize {
                    width: self.textarea_size?.width + diff_size.width,
                    height: self.textarea_size?.height + diff_size.height,
                });
            }
        }

        Some(())
    }

    pub fn update_widget_position(&mut self, widget_position_global: LogicalPosition) {
        // Transforming the global position of the widget to a local position.
        let widget_position =
            self.transform_global_position_to_local_position(widget_position_global, None);

        self.widget_position = Some(widget_position);

        // Recalculate boundaries
        if let (Some(textarea_pos), Some(textarea_size)) =
            (self.textarea_position, self.textarea_size)
        {
            let left_boundary = textarea_pos.x;
            let right_boundary = textarea_pos.x + textarea_size.width;
            let bottom_boundary = textarea_pos.y + textarea_size.height;
            let top_boundary = textarea_pos.y;

            let dist_to_left = (left_boundary - widget_position.x).abs();
            let dist_to_right = (right_boundary - widget_position.x).abs();
            let dist_to_top = (top_boundary - widget_position.y).abs();
            let dist_to_bottom = (bottom_boundary - widget_position.y).abs();

            // Match closest distance
            if dist_to_left > dist_to_right {
                self.h_boundary = HorizontalBoundary::Right;
            } else {
                self.h_boundary = HorizontalBoundary::Left;
            }

            if dist_to_top > dist_to_bottom {
                self.v_boundary = VerticalBoundary::Bottom;
            } else {
                self.v_boundary = VerticalBoundary::Top;
            }
        }
    }

    fn calc_widget_pos_by_respecting_boundaries(
        &mut self,
        diff_pos: LogicalSize,
        diff_size: LogicalSize,
    ) -> Option<()> {
        let widget_pos = &mut self.widget_position?;
        // Determine how much each side/boundary moved
        let left_boundary_diff = diff_pos.width;
        let right_boundary_diff = diff_pos.width + diff_size.width;
        let bottom_boundary_diff = diff_pos.height + diff_size.height;
        let top_boundary_diff = diff_pos.height;

        match self.v_boundary {
            VerticalBoundary::Top => {
                widget_pos.y = widget_pos.y + top_boundary_diff;
            }
            VerticalBoundary::Bottom => {
                widget_pos.y = widget_pos.y + bottom_boundary_diff;
            }
        }

        match self.h_boundary {
            HorizontalBoundary::Left => {
                widget_pos.x = widget_pos.x + left_boundary_diff;
            }
            HorizontalBoundary::Right => {
                widget_pos.x = widget_pos.x + right_boundary_diff;
            }
        }

        Some(())
    }

    fn transform_local_position_to_global_position(
        &self,
        local_position: LogicalPosition,
    ) -> LogicalPosition {
        let mut global_position = local_position;

        global_position.x += &self.window_position.x;
        global_position.y += &self.window_position.y;

        return global_position;
    }

    fn transform_global_position_to_local_position(
        &self,
        global_position: LogicalPosition,
        local_origin: Option<LogicalPosition>,
    ) -> LogicalPosition {
        let mut local_position = global_position;

        if let Some(local_origin) = local_origin {
            local_position.x -= local_origin.x;
            local_position.y -= local_origin.y;
        } else {
            local_position.x -= &self.window_position.x;
            local_position.y -= &self.window_position.y;
        }

        return local_position;
    }
}

fn intersection_area(rect_a: LogicalFrame, rect_b: LogicalFrame) -> Option<f64> {
    let (a_x_min, a_y_min, a_x_max, a_y_max) = (
        rect_a.origin.x,
        rect_a.origin.y,
        rect_a.origin.x + rect_a.size.width,
        rect_a.origin.y + rect_a.size.height,
    );

    let (b_x_min, b_y_min, b_x_max, b_y_max) = (
        rect_b.origin.x,
        rect_b.origin.y,
        rect_b.origin.x + rect_b.size.width,
        rect_b.origin.y + rect_b.size.height,
    );

    let x_min = f64::max(a_x_min, b_x_min);
    let y_min = f64::max(a_y_min, b_y_min);
    let x_max = f64::min(a_x_max, b_x_max);
    let y_max = f64::min(a_y_max, b_y_max);
    let width = x_max - x_min;
    let height = y_max - y_min;
    if width < 0.0 || height < 0.0 {
        return None;
    }

    Some(width * height)
}

#[cfg(test)]
mod tests_editor_window {
    use crate::{
        utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
        window_controls_two::windows::editor_window::editor_window::intersection_area,
    };

    #[test]
    fn test_intersection_area_some() {
        let rect_a = LogicalFrame {
            origin: LogicalPosition { x: 0.0, y: 0.0 },
            size: LogicalSize {
                width: 10.0,
                height: 10.0,
            },
        };
        let rect_b = LogicalFrame {
            origin: LogicalPosition { x: 5.0, y: 5.0 },
            size: LogicalSize {
                width: 10.0,
                height: 10.0,
            },
        };
        let intersection = intersection_area(rect_a, rect_b);
        assert_eq!(intersection, Some(25.0));
    }

    #[test]
    fn test_intersection_area_none() {
        let rect_a = LogicalFrame {
            origin: LogicalPosition { x: 0.0, y: 0.0 },
            size: LogicalSize {
                width: 10.0,
                height: 10.0,
            },
        };
        let rect_b = LogicalFrame {
            origin: LogicalPosition { x: 15.0, y: 15.0 },
            size: LogicalSize {
                width: 10.0,
                height: 10.0,
            },
        };
        let intersection = intersection_area(rect_a, rect_b);
        assert_eq!(intersection, None);
    }
}
