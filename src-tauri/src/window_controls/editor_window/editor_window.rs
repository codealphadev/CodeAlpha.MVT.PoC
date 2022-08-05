use tauri::{LogicalPosition, LogicalSize, Manager};

use crate::{
    ax_interaction::models::{
        app::ContentWindowState,
        editor::{EditorWindowCreatedMessage, FocusedUIElement},
    },
    window_controls::config::AppWindow,
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
    pub id: uuid::Uuid,
    /// The reference to the AXUIElement of the editor window.
    pub uielement_hash: usize,

    #[allow(dead_code)]
    /// The application name of the editor this window belongs to. For XCode it is "Xcode".
    editor_name: String,

    /// The process identifier for the window's editor application.
    pub pid: i32,

    /// This enum holds the last type of the last focused ui element in this editor window.
    pub focused_ui_element: Option<FocusedUIElement>,

    /// The editor window's dimensions.
    window_position: tauri::LogicalPosition<f64>,
    window_size: tauri::LogicalSize<f64>,

    /// The Text Area is the ui element within xcode that is used for editing code
    /// When initially focusing an editor window the text area might not be visible,
    /// wherefore it's dimension might not be known.
    /// The coordinates of the textarea's origin are relative to the editor window.
    textarea_position: Option<tauri::LogicalPosition<f64>>,
    textarea_size: Option<tauri::LogicalSize<f64>>,

    /// Widget position to the editor's text area.
    widget_position: Option<tauri::LogicalPosition<f64>>,

    /// Enum to indicate if the content window is currently active or inactive.
    /// Because of the parent-child relationship between widget and content window the
    /// content window gets hidden as well when the widget is hidden. But if the widget
    /// is only temporarily hidden we need to know if the content window is active or inactive
    /// to correctly position the widget before bringing it back.
    pub content_window_state: ContentWindowState,

    /// When the editor text area's size or position is updated, the widget_position
    /// is recalculated with respect to the boundaries. The boundaries are initially set to bottom|right
    /// but get updated each time the user moves the widget manually
    h_boundary: HorizontalBoundary,
    v_boundary: VerticalBoundary,
}

impl EditorWindow {
    pub fn new(created_msg: &EditorWindowCreatedMessage) -> Self {
        Self {
            id: created_msg.id,
            uielement_hash: created_msg.ui_elem_hash,
            editor_name: created_msg.editor_name.clone(),
            pid: created_msg.pid,
            window_position: created_msg.window_position,
            window_size: created_msg.window_size,
            textarea_position: None,
            textarea_size: None,
            focused_ui_element: None,
            h_boundary: HorizontalBoundary::Right,
            v_boundary: VerticalBoundary::Bottom,
            widget_position: None,
            content_window_state: ContentWindowState::Inactive,
        }
    }

    pub fn textarea_position(
        &self,
        as_global_position: bool,
    ) -> Option<tauri::LogicalPosition<f64>> {
        if let Some(textarea_position) = self.textarea_position {
            if as_global_position {
                Some(self.transform_local_position_to_global_position(textarea_position))
            } else {
                Some(textarea_position)
            }
        } else {
            None
        }
    }

    pub fn textarea_size(&self) -> Option<tauri::LogicalSize<f64>> {
        self.textarea_size
    }

    pub fn widget_position(&self, as_global_position: bool) -> Option<tauri::LogicalPosition<f64>> {
        if let Some(widget_position) = self.widget_position {
            if as_global_position {
                Some(self.transform_local_position_to_global_position(widget_position))
            } else {
                Some(widget_position)
            }
        } else {
            None
        }
    }

