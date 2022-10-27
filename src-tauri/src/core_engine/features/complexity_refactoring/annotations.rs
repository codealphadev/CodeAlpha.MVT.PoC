use uuid::Uuid;

use crate::core_engine::{
    annotations_manager::{
        AnnotationJob, AnnotationJobInstructions, AnnotationJobSingleChar, AnnotationJobTrait,
        AnnotationKind, InstructionBounds, InstructionBoundsPropertyOfInterest,
        InstructionWrappedLines,
    },
    events::AnnotationManagerEvent,
    features::FeatureKind,
    TextRange,
};

pub fn remove_annotations_for_suggestions(suggestion_ids: Vec<uuid::Uuid>) {
    for suggestion_id in suggestion_ids {
        AnnotationManagerEvent::Remove(suggestion_id).publish_to_tauri();
    }
}

pub fn create_annotation_group_for_extraction_and_context(
    suggestion_id: Uuid,
    context_range: TextRange,
    extraction_range: TextRange,
    window_uid: usize,
) {
    let extract_start_char_job_id = uuid::Uuid::new_v4();
    let extract_start_char_job = AnnotationJobSingleChar::new(
        extract_start_char_job_id,
        &TextRange {
            index: extraction_range.index,
            length: 1,
        },
        AnnotationKind::ExtractionStartChar,
        AnnotationJobInstructions {
            bounds: InstructionBounds::SingleRect,
            bounds_property_of_interest: InstructionBoundsPropertyOfInterest::PosTopLeft,
            wrapped_lines: InstructionWrappedLines::None,
        },
    );

    let extract_end_char_job_id = uuid::Uuid::new_v4();
    let extract_end_char_job = AnnotationJobSingleChar::new(
        extract_end_char_job_id,
        &TextRange {
            index: extraction_range.index + extraction_range.length,
            length: 1,
        },
        AnnotationKind::ExtractionEndChar,
        AnnotationJobInstructions {
            bounds: InstructionBounds::SingleRect,
            bounds_property_of_interest: InstructionBoundsPropertyOfInterest::PosBotRight,
            wrapped_lines: InstructionWrappedLines::None,
        },
    );

    let context_range_start_char_job_id = uuid::Uuid::new_v4();
    let context_range_start_char_job = AnnotationJobSingleChar::new(
        context_range_start_char_job_id,
        &TextRange {
            index: context_range.index,
            length: 1,
        },
        AnnotationKind::CodeblockFirstChar,
        AnnotationJobInstructions {
            bounds: InstructionBounds::SingleRect,
            bounds_property_of_interest: InstructionBoundsPropertyOfInterest::PosTopLeft,
            wrapped_lines: InstructionWrappedLines::None,
        },
    );

    let context_range_end_char_job_id = uuid::Uuid::new_v4();
    let context_range_end_char_job = AnnotationJobSingleChar::new(
        context_range_end_char_job_id,
        &TextRange {
            index: context_range.index + context_range.length,
            length: 1,
        },
        AnnotationKind::CodeblockLastChar,
        AnnotationJobInstructions {
            bounds: InstructionBounds::SingleRect,
            bounds_property_of_interest: InstructionBoundsPropertyOfInterest::PosBotRight,
            wrapped_lines: InstructionWrappedLines::None,
        },
    );

    AnnotationManagerEvent::Upsert((
        suggestion_id,
        FeatureKind::ComplexityRefactoring,
        vec![
            AnnotationJob::SingleChar(context_range_start_char_job),
            AnnotationJob::SingleChar(context_range_end_char_job),
            AnnotationJob::SingleChar(extract_start_char_job),
            AnnotationJob::SingleChar(extract_end_char_job),
        ],
        window_uid,
    ))
    .publish_to_tauri();
}
