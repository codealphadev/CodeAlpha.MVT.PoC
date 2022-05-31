#![allow(dead_code)]

use cocoa::{base::id, foundation::NSInteger};
use objc::{msg_send, sel, sel_impl};
use tauri::{Error, LogicalPosition, LogicalSize, Manager};

use crate::ax_interaction::models::{
    app::ContentWindowState,
    editor::{EditorWindowCreatedMessage, FocusedUIElement},
};

use super::{
    actions::{close_window, open_window, resize_window, set_position},
    AppWindow,
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
    textarea_position: Option<tauri::LogicalPosition<f64>>,
    textarea_size: Option<tauri::LogicalSize<f64>>,

    /// Widget position to the editor's text area.
    pub widget_position: Option<tauri::LogicalPosition<f64>>,

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

    pub fn update_window_dimensions(
        &mut self,
        window_position: tauri::LogicalPosition<f64>,
        window_size: tauri::LogicalSize<f64>,
        textarea_position: Option<tauri::LogicalPosition<f64>>,
        textarea_size: Option<tauri::LogicalSize<f64>>,
    ) {
        // Calculate the change of the window dimensions
        let (diff_pos, diff_size) = self.calculate_dimension_change(
            window_position,
            window_size,
            textarea_position,
            textarea_size,
        );

        // Update textarea dimension
        self.update_textarea_dimensions(diff_pos, diff_size, textarea_position, textarea_size);

        // Update widget position
        self.calc_widget_pos_by_respecting_boundaries(diff_pos, diff_size);

        self.window_position = window_position;
        self.window_size = window_size;
    }

    pub fn update_focused_ui_element(
        &mut self,
        focused_ui_element: &FocusedUIElement,
        textarea_position: Option<tauri::LogicalPosition<f64>>,
        textarea_size: Option<tauri::LogicalSize<f64>>,
    ) {
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

    pub fn update_widget_position(&mut self, widget_position: tauri::LogicalPosition<f64>) {
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
        diff_pos: tauri::LogicalSize<f64>,
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
                    x: textarea_pos.x + diff_pos.width,
                    y: textarea_pos.y + diff_pos.height,
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

    /// It opens the code overlay window and sets its position and size to match the textarea
    ///
    /// Arguments:
    ///
    /// * `app_handle`: The handle to the application.
    ///
    /// Returns:
    ///
    /// A Result<(), Error>
    pub fn show_code_overlay(&self, app_handle: &tauri::AppHandle) -> Result<(), Error> {
        if let (Some(origin), Some(size)) = (self.textarea_position, self.textarea_size) {
            resize_window(app_handle, AppWindow::CodeOverlay, &size)?;
            set_position(app_handle, AppWindow::CodeOverlay, &origin)?;

            open_window(app_handle, AppWindow::CodeOverlay);

            app_handle.emit_to(
                &AppWindow::CodeOverlay.to_string(),
                "event-compute-height",
                {},
            )?;
        }

        Self::configure_code_overlay_properties(app_handle);

        Ok(())
    }

    /// It closes the code overlay window
    ///
    /// Arguments:
    ///
    /// * `app_handle`: The handle to the tauri app.
    pub fn hide_code_overlay(&self, app_handle: &tauri::AppHandle) {
        close_window(app_handle, AppWindow::CodeOverlay);
    }

    /// It gets the window handle for the code overlay window, and then sets the `ignoresMouseEvents`
    /// property to `true`
    ///
    /// Arguments:
    ///
    /// * `app_handle`: The app handle that you can get from the tauri::AppBuilder.
    fn configure_code_overlay_properties(app_handle: &tauri::AppHandle) {
        if let (Some(overlay_window), Some(widget_window)) = (
            app_handle.get_window(&AppWindow::CodeOverlay.to_string()),
            app_handle.get_window(&AppWindow::Widget.to_string()),
        ) {
            if let (Ok(ns_window_ptr_overlay), Ok(ns_window_ptr_widget)) =
                (overlay_window.ns_window(), widget_window.ns_window())
            {
                // Setting the mouse events to be ignored for the overlay window.
                unsafe {
                    if !msg_send![ns_window_ptr_overlay as id, ignoresMouseEvents] {
                        let _: () =
                            msg_send![ns_window_ptr_overlay as id, setIgnoresMouseEvents: true];
                    }
                }

                // Ordering the widget window to the front. This prevents overlap.
                unsafe {
                    let overlay_window_level: i64 = msg_send![ns_window_ptr_overlay as id, level];

                    let _: () = msg_send![
                        ns_window_ptr_widget as id,
                        setLevel: overlay_window_level + 1 as NSInteger
                    ];
                }
            }
        }
    }
}
