use std::sync::Arc;

use cocoa::{base::id, foundation::NSInteger};
use objc::{msg_send, sel, sel_impl};

use parking_lot::Mutex;
use tauri::Manager;
use window_shadows::set_shadow;

use crate::{
    app_handle,
    platform::macos::{CodeDocumentFrameProperties, ViewportProperties},
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::{default_properties, AppWindow, WindowLevel},
        models::TrackingAreaClickedOutsideMessage,
        utils::create_default_window_builder,
        windows::EditorWindow,
        EventTrackingArea, TrackingArea, TrackingEventSubscription, TrackingEventType,
    },
};

use super::listeners::window_control_events_listener;

static Y_OFFSET: f64 = 0.;

#[derive(Clone, Debug)]
pub struct ExplainWindow {
    app_handle: tauri::AppHandle,
    tracking_area: Option<TrackingArea>,
    window_size: LogicalSize,
    window_origin_local: Option<LogicalPosition>, // The origin of the window relative to the code document frame.
    viewport_props: Option<ViewportProperties>,
    code_document_props: Option<CodeDocumentFrameProperties>,
    monitor: Option<LogicalFrame>,
    annotation_area: Option<LogicalFrame>, // The origin of the annotation area relative to the viewport frame.
}

impl ExplainWindow {
    pub fn new() -> Result<Self, tauri::Error> {
        let app_handle = app_handle();

        if app_handle
            .get_window(&AppWindow::Explain.to_string())
            .is_none()
        {
            let window_builder = create_default_window_builder(&app_handle, AppWindow::Explain)?;
            let _window = window_builder.build()?;

            // #[cfg(debug_assertions)] // only include this code on debug builds
            // window.open_devtools();
        }

        Ok(Self {
            app_handle,
            tracking_area: None,
            monitor: None,
            window_size: LogicalSize {
                width: default_properties::size(&AppWindow::Explain).0,
                height: default_properties::size(&AppWindow::Explain).1,
            },
            viewport_props: None,
            code_document_props: None,
            window_origin_local: None,
            annotation_area: None,
        })
    }

    pub fn set_macos_properties(&self) -> Option<()> {
        let ns_window_ptr_explain = self
            .app_handle
            .get_window(&AppWindow::Explain.to_string())?
            .ns_window();

        if let Ok(ns_window_ptr_explain) = ns_window_ptr_explain {
            unsafe {
                // Set the explain window's window level
                let _: () = msg_send![
                    ns_window_ptr_explain as id,
                    setLevel: WindowLevel::FloatingCard as NSInteger
                ];

                // Preventing the explain window from activating our activated.
                let _: () = msg_send![ns_window_ptr_explain as id, _setPreventsActivation: true];
            }
        }

        Some(())
    }

    pub fn start_event_listeners(explain_window: &Arc<Mutex<ExplainWindow>>) {
        window_control_events_listener(explain_window);
    }

    pub fn show(
        &mut self,
        annotation_area: Option<LogicalFrame>,
        viewport: &ViewportProperties,
        code_document: &CodeDocumentFrameProperties,
        monitor: &LogicalFrame,
    ) -> Option<()> {
        self.monitor.replace(monitor.to_owned());
        self.viewport_props.replace(viewport.to_owned());
        self.code_document_props.replace(code_document.to_owned());

        if let Some(annotation_area) = annotation_area {
            // Update the explain window's origin in local coordinates relative to code document frame.
            let local_origin = LogicalPosition {
                x: annotation_area.origin.x - self.window_size.width,
                y: annotation_area.origin.y - Y_OFFSET,
            }
            .to_local(&self.get_coordinate_system_origin()?);

            self.window_origin_local.replace(local_origin);

            // Only set the annotation area if it's not None to not throw away a previously known annotation area.
            self.annotation_area
                .replace(annotation_area.to_local(&self.get_coordinate_system_origin()?));
        }

        // Derive an origin for the explain window following a set of positioning rules.
        let corrected_window_origin_global = self.corrected_window_origin_global()?;

        // Create Tracking Area to detect clicks outside the explain window.
        self.create_tracking_area(&LogicalFrame {
            origin: corrected_window_origin_global,
            size: self.window_size,
        })?;

        let tauri_window = self
            .app_handle
            .get_window(&AppWindow::Explain.to_string())?;

        tauri_window
            .set_position(corrected_window_origin_global.as_tauri_LogicalPosition())
            .ok()?;

        set_shadow(&tauri_window, true).expect("Unsupported platform!");

        tauri_window.show().ok()?;

        Some(())
    }

