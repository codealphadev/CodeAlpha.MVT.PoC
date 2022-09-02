use crate::{
    core_engine::CodeDocument, platform::macos::models::editor::EditorShortcutPressedMessage,
    window_controls::models::TrackingAreaClickedMessage,
};

use super::{
    docs_generation::DocsGenerationError,
    formatter::{SwiftFormatError, SwiftFormatter},
    BracketHighlight, BracketHighlightError, DocsGenerator,
};

#[derive(Debug, Clone)]
pub enum CoreEngineTrigger {
    OnShortcutPressed(EditorShortcutPressedMessage),
    OnTextContentChange,
    OnTextSelectionChange,
    OnViewportMove,
    OnViewportDimensionsChange,
    OnVisibleTextRangeChange,
    OnScrollingFinished,
    OnTrackingAreaClicked(TrackingAreaClickedMessage),
}

pub enum Feature {
    BracketHighlighting(BracketHighlight),
    DocsGeneration(DocsGenerator),
    Formatter(SwiftFormatter),
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
        }
    }

    fn activate(&mut self) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => feature.activate(),
            Feature::DocsGeneration(feature) => feature.activate(),
            Feature::Formatter(feature) => feature.activate(),
        }
    }

    fn deactivate(&mut self) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => feature.deactivate(),
            Feature::DocsGeneration(feature) => feature.deactivate(),
            Feature::Formatter(feature) => feature.deactivate(),
        }
    }

    fn reset(&mut self) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => feature.reset(),
            Feature::DocsGeneration(feature) => feature.reset(),
            Feature::Formatter(feature) => feature.reset(),
        }
    }
}
