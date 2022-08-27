use crate::{
    ax_interaction::models::editor::EditorShortcutPressedMessage, core_engine::CodeDocument,
};

use super::{formatter::SwiftFormatter, BracketHighlight, DocsGenerator};

pub enum CoreEngineTrigger {
    OnShortcutPressed(EditorShortcutPressedMessage),
    OnTextContentChange,
    OnTextSelectionChange,
    OnViewportMove,
    OnViewportDimensionsChange,
    OnVisibleTextRangeChange,
}

pub enum Feature {
    BracketHighlighting(BracketHighlight),
    DocsGeneration(DocsGenerator),
    Formatter(SwiftFormatter),
}

#[derive(thiserror::Error, Debug)]
pub enum FeatureError {
    #[error("Feature could not compute.")]
    ComputeUnsuccessful(),
    #[error("Feature could not update visualization.")]
    UpdateVisualizationUnsuccessful,
    #[error("Something went wrong when executing this feature.")]
    GenericError(#[source] anyhow::Error),
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
}

impl FeatureBase for Feature {
    fn compute(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => Ok(()),
            Feature::DocsGeneration(feature) => Ok(()),
            Feature::Formatter(feature) => Ok(()),
        }
    }

    fn update_visualization(
        &mut self,
        code_document: &CodeDocument,
        trigger: &CoreEngineTrigger,
    ) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => Ok(()),
            Feature::DocsGeneration(feature) => Ok(()),
            Feature::Formatter(feature) => Ok(()),
        }
    }
}