    pub fn hide(&mut self) -> Option<()> {
        if let Some(tracking_area) = self.tracking_area.as_ref() {
            EventTrackingArea::Remove(vec![tracking_area.id]).publish_to_tauri(&app_handle());
        }

        _ = self
            .app_handle
            .get_window(&AppWindow::Explain.to_string())?
            .hide();

        // Reset properties
        self.window_origin_local = None;
        self.annotation_area = None;
        self.monitor = None;
        self.tracking_area = None;

        Some(())
    }

    pub fn update(
        &mut self,
        viewport: &Option<ViewportProperties>,
        code_document: &Option<CodeDocumentFrameProperties>,
        window_position_global: &Option<LogicalPosition>,
        window_size: &Option<LogicalSize>,
    ) -> Option<()> {
        if let (Some(viewport), Some(code_document)) = (viewport, code_document) {
            self.update_editor_properties(viewport, code_document);
        }

        if let Some(position_global) = window_position_global {
            self.update_window_origin_local(&position_global)?;
        }

        if let Some(window_size) = window_size {
            self.window_size = window_size.to_owned();

            let tauri_window = self
                .app_handle
                .get_window(&AppWindow::Explain.to_string())?;

            tauri_window
                .set_size(window_size.as_tauri_LogicalSize())
                .ok()?;
        }

        Some(())
    }

    fn update_window_origin_local(&mut self, window_origin_global: &LogicalPosition) -> Option<()> {
        let local_origin = window_origin_global.to_local(&self.get_coordinate_system_origin()?);

        self.window_origin_local.replace(local_origin);

        Some(())
    }

    fn update_editor_properties(
        &mut self,
        viewport: &ViewportProperties,
        code_document: &CodeDocumentFrameProperties,
    ) -> Option<()> {
        self.viewport_props.replace(viewport.to_owned());
        self.code_document_props.replace(code_document.to_owned());

        // Derive an origin for the explain window following a set of positioning rules.
        let corrected_global_origin = self.corrected_window_origin_global()?;

        // Create Tracking Area to detect clicks outside the explain window.
        self.update_tracking_area(&LogicalFrame {
            origin: corrected_global_origin,
            size: self.window_size,
        })?;

        let tauri_window = self
            .app_handle
            .get_window(&AppWindow::Explain.to_string())?;

        tauri_window
            .set_position(corrected_global_origin.as_tauri_LogicalPosition())
            .ok()?;

        Some(())
    }

    fn corrected_window_origin_global(&self) -> Option<LogicalPosition> {
        // 1. Derive valid area of the screen where to put the explain window.
        let valid_monitor_area = LogicalFrame {
            origin: LogicalPosition {
                x: self.monitor?.origin.x,
                y: f64::max(
                    self.monitor?.origin.y,
                    self.viewport_props.as_ref()?.dimensions.origin.y,
                ),
            },
            size: LogicalSize {
                width: self.monitor?.size.width,
                height: f64::min(
                    self.viewport_props.as_ref()?.dimensions.size.height,
                    self.monitor?.size.height
                        - f64::abs(
                            self.viewport_props.as_ref()?.dimensions.origin.y
                                - self.monitor?.origin.y,
                        ),
                ),
            },
        };

        let mut corrected_global_origin = self
            .window_origin_local?
            .to_global(&self.get_coordinate_system_origin()?);

        // 2. Compute the diff to prevent drawing off-screen.
        let (offscreen_dist_x, offscreen_dist_y) = Self::calc_off_screen_distance(
            &LogicalFrame {
                origin: corrected_global_origin,
                size: self.window_size,
            },
            &valid_monitor_area,
        );

        if let Some(offscreen_dist_x) = offscreen_dist_x {
            corrected_global_origin.x += offscreen_dist_x;
        }

        if let Some(offscreen_dist_y) = offscreen_dist_y {
            corrected_global_origin.y += offscreen_dist_y;
        }

        let annotation_area_global = self
            .annotation_area?
            .to_global(&self.get_coordinate_system_origin()?);

        // 3. Check if there is overlap between the repositioned explain window and the annotation frame.
        if EditorWindow::intersection_area(
            LogicalFrame {
                origin: corrected_global_origin,
                size: self.window_size,
            },
            annotation_area_global,
        )
        .is_some()
        {
            // 3.1. If there is overlap, check if there is enough space above the annotation frame.
            if self.window_size.height
                <= f64::abs(annotation_area_global.origin.y - valid_monitor_area.origin.y)
            {
                // 3.1.1. If there is enough space, move the explain window above the annotation frame.
                corrected_global_origin.y =
                    annotation_area_global.origin.y - self.window_size.height;
            } else {
                // 3.1.2. If there is not enough space, move the explain window below the annotation frame.
                corrected_global_origin.y =
                    annotation_area_global.origin.y + annotation_area_global.size.height;
            }
        }

        Some(corrected_global_origin)
    }