    pub fn update_window_dimensions(
        &mut self,
        window_position: tauri::LogicalPosition<f64>,
        window_size: tauri::LogicalSize<f64>,
        textarea_position_global: Option<tauri::LogicalPosition<f64>>,
        textarea_size: Option<tauri::LogicalSize<f64>>,
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

    pub fn update_focused_ui_element(
        &mut self,
        focused_ui_element: &FocusedUIElement,
        textarea_position_global: Option<tauri::LogicalPosition<f64>>,
        textarea_size: Option<tauri::LogicalSize<f64>>,
    ) {
        // Transforming the global position of the textarea to a local position.
        let textarea_position = if let Some(textarea_position) = textarea_position_global {
            Some(self.transform_global_position_to_local_position(textarea_position, None))
        } else {
            None
        };

        if let Some(position) = textarea_position {
            self.textarea_position = Some(position);
        }

        if let Some(size) = textarea_size {
            self.textarea_size = Some(size);
        }

        self.focused_ui_element = Some(focused_ui_element.clone());

        // In case no widget position is set yet, initialize widget position on editor textarea
        self.initialize_widget_position();
    }

    pub fn update_content_window_state(&mut self, content_window_state: &ContentWindowState) {
        self.content_window_state = *content_window_state;
    }

    pub fn update_widget_position(&mut self, widget_position_global: tauri::LogicalPosition<f64>) {
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

    fn calculate_dimension_change(
        &self,
        window_position: tauri::LogicalPosition<f64>,
        window_size: tauri::LogicalSize<f64>,
        textarea_position: Option<tauri::LogicalPosition<f64>>,
        textarea_size: Option<tauri::LogicalSize<f64>>,
    ) -> (LogicalSize<f64>, LogicalSize<f64>) {
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
        diff_size: tauri::LogicalSize<f64>,
        textarea_position: Option<tauri::LogicalPosition<f64>>,
        textarea_size: Option<tauri::LogicalSize<f64>>,
    ) {
        if let (Some(textarea_pos_new), Some(textarea_size_new)) =
            (textarea_position, textarea_size)
        {
            self.textarea_position = Some(textarea_pos_new);
            self.textarea_size = Some(textarea_size_new);
        } else {
            // Case: valid updated textarea dimensions are provided;
            // Case: Deriving updated textarea dimensions from window dimension change;
            if let (Some(textarea_pos), Some(textarea_size)) =
                (self.textarea_position, self.textarea_size)
            {
                self.textarea_position = Some(LogicalPosition {
                    x: textarea_pos.x,
                    y: textarea_pos.y,
                });

                self.textarea_size = Some(LogicalSize {
                    width: textarea_size.width + diff_size.width,
                    height: textarea_size.height + diff_size.height,
                });
            }
        }
    }

    fn calc_widget_pos_by_respecting_boundaries(
        &mut self,
        diff_pos: tauri::LogicalSize<f64>,
        diff_size: tauri::LogicalSize<f64>,
    ) {
        if let Some(widget_pos) = &mut self.widget_position {
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
        } else {
            // In case no widget position is set yet, initialize widget position on editor textarea
            self.initialize_widget_position();
        }
    }

    fn initialize_widget_position(&mut self) {
        // In case no widget position is set yet, initialize widget position on editor textarea
        if self.widget_position.is_none() {
            if let (Some(textarea_pos), Some(textarea_size)) =
                (self.textarea_position, self.textarea_size)
            {
                self.widget_position = Some(LogicalPosition {
                    x: textarea_pos.x + textarea_size.width - 100.,
                    y: textarea_pos.y + textarea_size.height - 100.,
                });
            }
        }
    }

    fn transform_global_position_to_local_position(
        &self,
        global_position: tauri::LogicalPosition<f64>,
        local_origin: Option<tauri::LogicalPosition<f64>>,
    ) -> tauri::LogicalPosition<f64> {
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

    fn transform_local_position_to_global_position(
        &self,
        local_position: tauri::LogicalPosition<f64>,
    ) -> tauri::LogicalPosition<f64> {
        let mut global_position = local_position;

        global_position.x += &self.window_position.x;
        global_position.y += &self.window_position.y;

        return global_position;
    }

    /// If the widget window is available, check if the textarea's origin is within the bounds of any of the
    /// available monitors. If it is, return the monitor
    ///
    /// Arguments:
    ///
    /// * `app_handle`: The handle to the application.
    ///
    /// Returns:
    ///
    /// A `tauri::Monitor` object.
    pub fn get_monitor_for_editor_window(
        &self,
        app_handle: &tauri::AppHandle,
    ) -> Option<tauri::Monitor> {
        if let Some(widget_window) = app_handle.get_window(&AppWindow::Widget.to_string()) {
            if let Ok(available_monitors) = widget_window.available_monitors() {
                for monitor in available_monitors {
                    let scale_factor = monitor.scale_factor();
                    let origin = monitor.position().to_logical::<f64>(scale_factor);
                    let size = monitor.size().to_logical::<f64>(scale_factor);

                    let textarea_position_global =
                        self.transform_local_position_to_global_position(self.window_position);

                    if origin.x <= textarea_position_global.x
                        && origin.y <= textarea_position_global.y
                        && origin.x + size.width >= textarea_position_global.x
                        && origin.y + size.height >= textarea_position_global.y
                    {
                        return Some(monitor);
                    }
                }
            }
        }

        None
    }
}
