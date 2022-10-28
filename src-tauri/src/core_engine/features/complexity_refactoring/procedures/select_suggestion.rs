use crate::core_engine::{
    annotations_manager::{AnnotationKind, GetAnnotationInGroupVia},
    events::AnnotationManagerEvent,
    features::{complexity_refactoring::ComplexityRefactoringError, SuggestionId},
};

pub fn select_suggestion(suggestion_id: SuggestionId) -> Result<(), ComplexityRefactoringError> {
    AnnotationManagerEvent::ScrollToAnnotationInGroup((
        suggestion_id,
        GetAnnotationInGroupVia::Kind(AnnotationKind::ExtractionStartChar),
    ))
    .publish_to_tauri();

    Ok(())
}
