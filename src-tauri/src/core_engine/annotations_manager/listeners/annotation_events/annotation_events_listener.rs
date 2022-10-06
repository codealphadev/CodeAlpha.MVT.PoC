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
                AnnotationManagerEvent::Add(annotation_job_group) => {
                    annotations_manager
                        .lock()
                        .add_annotation_job_group(annotation_job_group);
                }
                AnnotationManagerEvent::Update(annotation_job_group) => {
                    annotations_manager
                        .lock()
                        .update_annotation_job_group(annotation_job_group);
                }
                AnnotationManagerEvent::Remove(id) => {
                    annotations_manager.lock().remove_annotation_job_group(id);
                }
            }
        }
    });
}
