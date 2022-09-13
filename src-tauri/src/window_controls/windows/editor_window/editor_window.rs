use tauri::Manager;

use crate::{
    app_handle,
    platform::macos::{
        get_dark_mode,
        models::editor::{EditorWindowCreatedMessage, FocusedUIElement},
        CodeDocumentFrameProperties, ViewportProperties,
    },
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::AppWindow, events::EventWindowControls, models::dark_mode::DarkModeUpdateMessage,
    },
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
    _id: usize,

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

    dark_mode: Option<bool>,

    viewport_props: Option<ViewportProperties>,
    code_document_props: Option<CodeDocumentFrameProperties>,
}

impl EditorWindow {
    pub fn new(created_msg: &EditorWindowCreatedMessage) -> Self {
        let mut editor_window = Self {
            _id: created_msg.window_uid,
            dark_mode: None,
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
            viewport_props: None,
            code_document_props: None,
        };
        editor_window.check_and_update_dark_mode().ok();
        editor_window
    }

    pub fn pid(&self) -> i32 {
        self.pid
    }

    pub fn editor_name(&self) -> &String {
        &self.editor_name
    }

    pub fn textarea_position(&self, as_global_position: bool) -> Option<LogicalPosition> {
        if as_global_position {
            Some(Self::transform_local_position_to_global_position(
                self.window_position,
                self.textarea_position?,
            ))
        } else {
            self.textarea_position
        }
    }

    pub fn textarea_size(&self) -> Option<LogicalSize> {
        self.textarea_size
    }

    pub fn widget_position(&self, as_global_position: bool) -> Option<LogicalPosition> {
        if as_global_position {
            Some(Self::transform_local_position_to_global_position(
                self.window_position,
                self.widget_position?,
            ))
        } else {
            Some(self.widget_position?)
        }
    }

    pub fn focused_ui_element(&self) -> Option<&FocusedUIElement> {
        self.focused_ui_element.as_ref()
    }

    pub fn viewport(&self) -> Option<ViewportProperties> {
        self.viewport_props.clone()
    }

    pub fn set_viewport(&mut self, viewport: Option<ViewportProperties>) {
        if viewport.is_some() {
            self.viewport_props = viewport;
        }
    }

    pub fn code_document(&self) -> Option<CodeDocumentFrameProperties> {
        self.code_document_props.clone()
    }

    pub fn set_code_document(&mut self, code_document: Option<CodeDocumentFrameProperties>) {
        if code_document.is_some() {
            self.code_document_props = code_document;
        }
    }

    fn set_textarea_dimensions(
        &mut self,
        position: LogicalPosition,
        size: LogicalSize,
    ) -> Result<(), String> {
        self.textarea_position = Some(position);
        self.textarea_size = Some(size);

        Ok(())
    }

    pub fn check_and_update_dark_mode(&mut self) -> Result<(), String> {
        self.dark_mode = get_dark_mode().ok();

        EventWindowControls::DarkModeUpdate(DarkModeUpdateMessage {
            dark_mode: self.dark_mode.ok_or("dark mode is None".to_string())?,
        })
        .publish_to_tauri(&app_handle());
        Ok(())
    }

    pub fn update_window_and_textarea_dimensions(
        &mut self,
        window: LogicalFrame,
        textarea_global: LogicalFrame,
    ) -> Option<()> {
        // Transform the textarea's origin to local coordinates.
        let textarea = LogicalFrame {
            origin: Self::transform_global_position_to_local_position(
                self.window_position,
                textarea_global.origin,
                Some(window.origin),
            ),
            size: textarea_global.size,
        };

        // Calculate how much both the window and the textarea have been moved and/or changed in size.
        // These values are needed to calculate the widget's position after the update.
        // If self.textarea_position is None, we use the change from the window's origin/size.
        let mut change_of_origin = LogicalSize {
            width: window.origin.x - self.window_position.x,
            height: window.origin.y - self.window_position.y,
        };

        let mut change_of_size = LogicalSize {
            width: window.size.width - self.window_size.width,
            height: window.size.height - self.window_size.height,
        };

        if let (Some(textarea_pos_old), Some(textarea_size_old)) =
            (self.textarea_position, self.textarea_size)
        {
            let textarea_pos_old_global = Self::transform_local_position_to_global_position(
                self.window_position,
                textarea_pos_old,
            );

            change_of_origin = LogicalSize {
                width: textarea_global.origin.x - textarea_pos_old_global.x,
                height: textarea_global.origin.y - textarea_pos_old_global.y,
            };

            change_of_size = LogicalSize {
                width: textarea.size.width - textarea_size_old.width,
                height: textarea.size.height - textarea_size_old.height,
            };
        }

        // Update the widget's position
        let widget_position_global_updated =
            self.get_updated_widget_position_global(change_of_origin, change_of_size);

        // Update the editor window's and textarea's dimensions.
        self.window_position = window.origin;
        self.window_size = window.size;
        _ = self.set_textarea_dimensions(textarea.origin, textarea.size);
        self.widget_position
            .replace(Self::transform_global_position_to_local_position(
                self.window_position,
                widget_position_global_updated?,
                Some(window.origin),
            ));

        Some(())
    }

