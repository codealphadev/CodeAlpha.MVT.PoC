use crate::ax_interaction::models::editor::EditorShortcutPressedMessage;

use super::{formatter::SwiftFormatter, BracketHighlight, DocsGenerator};

use std::error::Error;

pub enum CoreEngineTrigger {
    OnShortcutPressed(EditorShortcutPressedMessage),
    OnTextContentChange,
    OnTextSelectionChange,
    OnViewportMove,
    OnViewportDimensionsChange,
    OnVisibleTextRangeChange,
}

pub enum Feature<'a> {
    BracketHighlighting(BracketHighlight<'a>),
    DocsGeneration(DocsGenerator),
    Formatter(SwiftFormatter<'a>),
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
    fn compute(&mut self, trigger: &CoreEngineTrigger) -> Result<(), FeatureError>;
    fn update_visualization(&mut self, trigger: &CoreEngineTrigger) -> Result<(), FeatureError>;
}

impl<'a> FeatureBase for Feature<'a> {
    fn compute(&mut self, trigger: &CoreEngineTrigger) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => Ok(()),
            Feature::DocsGeneration(feature) => Ok(()),
            Feature::Formatter(feature) => Ok(()),
        }
    }

    fn update_visualization(&mut self, trigger: &CoreEngineTrigger) -> Result<(), FeatureError> {
        match self {
            Feature::BracketHighlighting(feature) => Ok(()),
            Feature::DocsGeneration(feature) => Ok(()),
            Feature::Formatter(feature) => Ok(()),
        }
    }
}