    pub fn clicked_outside(
        &mut self,
        clicked_outside_msg: &TrackingAreaClickedOutsideMessage,
    ) -> Option<()> {
        if let Some(tracking_area) = self.tracking_area.as_ref() {
            if tracking_area.id == clicked_outside_msg.id {
                self.hide();
            }
        }

        Some(())
    }

    fn update_tracking_area(&mut self, updated_tracking_rect: &LogicalFrame) -> Option<()> {
        let mut tracking_area = self.tracking_area.as_ref()?.clone();
        tracking_area.rectangles = vec![updated_tracking_rect.to_owned()];

        EventTrackingArea::Update(vec![tracking_area.clone()]).publish_to_tauri(&app_handle());

        self.tracking_area.replace(tracking_area);

        Some(())
    }

    fn create_tracking_area(&mut self, tracking_rect: &LogicalFrame) -> Option<()> {
        let tracking_area = TrackingArea {
            id: uuid::Uuid::new_v4(),
            window_uid: 0,
            rectangles: vec![tracking_rect.to_owned()],
            event_subscriptions: TrackingEventSubscription::TrackingEventTypes(vec![
                TrackingEventType::MouseClickedOutside,
            ]),
            app_window: AppWindow::Explain,
        };

        // Publish to the tracking area manager
        EventTrackingArea::Add(vec![tracking_area.clone()]).publish_to_tauri(&app_handle());

        self.tracking_area.replace(tracking_area);

        Some(())
    }

    fn get_coordinate_system_origin(&self) -> Option<LogicalPosition> {
        // To prevent the window from scrolling horizontally when the user scrolls the code document,
        // we define an artificial coordinate system origin.

        Some(LogicalPosition {
            x: self.viewport_props.as_ref()?.dimensions.origin.x,
            y: self.code_document_props.as_ref()?.dimensions.origin.y,
        })
    }

    pub fn calc_off_screen_distance(
        window: &LogicalFrame,
        valid_monitor_area: &LogicalFrame,
    ) -> (Option<f64>, Option<f64>) {
        let mut dist_x: Option<f64> = None;
        let mut dist_y: Option<f64> = None;

        // prevent widget from going off-screen
        if window.origin.x < valid_monitor_area.origin.x {
            dist_x = Some(valid_monitor_area.origin.x - window.origin.x);
        }
        if window.origin.y < valid_monitor_area.origin.y {
            dist_y = Some(valid_monitor_area.origin.y - window.origin.y);
        }
        if window.origin.x + window.size.width
            > valid_monitor_area.origin.x + valid_monitor_area.size.width
        {
            dist_x = Some(
                valid_monitor_area.origin.x + valid_monitor_area.size.width
                    - window.size.width
                    - window.origin.x,
            );
        }
        if window.origin.y + window.size.height
            > valid_monitor_area.origin.y + valid_monitor_area.size.height
        {
            dist_y = Some(
                valid_monitor_area.origin.y + valid_monitor_area.size.height
                    - window.size.height
                    - window.origin.y,
            );
        }

        (dist_x, dist_y)
    }
}
