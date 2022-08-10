use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::{
    app_handle,
    core_engine::{
        rules::TextRange,
        syntax_tree::{SwiftCodeBlock, SwiftSyntaxTree},
        types::MatchRectangle,
    },
    utils::{
        geometry::{LogicalPosition, LogicalSize},
        messaging::ChannelList,
    },
    window_controls::{
        code_overlay::{EventTrackingArea, TrackingArea, TrackingEvent, TrackingEventSubscription},
        events::EventWindowControls,
    },
};

pub struct DocsGenerator {
    swift_syntax_tree: SwiftSyntaxTree,
    text_content: Option<String>,
    tracking_area: Option<TrackingArea>,
    window_pid: i32,
}

impl DocsGenerator {
    pub fn new(window_pid: i32) -> Self {
        Self {
            swift_syntax_tree: SwiftSyntaxTree::new(),
            text_content: None,
            window_pid,
            tracking_area: None,
        }
    }

    pub fn update_content(&mut self, text_content: &String) {
        if self.swift_syntax_tree.parse(text_content) {
            self.text_content = Some(text_content.to_owned());
        }
    }

    pub fn update_selected_text_range(&mut self, selected_text_range: Option<TextRange>) {
        // 1. Get Codeblock
        let codeblock_node = self
            .swift_syntax_tree
            .get_selected_codeblock_node(&selected_text_range.unwrap());

        // 2. Derive the TrackingArea for positioning (and tracking) of the code annotation icon
        if let Some(codeblock_node) = codeblock_node {
            if let Some(tracking_area) = self.derive_annotation_tracking_area(&codeblock_node) {
                // 3. Send the TrackingArea to the window controls
                if self.tracking_area.is_none() {
                    EventTrackingArea::Add(vec![tracking_area.clone()])
                        .publish_to_tauri(&app_handle());
                } else {
                    EventTrackingArea::Update(vec![tracking_area.clone()])
                        .publish_to_tauri(&app_handle());
                }

                // 4. Send the TrackingArea to the frontend
                todo!();

                // 5. Store the TrackingArea for future clicks
                self.tracking_area = Some(tracking_area);
            }
        }
    }

    fn derive_annotation_tracking_area(
        &self,
        codeblock_node: &SwiftCodeBlock,
    ) -> Option<TrackingArea> {
        // 3. calculate position for the annotation icon
        let single_char_bounds = codeblock_node.get_height_of_single_char(self.window_pid);

        // 4. calculate the bounds of the codeblock
        let codeblock_bounds = codeblock_node.get_codeblock_node_bounds(self.window_pid);

        if let (Some(single_char_bounds), Some(codeblock_bounds)) =
            (single_char_bounds, codeblock_bounds)
        {
            let annotation_position = LogicalPosition {
                x: codeblock_bounds.origin.x - single_char_bounds.size.width,
                y: codeblock_bounds.origin.y,
            };
            let annotation_size = LogicalSize {
                width: single_char_bounds.size.width,
                height: single_char_bounds.size.height,
            };

            let uuid = if let Some(tracking_area) = self.tracking_area.as_ref() {
                tracking_area.id
            } else {
                uuid::Uuid::new_v4()
            };

            let tracking_area = TrackingArea {
                id: uuid,
                rectangles: vec![MatchRectangle {
                    origin: annotation_position,
                    size: annotation_size,
                }],
                event_subscriptions: TrackingEventSubscription::TrackingEvent(vec![
                    TrackingEvent::MouseClicked,
                ]),
            };
            Some(tracking_area)
        } else {
            None
        }
    }

    pub fn start_listener_window_control_events(
        app_handle: &tauri::AppHandle,
        docs_generator: &Arc<Mutex<Self>>,
    ) {
        let docs_generator_move_copy = (docs_generator).clone();
        app_handle.listen_global(ChannelList::EventWindowControls.to_string(), move |msg| {
            let event_window_controls: EventWindowControls =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();

            let docs_manager = &mut *(match docs_generator_move_copy.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            });

            match event_window_controls {
                EventWindowControls::TrackingAreaClicked(msg) => {
                    if let Some(tracking_area) = docs_manager.tracking_area.as_ref() {
                        if msg.id == tracking_area.id {
                            // docs_manager.on_tracking_area_clicked();
                        }
                    }
                }
                _ => {}
            }
        });
    }
}
