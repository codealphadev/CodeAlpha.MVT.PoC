use std::fmt;

use crate::{
    core_engine::{events::models::PerformRefactoringOperationMessage, CodeDocument},
    platform::macos::models::editor::EditorShortcutPressedMessage,
    window_controls::models::TrackingAreaClickedMessage,
};

use super::{
    complexity_refactoring::ComplexityRefactoringError,
    docs_generation::DocsGenerationError,
    formatter::{SwiftFormatError, SwiftFormatter},
    BracketHighlight, BracketHighlightError, ComplexityRefactoring, DocsGenerator,
};

#[derive(Debug, Clone)]
pub enum CoreEngineTrigger {
    OnShortcutPressed(EditorShortcutPressedMessage),
    OnTextContentChange,
    OnTextSelectionChange,
    OnViewportMove,
    OnViewportDimensionsChange,
    OnVisibleTextRangeChange,
    OnTrackingAreaClicked(TrackingAreaClickedMessage),
    OnUserCommand(PerformRefactoringOperationMessage),
}

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
    ) -> Result<(), FeatureError>;
    fn update_visualization(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError>;
    fn activate(&mut self) -> Result<(), FeatureError>;
    fn deactivate(&mut self) -> Result<(), FeatureError>;
    fn reset(&mut self) -> Result<(), FeatureError>;
}

impl FeatureBase for Feature {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => feature.compute(code_document, trigger),
            Feature::DocsGeneration(feature) => feature.compute(code_document, trigger),
            Feature::Formatter(feature) => feature.compute(code_document, trigger),
            Feature::ComplexityRefactoring(feature) => feature.compute(code_document, trigger),
        }
    }

    fn update_visualization(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => {
                feature.update_visualization(code_document, trigger)
            }
            Feature::DocsGeneration(feature) => {
                feature.update_visualization(code_document, trigger)
            }
            Feature::Formatter(feature) => feature.update_visualization(code_document, trigger),
            Feature::ComplexityRefactoring(feature) => {
                feature.update_visualization(code_document, trigger)
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
}
