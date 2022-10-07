use std::sync::Arc;

use crate::{
    app_handle,
    core_engine::{
        annotations_manager::{AnnotationsManager, AnnotationsManagerTrait},
        events::AnnotationManagerEvent,
    },
    utils::messaging::ChannelList,
};
use parking_lot::Mutex;
use tauri::Manager;

pub fn annotation_events_listener(annotations_manager_arc: &Arc<Mutex<AnnotationsManager>>) {
    app_handle().listen_global(ChannelList::NodeAnnotationEvent.to_string(), {
        let annotations_manager = annotations_manager_arc.clone();
        move |msg| {
            let annotation_event: AnnotationManagerEvent =
                serde_json::from_str(&msg.payload().unwrap()).unwrap();
            match annotation_event {
                AnnotationManagerEvent::Add((group_id, feature, jobs, window_uid)) => {
                    annotations_manager
                        .lock()
                        .add_annotation_jobs(group_id, feature, jobs, window_uid);
                }
                AnnotationManagerEvent::Update((group_id, jobs)) => {
                    annotations_manager
                        .lock()
                        .update_annotation_job_group(group_id, jobs);
                }
                AnnotationManagerEvent::Remove(id) => {
                    annotations_manager.lock().remove_annotation_job_group(id);
                }
            }
        }
    });
}
