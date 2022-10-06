use serde::{Deserialize, Serialize};

use crate::{
    core_engine::{features::FeatureKind, TextRange},
    platform::macos::{get_bounds_for_TextRange, GetVia},
    utils::geometry::LogicalPosition,
};

use super::{
    annotations_manager::{Annotation, AnnotationError, AnnotationResult, AnnotationShape},
    AnnotationJobInstructions, AnnotationJobTrait, AnnotationKind,
    InstructionBoundsPropertyOfInterest, ViewportPositioning,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnnotationJobSingleChar {
    id: uuid::Uuid,
    char_index: usize,
    kind: AnnotationKind,
    feature: FeatureKind,
    instructions: AnnotationJobInstructions,
    result: Option<AnnotationResult>,
}

impl AnnotationJobTrait for AnnotationJobSingleChar {
    fn new(
        range: &TextRange,
        kind: AnnotationKind,
        feature: FeatureKind,
        instructions: AnnotationJobInstructions,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            char_index: range.index,
            kind,
            feature,
            instructions,
            result: None,
        }
    }

    fn compute_bounds(
        &mut self,
        visible_text_range: &TextRange,
        code_doc_origin: &LogicalPosition,
    ) -> Result<AnnotationResult, AnnotationError> {
        let viewport_positioning =
            Self::compare_with_visible_text_range(self.char_index, visible_text_range);

        // If the character is not within the VisibleTextRange, we can't compute the bounds
        if viewport_positioning != ViewportPositioning::Visible {
            return Ok(AnnotationResult {
                id: self.id,
                position_relative_to_viewport: viewport_positioning,
                bounds: None,
            });
        }

        let ax_bounds_global = get_bounds_for_TextRange(
            &TextRange {
                index: self.char_index,
                length: 1,
            },
            &GetVia::Current,
        )
        .map_err(|e| AnnotationError::GenericError(e.into()))?;

        let result = AnnotationResult {
            id: self.id,
            position_relative_to_viewport: viewport_positioning,
            bounds: Some(vec![ax_bounds_global.to_local(code_doc_origin)]),
        };

        self.result = Some(result.clone());
        Ok(result)
    }

    fn attempt_compute_bounds(
        &mut self,
        visible_text_range: &TextRange,
        code_doc_origin: &LogicalPosition,
    ) -> Result<AnnotationResult, AnnotationError> {
        let viewport_positioning =
            Self::compare_with_visible_text_range(self.char_index, visible_text_range);

        if let Some(previous_result) = self.result.as_ref() {
            if let Some(bounds) = previous_result.bounds.as_ref() {
                // Case: there is a previous result and it contains bounds -> we don't need to compute anything
                // and only update the viewport positioning
                let result = AnnotationResult {
                    id: self.id,
                    position_relative_to_viewport: viewport_positioning,
                    bounds: Some(bounds.clone()),
                };

                self.result = Some(result.clone());
                return Ok(result);
            }
        }

        self.compute_bounds(visible_text_range, code_doc_origin)
    }

    fn get_annotation(&self) -> Option<Annotation> {
        if let Some(result) = self.result.as_ref() {
            Some(Self::get_annotation_(self, &result))
        } else {
            return None;
        }
    }
}

impl AnnotationJobSingleChar {
    fn compare_with_visible_text_range(
        char_index: usize,
        visible_text_range: &TextRange,
    ) -> ViewportPositioning {
        if char_index < visible_text_range.index {
            ViewportPositioning::InvisibleAbove
        } else if char_index > visible_text_range.index + visible_text_range.length {
            ViewportPositioning::InvisibleBelow
        } else {
            ViewportPositioning::Visible
        }
    }