    pub fn update_window_dimensions(&mut self, window: LogicalFrame) -> Option<()> {
        // Calculate how much both the window and the textarea have been moved and/or changed in size.
        // These values are needed to calculate the widget's position after the update.
        // If self.textarea_position is None, we use the change from the window's origin/size.
        let change_of_origin = LogicalSize {
            width: window.origin.x - self.window_position.x,
            height: window.origin.y - self.window_position.y,
        };

        let change_of_size = LogicalSize {
            width: window.size.width - self.window_size.width,
            height: window.size.height - self.window_size.height,
        };

        // Update the widget's position
        let widget_position_global_updated =
            self.get_updated_widget_position_global(change_of_origin, change_of_size);

        // Update the editor window's and textarea's dimensions.
        self.window_position = window.origin;
        self.window_size = window.size;

        self.widget_position
            .replace(Self::transform_global_position_to_local_position(
                self.window_position,
                widget_position_global_updated?,
                Some(window.origin),
            ));

        Some(())
    }

    pub fn update_textarea_dimensions(&mut self, textarea_global: LogicalFrame) -> Option<()> {
        // Transform the textarea's origin to local coordinates.
        let textarea = LogicalFrame {
            origin: Self::transform_global_position_to_local_position(
                self.window_position,
                textarea_global.origin,
                None,
            ),
            size: textarea_global.size,
        };

        let mut change_of_origin = None;
        let mut change_of_size = None;
        if let (Some(textarea_pos_old), Some(textarea_size_old)) =
            (self.textarea_position, self.textarea_size)
        {
            let textarea_pos_old_global = Self::transform_local_position_to_global_position(
                self.window_position,
                textarea_pos_old,
            );

            change_of_origin = Some(LogicalSize {
                width: textarea_global.origin.x - textarea_pos_old_global.x,
                height: textarea_global.origin.y - textarea_pos_old_global.y,
            });

            change_of_size = Some(LogicalSize {
                width: textarea.size.width - textarea_size_old.width,
                height: textarea.size.height - textarea_size_old.height,
            });
        }

        // Update the editor window's and textarea's dimensions.
        _ = self.set_textarea_dimensions(textarea.origin, textarea.size);
        self.widget_position
            .replace(Self::transform_global_position_to_local_position(
                self.window_position,
                self.get_updated_widget_position_global(change_of_origin?, change_of_size?)?,
                None,
            ));

        Some(())
    }
    pub fn update_focused_ui_element(
        &mut self,
        focused_ui_element: &FocusedUIElement,
        textarea_position_global: Option<LogicalPosition>,
        textarea_size: Option<LogicalSize>,
    ) {
        // Transforming the global position of the textarea to a local position.
        let textarea_position = if let Some(textarea_position) = textarea_position_global {
            Some(Self::transform_global_position_to_local_position(
                self.window_position,
                textarea_position,
                None,
            ))
        } else {
            None
        };

        if let (Some(position), Some(size)) = (textarea_position, textarea_size) {
            _ = self.set_textarea_dimensions(position, size);
        }
        self.focused_ui_element = Some(focused_ui_element.clone());
        self.check_and_update_dark_mode().ok();
    }

