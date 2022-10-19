use crate::{
    core_engine::{
        events::models::{
            DismissSuggestionMessage, NodeAnnotationClickedMessage, PerformSuggestionMessage,
            UpdateSelectedSuggestionMessage,
        },
        CodeDocument,
    },
    platform::macos::models::editor::EditorShortcutPressedMessage,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use ts_rs::TS;
use uuid::Uuid;

use super::{
    complexity_refactoring::ComplexityRefactoringError,
    docs_generation::DocsGenerationError,
    formatter::{SwiftFormatError, SwiftFormatter},
    BracketHighlight, BracketHighlightError, ComplexityRefactoring, DocsGenerator,
};

#[derive(Debug, Clone, PartialEq)]
pub enum UserCommand {
    PerformSuggestion(PerformSuggestionMessage),
    DismissSuggestion(DismissSuggestionMessage),
    SelectSuggestion(UpdateSelectedSuggestionMessage),
    NodeAnnotationClicked(NodeAnnotationClickedMessage),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoreEngineTrigger {
    OnShortcutPressed(EditorShortcutPressedMessage),
    OnTextContentChange,
    OnTextSelectionChange,
    OnViewportMove,
    OnViewportDimensionsChange,
    OnVisibleTextRangeChange,
    OnUserCommand(UserCommand),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/code_annotations/")]
pub enum FeatureKind {
    BracketHighlight,
    ComplexityRefactoring,
    DocsGeneration,
    Formatter,
}

pub enum Feature {
    BracketHighlighting(BracketHighlight),
    DocsGeneration(DocsGenerator),
    Formatter(SwiftFormatter),
    ComplexityRefactoring(ComplexityRefactoring),
}
impl fmt::Debug for Feature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Feature::BracketHighlighting(_) => "BracketHighlighting",
            Feature::DocsGeneration(_) => "DocsGeneration",
            Feature::Formatter(_) => "Formatter",
            Feature::ComplexityRefactoring(_) => "ComplexityRefactoring",
        };
        write!(f, "{}", name)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FeatureError {
    #[error("Something went wrong when executing this feature.")]
    GenericError(#[source] anyhow::Error),
}

impl From<BracketHighlightError> for FeatureError {
    fn from(cause: BracketHighlightError) -> Self {
        FeatureError::GenericError(cause.into())
    }
}

impl From<ComplexityRefactoringError> for FeatureError {
    fn from(cause: ComplexityRefactoringError) -> Self {
        FeatureError::GenericError(cause.into())
    }
}

impl From<DocsGenerationError> for FeatureError {
    fn from(cause: DocsGenerationError) -> Self {
        FeatureError::GenericError(cause.into())
    }
}

impl From<SwiftFormatError> for FeatureError {
    fn from(cause: SwiftFormatError) -> Self {
        FeatureError::GenericError(cause.into())
    }
}

pub trait FeatureBase {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
        execution_id: Uuid,
    ) -> Result<(), FeatureError>;
    fn activate(&mut self) -> Result<(), FeatureError>;
    fn deactivate(&mut self) -> Result<(), FeatureError>;
    fn reset(&mut self) -> Result<(), FeatureError>;
    fn requires_ai(&self) -> bool;
}

impl FeatureBase for Feature {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
        execution_id: Uuid,
    ) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => {
                feature.compute(code_document, trigger, execution_id)
            }
            Feature::DocsGeneration(feature) => {
                feature.compute(code_document, trigger, execution_id)
            }
            Feature::Formatter(feature) => feature.compute(code_document, trigger, execution_id),
            Feature::ComplexityRefactoring(feature) => {
                feature.compute(code_document, trigger, execution_id)
            }
        }
    }

    fn activate(&mut self) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => feature.activate(),
            Feature::DocsGeneration(feature) => feature.activate(),
            Feature::Formatter(feature) => feature.activate(),
            Feature::ComplexityRefactoring(feature) => feature.activate(),
        }
    }

    fn deactivate(&mut self) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => feature.deactivate(),
            Feature::DocsGeneration(feature) => feature.deactivate(),
            Feature::Formatter(feature) => feature.deactivate(),
            Feature::ComplexityRefactoring(feature) => feature.deactivate(),
        }
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => feature.reset(),
            Feature::DocsGeneration(feature) => feature.reset(),
            Feature::Formatter(feature) => feature.reset(),
            Feature::ComplexityRefactoring(feature) => feature.reset(),
        }
    }

    fn requires_ai(&self) -> bool {
        match self {
            Feature::BracketHighlighting(feature) => feature.requires_ai(),
            Feature::DocsGeneration(feature) => feature.requires_ai(),
            Feature::Formatter(feature) => feature.requires_ai(),
            Feature::ComplexityRefactoring(feature) => feature.requires_ai(),
        }
    }
}
