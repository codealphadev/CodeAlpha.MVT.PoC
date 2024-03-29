use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;
use tracing::error;

use crate::{
    app_handle,
    core_engine::{
        annotations_manager::{AnnotationsManager, AnnotationsManagerTrait},
        events::AnnotationManagerEvent,
    },
    utils::messaging::ChannelList,
};

pub fn annotation_events_listener(annotations_manager_arc: &Arc<Mutex<AnnotationsManager>>) {
    app_handle().listen_global(ChannelList::AnnotationEvent.to_string(), {
        let annotations_manager = annotations_manager_arc.clone();
        move |msg| {
            let annotation_event: AnnotationManagerEvent =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();
            match annotation_event {
                AnnotationManagerEvent::Add((group_id, feature, jobs, window_uid)) => {
                    annotations_manager
                        .lock()
                        .add_annotation_jobs_group(group_id, feature, jobs, window_uid);
                }
                AnnotationManagerEvent::Upsert((group_id, feature, jobs, window_uid)) => {
                    annotations_manager
                        .lock()
                        .upsert_annotation_job_group(group_id, feature, jobs, window_uid);
                }
                AnnotationManagerEvent::Remove(id) => {
                    annotations_manager.lock().remove_annotation_job_group(id);
                }
                AnnotationManagerEvent::ScrollToAnnotationInGroup((group_id, get_via)) => {
                    let cancel_recv = annotations_manager.lock().reset_scroll_cancel_channel();
                    if let Err(e) = annotations_manager.lock().scroll_to_annotation(
                        group_id,
                        get_via,
                        cancel_recv,
                    ) {
                        error!(?e, "Error scrolling to annotation");
                    }
                }
            }
        }
    });
}
