use std::sync::Arc;

use cocoa::{base::id, foundation::NSInteger};
use objc::{msg_send, sel, sel_impl};

use parking_lot::Mutex;
use tauri::Manager;
use window_shadows::set_shadow;

use crate::{
    app_handle,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::{default_properties, AppWindow, WindowLevel},
        models::TrackingAreaClickedOutsideMessage,
        utils::{create_default_window_builder, get_position, get_size},
        EventTrackingArea, TrackingArea, TrackingEventSubscription, TrackingEventType,
    },
};

use super::listeners::window_control_events_listener;

static Y_OFFSET: f64 = 16.;

#[derive(Clone, Debug)]
pub struct ExplainWindow {
    app_handle: tauri::AppHandle,
    tracking_area: Option<TrackingArea>,
    editor_textarea: Option<LogicalFrame>,
    monitor_frame: Option<LogicalFrame>,
    window_frame: LogicalFrame,
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
            editor_textarea: None,
            monitor_frame: None,
            window_frame: LogicalFrame {
                origin: LogicalPosition { x: 0., y: 0. },
                size: LogicalSize {
                    width: default_properties::size(&AppWindow::Explain).0,
                    height: default_properties::size(&AppWindow::Explain).1,
                },
            },
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
        annotation_anchor: Option<LogicalFrame>,
        editor_textarea: &LogicalFrame,
        monitor_frame: &LogicalFrame,
    ) -> Option<()> {
        self.editor_textarea.replace(editor_textarea.to_owned());
        self.monitor_frame.replace(editor_textarea.to_owned());
        self.window_frame = LogicalFrame {
            origin: get_position(AppWindow::Explain)?,
            size: get_size(AppWindow::Explain)?,
        };

        // 0. Start with the optimal position
        let mut optimal_position = self.window_frame.origin;
        if let Some(anchor) = annotation_anchor {
            optimal_position = LogicalPosition {
                x: anchor.origin.x - self.window_frame.size.width,
                y: anchor.origin.y - Y_OFFSET,
            };
        }

        println!("optimal_position: {:?}", optimal_position);

        // 1. Derive valid area of the screen where to put the explain window.
        let valid_monitor_area = LogicalFrame {
            origin: LogicalPosition {
                x: monitor_frame.origin.x,
                y: f64::max(monitor_frame.origin.y, editor_textarea.origin.y),
            },
            size: LogicalSize {
                width: monitor_frame.size.width,
                height: f64::min(
                    editor_textarea.size.height,
                    monitor_frame.size.height
                        - f64::abs(editor_textarea.origin.y - monitor_frame.origin.y),
                ),
            },
        };

        println!("valid_monitor_area: {:?}", valid_monitor_area);

        // 2. Compute the diff to prevent drawing off-screen.
        let (offscreen_dist_x, offscreen_dist_y) =
            Self::calc_off_screen_distance(&self.window_frame, &valid_monitor_area);

        if let Some(offscreen_dist_x) = offscreen_dist_x {
            optimal_position.x += offscreen_dist_x;
        }

        if let Some(offscreen_dist_y) = offscreen_dist_y {
            optimal_position.y += offscreen_dist_y;
        }

        println!(
            "offscreen_dist_x: {:?}, offscreen_dist_y: {:?}",
            offscreen_dist_x, offscreen_dist_y
        );

        // 3. Check if there is overlap between the repositioned explain window and the annotation frame.
        if let Some(anchor) = annotation_anchor {
            if Self::intersection_area(&self.window_frame, &anchor).is_some() {
                // 3.1. If there is overlap, check if there is enough space above the annotation frame.
                if self.window_frame.size.height <= valid_monitor_area.origin.y - anchor.origin.y {
                    // 3.1.1. If there is enough space, move the explain window above the annotation frame.
                    optimal_position.y = anchor.origin.y - self.window_frame.size.height;
                } else {
                    // 3.1.2. If there is not enough space, move the explain window below the annotation frame.
                    optimal_position.y = anchor.origin.y + anchor.size.height;
                }
            }
        }

        println!("updated optimal_position: {:?}", optimal_position);

        self.window_frame = LogicalFrame {
            origin: optimal_position,
            size: self.window_frame.size,
        };

        // 4. Create Tracking Area to detect clicks outside the explain window.
        self.create_tracking_area()?;

        let tauri_window = self
            .app_handle
            .get_window(&AppWindow::Explain.to_string())?;

        tauri_window
            .set_position(self.window_frame.origin.as_tauri_LogicalPosition())
            .ok()?;

        set_shadow(&tauri_window, false).expect("Unsupported platform!");

        println!("showing explain window");

        tauri_window.show().ok()?;

        println!("showed explain window");

        Some(())
    }

    pub fn hide(&mut self) -> Option<()> {
        if let Some(tracking_area) = self.tracking_area.as_ref() {
            EventTrackingArea::Remove(vec![tracking_area.id]).publish_to_tauri(&app_handle());
            self.tracking_area = None;
        }

        _ = self
            .app_handle
            .get_window(&AppWindow::Explain.to_string())?
            .hide();

        Some(())
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

    fn create_tracking_area(&mut self) -> Option<()> {
        let tracking_area = TrackingArea {
            id: uuid::Uuid::new_v4(),
            window_uid: 0,
            rectangles: vec![self.window_frame],
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

    pub fn calc_off_screen_distance(
        window: &LogicalFrame,
        valid_monitor_area: &LogicalFrame,
    ) -> (Option<f64>, Option<f64>) {
        let mut dist_x: Option<f64> = None;
        let mut dist_y: Option<f64> = None;

        println!("window: {:?}", window);
        println!("valid_monitor_area: {:?}", valid_monitor_area);

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

    fn intersection_area(rect_a: &LogicalFrame, rect_b: &LogicalFrame) -> Option<f64> {
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
}
