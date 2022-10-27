use crate::{
    core_engine::{
        core_engine::CoreEngineProcedure,
        events::models::{
            DismissSuggestionMessage, NodeAnnotationClickedMessage, PerformSuggestionMessage,
            UpdateSelectedSuggestionMessage,
        },
        CodeDocument, EditorWindowUid, SwiftFormatError,
    },
    platform::macos::models::editor::{EditorShortcutPressedMessage, ModifierKey},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use strum::{Display, EnumIter};
use tauri::api::process::CommandChild;
use ts_rs::TS;
use uuid::Uuid;

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

#[derive(Clone, Debug, Hash)]
pub enum FeatureProcedure {
    LongRunning,
    ShortRunning,
}

impl FeatureKind {
    pub fn requires_ai(&self) -> bool {
        match self {
            FeatureKind::BracketHighlight => false,
            FeatureKind::ComplexityRefactoring => false,
            FeatureKind::DocsGeneration => true,
            FeatureKind::Formatter => false,
        }
    }
    pub fn should_compute(&self, trigger: &CoreEngineTrigger) -> Option<FeatureProcedure> {
        match self {
            FeatureKind::ComplexityRefactoring => match trigger {
                CoreEngineTrigger::OnTextContentChange => Some(FeatureProcedure::LongRunning),
                CoreEngineTrigger::OnUserCommand(UserCommand::PerformSuggestion(_)) => {
                    Some(FeatureProcedure::ShortRunning)
                }
                CoreEngineTrigger::OnUserCommand(UserCommand::DismissSuggestion(_)) => {
                    Some(FeatureProcedure::ShortRunning)
                }
                CoreEngineTrigger::OnUserCommand(UserCommand::SelectSuggestion(_)) => {
                    Some(FeatureProcedure::ShortRunning)
                }
                _ => None,
            },
            FeatureKind::DocsGeneration => match trigger {
                CoreEngineTrigger::OnTextContentChange => Some(FeatureProcedure::ShortRunning),
                CoreEngineTrigger::OnTextSelectionChange => Some(FeatureProcedure::ShortRunning),
                CoreEngineTrigger::OnUserCommand(cmd) => match cmd {
                    UserCommand::NodeAnnotationClicked(_) => Some(FeatureProcedure::ShortRunning),
                    _ => None,
                },
                _ => None,
            },
            FeatureKind::BracketHighlight => match trigger {
                CoreEngineTrigger::OnTextSelectionChange => Some(FeatureProcedure::ShortRunning),
                CoreEngineTrigger::OnTextContentChange => None, // The TextSelectionChange is already triggered on text content change
                _ => None,
            },
            FeatureKind::Formatter => match trigger {
                CoreEngineTrigger::OnShortcutPressed(msg) => {
                    if msg.modifier == ModifierKey::Cmd && msg.key == "S" {
                        return Some(FeatureProcedure::ShortRunning);
                    } else {
                        return None;
                    }
                }
                _ => None,
            },
        }
    }
}

pub fn hash_trigger_and_feature(core_engine_procedure: &CoreEngineProcedure) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    core_engine_procedure.trigger.hash(&mut hasher);
    core_engine_procedure.feature.hash(&mut hasher);
    core_engine_procedure.procedure.hash(&mut hasher);
    core_engine_procedure.window_uid.hash(&mut hasher);
    hasher.finish()
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
    fn compute_short_running(
        &mut self,
        code_document: CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError>;
    fn compute_long_running(
        &mut self,
        code_document: CodeDocument,
        trigger: &CoreEngineTrigger,
        execution_id: Option<Uuid>,
    ) -> Result<(), FeatureError>;
    fn activate(&mut self) -> Result<(), FeatureError>;
    fn deactivate(&mut self) -> Result<(), FeatureError>;
    fn reset(&mut self) -> Result<(), FeatureError>;
}

impl FeatureBase for Feature {
    fn compute_long_running(
        &mut self,
        code_document: CodeDocument,
        trigger: &CoreEngineTrigger,
        execution_id: Option<Uuid>,
    ) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => {
                feature.compute_long_running(code_document, trigger, execution_id)
            }
            Feature::DocsGeneration(feature) => {
                feature.compute_long_running(code_document, trigger, execution_id)
            }
            Feature::Formatter(feature) => {
                feature.compute_long_running(code_document, trigger, execution_id)
            }
            Feature::ComplexityRefactoring(feature) => {
                feature.compute_long_running(code_document, trigger, execution_id)
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

    fn kind(&self) -> FeatureKind {
        match self {
            Feature::BracketHighlighting(feature) => feature.kind(),
            Feature::DocsGeneration(feature) => feature.kind(),
            Feature::Formatter(feature) => feature.kind(),
            Feature::ComplexityRefactoring(feature) => feature.kind(),
        }
    }

    fn compute_short_running(
        &mut self,
        code_document: CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => {
                feature.compute_short_running(code_document, trigger)
            }
            Feature::DocsGeneration(feature) => {
                feature.compute_short_running(code_document, trigger)
            }
            Feature::Formatter(feature) => feature.compute_short_running(code_document, trigger),
            Feature::ComplexityRefactoring(feature) => {
                feature.compute_short_running(code_document, trigger)
            }
        }
    }
}