    fn get_annotation_(annotation_job: &Self, result: &AnnotationResult) -> Annotation {
        let mut annotation = Annotation {
            id: result.id,
            kind: annotation_job.kind.clone(),
            feature: annotation_job.feature.clone(),
            position_relative_to_viewport: result.position_relative_to_viewport.clone(),
            shapes: vec![],
        };

        match annotation_job.instructions.bounds_property_of_interest {
            InstructionBoundsPropertyOfInterest::Frame => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Rectangle(r.clone()))
                        .collect()
                });
            }
            InstructionBoundsPropertyOfInterest::PosTopLeft => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Point(r.top_left()))
                        .collect()
                });
            }
            InstructionBoundsPropertyOfInterest::PosTopRight => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Point(r.top_right()))
                        .collect()
                });
            }
            InstructionBoundsPropertyOfInterest::PosBotLeft => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Point(r.bottom_left()))
                        .collect()
                });
            }
            InstructionBoundsPropertyOfInterest::PosBotRight => {
                annotation.shapes = result.bounds.as_ref().map_or(vec![], |r| {
                    r.into_iter()
                        .map(|r| AnnotationShape::Point(r.bottom_right()))
                        .collect()
                });
            }
        }

        annotation
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core_engine::{
            annotations_manager::{
                annotation_job::InstructionBoundsPropertyOfInterest,
                annotations_manager::{Annotation, AnnotationResult, AnnotationShape},
            },
            features::FeatureKind,
            TextRange,
        },
        utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
    };

    use super::{
        AnnotationJobInstructions, AnnotationJobSingleChar, AnnotationJobTrait, AnnotationKind,
        ViewportPositioning,
    };

    #[test]
    fn get_annotation_instruct_property_of_interest() {
        let instructions_prop_as_frame = AnnotationJobInstructions::default();
        let instructions_prop_as_top_left_point = AnnotationJobInstructions {
            bounds_property_of_interest: InstructionBoundsPropertyOfInterest::PosTopLeft,
            ..instructions_prop_as_frame
        };

        let annotation_job1 = AnnotationJobSingleChar::new(
            &TextRange {
                index: 0,
                length: 1,
            },
            AnnotationKind::CodeblockFirstChar,
            FeatureKind::ComplexityRefactoring,
            AnnotationJobInstructions::default(),
        );

        let annotation_job2 = AnnotationJobSingleChar::new(
            &TextRange {
                index: 0,
                length: 1,
            },
            AnnotationKind::CodeblockFirstChar,
            FeatureKind::ComplexityRefactoring,
            instructions_prop_as_top_left_point,
        );

        let result = AnnotationResult {
            id: annotation_job1.clone().id,
            position_relative_to_viewport: ViewportPositioning::Visible,
            bounds: Some(vec![LogicalFrame {
                origin: LogicalPosition { x: 0.0, y: 0.0 },
                size: LogicalSize {
                    width: 10.0,
                    height: 10.0,
                },
            }]),
        };

        let expected_annotation = Annotation {
            id: annotation_job1.clone().id,
            kind: annotation_job1.clone().kind,
            feature: annotation_job1.clone().feature,
            position_relative_to_viewport: ViewportPositioning::Visible,
            shapes: vec![AnnotationShape::Rectangle(LogicalFrame {
                origin: LogicalPosition { x: 0.0, y: 0.0 },
                size: LogicalSize {
                    width: 10.0,
                    height: 10.0,
                },
            })],
        };

        assert_eq!(
            expected_annotation,
            AnnotationJobSingleChar::get_annotation_(&annotation_job1, &result)
        );

        let expected_annotation = Annotation {
            id: annotation_job1.clone().id,
            kind: annotation_job1.clone().kind,
            feature: annotation_job1.clone().feature,
            position_relative_to_viewport: ViewportPositioning::Visible,
            shapes: vec![AnnotationShape::Point(LogicalPosition { x: 0.0, y: 0.0 })],
        };

        assert_eq!(
            expected_annotation,
            AnnotationJobSingleChar::get_annotation_(&annotation_job2, &result)
        );
    }
}
