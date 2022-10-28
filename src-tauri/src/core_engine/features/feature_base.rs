use crate::{
    core_engine::{
        events::models::{
            DismissSuggestionMessage, NodeAnnotationClickedMessage, PerformSuggestionMessage,
            UpdateSelectedSuggestionMessage,
        },
        CodeDocument, SwiftFormatError,
    },
    platform::macos::models::editor::EditorShortcutPressedMessage,
};
use serde::{Deserialize, Serialize};
use std::{fmt, hash::Hash};
use strum::{Display, EnumIter};
use tauri::api::process::CommandChild;
use ts_rs::TS;

use super::{
    complexity_refactoring::ComplexityRefactoringError, docs_generation::DocsGenerationError,
    formatter::SwiftFormatter, BracketHighlight, BracketHighlightError, ComplexityRefactoring,
    DocsGenerator,
};

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum UserCommand {
    PerformSuggestion(PerformSuggestionMessage),
    DismissSuggestion(DismissSuggestionMessage),
    SelectSuggestion(UpdateSelectedSuggestionMessage),
    NodeAnnotationClicked(NodeAnnotationClickedMessage),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum CoreEngineTrigger {
    OnShortcutPressed(EditorShortcutPressedMessage),
    OnTextContentChange,
    OnTextSelectionChange,
    OnViewportMove,
    OnViewportDimensionsChange,
    OnVisibleTextRangeChange,
    OnUserCommand(UserCommand),
}

#[derive(EnumIter, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, TS, Hash, Display)]
#[ts(export, export_to = "bindings/features/code_annotations/")]
pub enum FeatureKind {
    BracketHighlight,
    ComplexityRefactoring,
    DocsGeneration,
    Formatter,
}

impl FeatureKind {
    pub fn requires_ai(&self, trigger: &CoreEngineTrigger) -> bool {
        match self {
            FeatureKind::BracketHighlight => BracketHighlight::requires_ai(self, trigger),
            FeatureKind::DocsGeneration => DocsGenerator::requires_ai(self, trigger),
            FeatureKind::Formatter => SwiftFormatter::requires_ai(self, trigger),
            FeatureKind::ComplexityRefactoring => ComplexityRefactoring::requires_ai(self, trigger),
        }
    }
    pub fn should_compute(&self, trigger: &CoreEngineTrigger) -> bool {
        match self {
            FeatureKind::BracketHighlight => BracketHighlight::should_compute(self, trigger),
            FeatureKind::DocsGeneration => DocsGenerator::should_compute(self, trigger),
            FeatureKind::Formatter => SwiftFormatter::should_compute(self, trigger),
            FeatureKind::ComplexityRefactoring => {
                ComplexityRefactoring::should_compute(self, trigger)
            }
        }
    }
}

#[derive(Debug)]
pub enum FeatureSignals {
    ComputationCompleted,
    SwiftLspCommandSpawned(CommandChild),
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
        match cause {
            _ => FeatureError::GenericError(cause.into()),
        }
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
    fn kind(&self) -> FeatureKind;
    fn compute(
        &mut self,
        code_document: CodeDocument,
        trigger: CoreEngineTrigger,
    ) -> Result<(), FeatureError>;
    fn activate(&mut self) -> Result<(), FeatureError>;
    fn deactivate(&mut self) -> Result<(), FeatureError>;
    fn reset(&mut self) -> Result<(), FeatureError>;
    fn should_compute(kind: &FeatureKind, trigger: &CoreEngineTrigger) -> bool;
    fn requires_ai(kind: &FeatureKind, trigger: &CoreEngineTrigger) -> bool;
}

impl FeatureBase for Feature {
    fn compute(
        &mut self,
        code_document: CodeDocument,
        trigger: CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => feature.compute(code_document, trigger),
            Feature::DocsGeneration(feature) => feature.compute(code_document, trigger),
            Feature::Formatter(feature) => feature.compute(code_document, trigger),
            Feature::ComplexityRefactoring(feature) => feature.compute(code_document, trigger),
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

    fn kind(&self) -> FeatureKind {
        match self {
            Feature::BracketHighlighting(feature) => feature.kind(),
            Feature::DocsGeneration(feature) => feature.kind(),
            Feature::Formatter(feature) => feature.kind(),
            Feature::ComplexityRefactoring(feature) => feature.kind(),
        }
    }

    fn should_compute(kind: &FeatureKind, trigger: &CoreEngineTrigger) -> bool {
        match kind {
            FeatureKind::BracketHighlight => BracketHighlight::should_compute(kind, trigger),
            FeatureKind::DocsGeneration => DocsGenerator::should_compute(kind, trigger),
            FeatureKind::Formatter => SwiftFormatter::should_compute(kind, trigger),
            FeatureKind::ComplexityRefactoring => {
                ComplexityRefactoring::should_compute(kind, trigger)
            }
        }
    }

    fn requires_ai(kind: &FeatureKind, trigger: &CoreEngineTrigger) -> bool {
        match kind {
            FeatureKind::BracketHighlight => BracketHighlight::requires_ai(kind, trigger),
            FeatureKind::DocsGeneration => DocsGenerator::requires_ai(kind, trigger),
            FeatureKind::Formatter => SwiftFormatter::requires_ai(kind, trigger),
            FeatureKind::ComplexityRefactoring => ComplexityRefactoring::requires_ai(kind, trigger),
        }
    }
}
