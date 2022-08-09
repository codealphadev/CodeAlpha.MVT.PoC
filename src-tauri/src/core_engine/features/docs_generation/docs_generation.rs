use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use tauri::{regex::Match, Manager};
use tree_sitter::{Node, Point};

use crate::{
    app_handle,
    ax_interaction::{derive_xcode_textarea_dimensions, get_textarea_uielement},
    core_engine::{
        ax_utils::get_bounds_of_TextRange,
        rules::{TextPosition, TextRange},
        syntax_tree::{SwiftCodeBlockType, SwiftSyntaxTree},
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
        let codeblock_node = self.get_selected_codeblock_node(&selected_text_range.unwrap());

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

    fn derive_annotation_tracking_area(&self, codeblock_node: &Node) -> Option<TrackingArea> {
        // 3. calculate position for the annotation icon
        let single_char_bounds = self.get_height_of_single_char(codeblock_node);

        // 4. calculate the bounds of the codeblock
        let codeblock_bounds = self.get_bounds_for_codeblock_node(codeblock_node);

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

    fn get_height_of_single_char(&self, codeblock_node: &Node) -> Option<MatchRectangle> {
        let (content, textarea_ui_element) = if let (Some(text), Some(elem)) = (
            self.text_content.clone(),
            get_textarea_uielement(self.window_pid),
        ) {
            (text, elem)
        } else {
            return None;
        };

        if let Some(first_char_text_pos) =
            TextPosition::from_TSPoint(&codeblock_node.start_position()).as_TextIndex(&content)
        {
            return get_bounds_of_TextRange(
                &TextRange {
                    index: first_char_text_pos,
                    length: 0,
                },
                &textarea_ui_element,
            );
        }

        None
    }

    fn get_bounds_for_codeblock_node(&self, codeblock_node: &Node) -> Option<MatchRectangle> {
        // 1. Get textarea dimensions
        let textarea_ui_element = if let Some(elem) = get_textarea_uielement(self.window_pid) {
            elem
        } else {
            return None;
        };

        let (textarea_origin, textarea_size) =
            if let Ok((origin, size)) = derive_xcode_textarea_dimensions(&textarea_ui_element) {
                (origin, size)
            } else {
                return None;
            };

        // 2. Get codeblock dimensions
        let content = if let Some(text) = self.text_content.clone() {
            text
        } else {
            return None;
        };

        let codeblock_first_char_pos =
            TextPosition::from_TSPoint(&codeblock_node.start_position()).as_TextIndex(&content);
        let codeblock_last_char_pos =
            TextPosition::from_TSPoint(&codeblock_node.end_position()).as_TextIndex(&content);

        if let (Some(first_char_pos), Some(last_char_pos)) =
            (codeblock_first_char_pos, codeblock_last_char_pos)
        {
            let first_char_bounds = get_bounds_of_TextRange(
                &TextRange {
                    index: first_char_pos,
                    length: 0,
                },
                &textarea_ui_element,
            );
            let last_char_bounds = get_bounds_of_TextRange(
                &TextRange {
                    index: last_char_pos,
                    length: 0,
                },
                &textarea_ui_element,
            );

            if let (Some(first_char_bounds), Some(last_char_bounds)) =
                (first_char_bounds, last_char_bounds)
            {
                let codeblock_bounds = MatchRectangle {
                    origin: LogicalPosition {
                        x: textarea_origin.x,
                        y: first_char_bounds.origin.y,
                    },
                    size: LogicalSize {
                        width: textarea_size.width,
                        height: last_char_bounds.origin.y - first_char_bounds.origin.y,
                    },
                };
                return Some(codeblock_bounds);
            }
        }

        None
    }

    fn get_selected_codeblock_node(&self, selected_text_range: &TextRange) -> Option<Node> {
        // 1. Determine the node that the curser currently is on
        let mut currently_selected_node = None;
        if let (Some(syntax_tree), Some(text_content)) =
            (self.swift_syntax_tree.tree(), &self.text_content)
        {
            if let Some((selected_text_range_start_pos, _)) =
                selected_text_range.as_StartEndTextPosition(text_content)
            {
                currently_selected_node = syntax_tree.root_node().named_descendant_for_point_range(
                    Point {
                        row: selected_text_range_start_pos.row,
                        column: selected_text_range_start_pos.column,
                    },
                    Point {
                        row: selected_text_range_start_pos.row,
                        column: selected_text_range_start_pos.column,
                    },
                );
            }
        }

        // 2. Find the nearest codeblock node
        if let Some(mut node) = currently_selected_node.clone() {
            loop {
                if SwiftCodeBlockType::from_str(&node.kind()).is_ok() {
                    currently_selected_node = Some(node);
                    break;
                }

                if let Some(parent) = node.parent() {
                    node = parent;
                } else {
                    break;
                }
            }
        }

        currently_selected_node
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