    pub fn update_widget_position(&mut self, widget_position_global: LogicalPosition) {
        // Transforming the global position of the widget to a local position.
        let widget_position = Self::transform_global_position_to_local_position(
            self.window_position,
            widget_position_global,
            None,
        );

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
        } else {
            panic!("Textarea position and size are not set; this should be impossible");
        }
    }

    fn get_updated_widget_position_global(
        &self,
        diff_pos: LogicalSize,
        diff_size: LogicalSize,
    ) -> Option<LogicalPosition> {
        let widget_pos = &mut self.widget_position?;

        // Determine how much each side/boundary moved
        let left_boundary_diff = diff_pos.width;
        let right_boundary_diff = diff_pos.width + diff_size.width;
        let bottom_boundary_diff = diff_pos.height + diff_size.height;
        let top_boundary_diff = diff_pos.height;

        match self.v_boundary {
            VerticalBoundary::Top => {
                widget_pos.y += top_boundary_diff;
            }
            VerticalBoundary::Bottom => {
                widget_pos.y += bottom_boundary_diff;
            }
        }

        match self.h_boundary {
            HorizontalBoundary::Left => {
                widget_pos.x += left_boundary_diff;
            }
            HorizontalBoundary::Right => {
                widget_pos.x += right_boundary_diff;
            }
        }

        Some(Self::transform_local_position_to_global_position(
            self.window_position,
            *widget_pos,
        ))
    }

    pub fn get_monitor(&self, app_handle: &tauri::AppHandle) -> Option<LogicalFrame> {
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

            if let Some(intersection_area) = Self::intersection_area(
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

    /// A global position is relative to the origin of the primary screen. A local position, in this context,
    /// is defined as a position relative to the top left corner of the editor window (=origin).
    fn transform_local_position_to_global_position(
        local_origin: LogicalPosition,
        local_position: LogicalPosition,
    ) -> LogicalPosition {
        let mut global_position = local_position;

        global_position.x += local_origin.x;
        global_position.y += local_origin.y;

        return global_position;
    }

    /// A global position is relative to the origin of the primary screen. A local position, in this context,
    /// is defined as a position relative to the top left corner of the editor window (=origin).
    fn transform_global_position_to_local_position(
        local_origin: LogicalPosition,
        global_position: LogicalPosition,
        updated_local_origin: Option<LogicalPosition>,
    ) -> LogicalPosition {
        let mut local_position = global_position;

        if let Some(updated_local_origin) = updated_local_origin {
            local_position.x -= updated_local_origin.x;
            local_position.y -= updated_local_origin.y;
        } else {
            local_position.x -= local_origin.x;
            local_position.y -= local_origin.y;
        }

        return local_position;
    }

    pub fn intersection_area(rect_a: LogicalFrame, rect_b: LogicalFrame) -> Option<f64> {
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
        if width <= 0.0 || height <= 0.0 {
            return None;
        }

        Some(width * height)
    }
}

#[cfg(test)]
mod tests_editor_window {
    use crate::{
        utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
        window_controls::windows::editor_window::editor_window::EditorWindow,
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
        let intersection = EditorWindow::intersection_area(rect_a, rect_b);
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
        let intersection = EditorWindow::intersection_area(rect_a, rect_b);
        assert_eq!(intersection, None);
    }

    #[test]
    fn test_transform_local_position_to_global_position() {
        let local_origin = LogicalPosition { x: 5.0, y: 3.0 };
        let local_position = LogicalPosition { x: 5.0, y: 5.0 };
        let global_position =
            EditorWindow::transform_local_position_to_global_position(local_origin, local_position);
        assert_eq!(global_position, LogicalPosition { x: 10.0, y: 8.0 });
    }

    #[test]
    fn test_transform_global_position_to_local_position() {
        let local_origin = LogicalPosition { x: 10.2, y: 22.2 };
        let global_position = LogicalPosition { x: -5.0, y: 5.0 };
        let updated_local_origin = None;
        let local_position = EditorWindow::transform_global_position_to_local_position(
            local_origin,
            global_position,
            updated_local_origin,
        );
        assert_eq!(local_position, LogicalPosition { x: -15.2, y: -17.2 });
    }
}
