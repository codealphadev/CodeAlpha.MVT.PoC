use tauri::Manager;

use crate::{
    app_handle,
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    window_controls::{
        config::{default_properties, AppWindow},
        EventTrackingArea, TrackingArea, TrackingEventSubscriber, TrackingEventType,
        TrackingEvents,
    },
};

pub fn app_window_dimensions(app_window: AppWindow) -> LogicalFrame {
    let tauri_window = app_handle()
        .get_window(&app_window.to_string())
        .expect(&format!(
            "Could not get window: {:?}!",
            app_window.to_string()
        ));

    let scale_factor = tauri_window.scale_factor().expect(&format!(
        "Could not get window: {:?} scale factor!",
        app_window.to_string()
    ));
    let app_window_position = LogicalPosition::from_tauri_LogicalPosition(
        &tauri_window
            .outer_position()
            .expect(&format!(
                "Could not get window: {:?} outer position!",
                app_window.to_string()
            ))
            .to_logical::<f64>(scale_factor),
    );
    let app_window_size = LogicalSize::from_tauri_LogicalSize(
        &tauri_window
            .outer_size()
            .expect(&format!(
                "Could not get window: {:?} outer size!",
                app_window.to_string()
            ))
            .to_logical::<f64>(scale_factor),
    );

    LogicalFrame {
        origin: app_window_position,
        size: app_window_size,
    }
}

pub fn update_tracking_area(
    app_window: AppWindow,
    existing_tracking_area: TrackingArea,
    is_visible: bool,
) {
    if is_visible {
        let app_window_frame = app_window_dimensions(app_window);
        let mut tracking_area = existing_tracking_area.clone();
        tracking_area.rectangle = LogicalFrame {
            origin: LogicalPosition::default(),
            size: app_window_frame.size,
        };
        EventTrackingArea::Update(vec![tracking_area.clone()]).publish_to_tauri(&app_handle());
    } else {
        let mut tracking_area = existing_tracking_area.clone();
        tracking_area.rectangle = LogicalFrame::default();
        EventTrackingArea::Update(vec![tracking_area.clone()]).publish_to_tauri(&app_handle());
    }
}

pub fn register_tracking_area(app_window: AppWindow) -> TrackingArea {
    let app_window_frame = if default_properties::is_visible(&app_window) {
        app_window_dimensions(app_window)
    } else {
        LogicalFrame::default()
    };

    let tracking_area = TrackingArea {
        id: uuid::Uuid::new_v4(),
        window_uid: 0,
        rectangle: LogicalFrame {
            origin: LogicalPosition::default(),
            size: app_window_frame.size,
        },
        events: TrackingEvents::TrackingEventTypes(vec![
            TrackingEventType::MouseOver,
            TrackingEventType::MouseEntered,
            TrackingEventType::MouseExited,
            TrackingEventType::MouseClicked,
        ]),
        app_window,
        subscriber: vec![TrackingEventSubscriber::AppWindow(app_window)],
    };

    // 3. Publish to the tracking area manager with its original GLOBAL coordinates
    EventTrackingArea::Add(vec![tracking_area.clone()]).publish_to_tauri(&app_handle());

    tracking_area
}
