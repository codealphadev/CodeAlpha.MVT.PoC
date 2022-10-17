use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core_engine::{EditorWindowUid, TextRange};

use super::{
    annotations_manager::{Annotation, AnnotationError, AnnotationResult},
    AnnotationJobSingleChar,
};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, TS)]
#[ts(export, export_to = "bindings/features/code_annotations/")]
pub enum AnnotationKind {
    OpeningBracket,
    ClosingBracket,
    LineStart,
    LineEnd,
    Elbow,
    CodeblockFirstChar,
    CodeblockLastChar,
    ExtractionStartChar,
    ExtractionEndChar,
}

// Wrapped lines are tricky to handle using the macOS AX API. Lines wrapping always yield a rectangle that stretches
// to the fill width of the code editor. This enum specifies if the consumer accepts this or wants the AnnotationManager
// to put more effort into finding a better approximating rectangle for the underlying code block.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstructionWrappedLines {
    None,                    // Return the pure AX API results
    LeftWhitespaceCorrected, // Check how much whitespace is on the left side of the line and correct the rectangle accordingly
    Accurate, // Figure out on which characters the wrapped line splits and return the rectangle for each of them; only works `InstructionBounds::RectCollection`
}

// Specify which of the properties of the resulting rectangles should be sent to the frontend. It can be either the rectangle frame or one of its cornor positions.
// The feature "BracketHighlighting" is a user for this.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstructionBoundsPropertyOfInterest {
    Frame,       // Return the frame of the rectangle
    PosTopLeft,  // Return the top left position of the rectangle
    PosTopRight, // Return the top right position of the rectangle
    PosBotLeft,  // Return the bottom left position of the rectangle
    PosBotRight, // Return the bottom right position of the rectangle
}

// If a result would span multiple lines, it will be split into multiple rects, each one line high. The rects will be ordered from top to bottom.
// If selected `SingleRect` and the result spans multiple lines, the rect will be the union of all the lines.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstructionBounds {
    SingleRect,
    RectCollection,
}

// MacOS AXAPI only returns bounds for character indexes if these are within "VisibleTextRange". The VisibleTextRange is bigger than the actual viewport, which allows
// the AnnotationManager to fetch the bounds for characters that are not visible in the viewport yet and send it to the frontend. For the CodeOverlay window it is helpful
// information to know if the missing property is above or below the viewport.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, TS)]
#[ts(export, export_to = "bindings/features/code_annotations/")]
pub enum VisibleTextRangePositioning {
    Visible,
    InvisibleAbove,
    InvisibleBelow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnnotationJobInstructions {
    pub bounds: InstructionBounds,
    pub bounds_property_of_interest: InstructionBoundsPropertyOfInterest,
    pub wrapped_lines: InstructionWrappedLines,
}

impl Default for AnnotationJobInstructions {
    fn default() -> Self {
        Self {
            bounds: InstructionBounds::SingleRect,
            bounds_property_of_interest: InstructionBoundsPropertyOfInterest::Frame,
            wrapped_lines: InstructionWrappedLines::None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnnotationJob {
    SingleChar(AnnotationJobSingleChar), // Because SingleChar are much easier to compute, they are handled separately
}

pub trait AnnotationJobTrait {
    fn new(
        id: uuid::Uuid,
        range: &TextRange,
        kind: AnnotationKind,
        instructions: AnnotationJobInstructions,
    ) -> Self;

    fn id(&self) -> uuid::Uuid;

    // Computes the bounds for the given text range. Resets any previous result.
    fn compute_bounds(
        &mut self,
        visible_text_range: &TextRange,
        editor_window_uid: EditorWindowUid,
    ) -> Result<AnnotationResult, AnnotationError>;

    // Attempts to compute the bounds for the given text range only if no result is present yet, indicating that a previous
    // `compute_bounds` or subsequent `attempt_compute_bounds` didn't yield a result.
    fn compute_bounds_if_missing(
        &mut self,
        visible_text_range: &TextRange,
        editor_window_uid: EditorWindowUid,
    ) -> Result<AnnotationResult, AnnotationError>;

    fn get_annotation(&self) -> Option<Annotation>;
}

impl AnnotationJobTrait for AnnotationJob {
    fn new(
        id: uuid::Uuid,
        range: &TextRange,
        kind: AnnotationKind,
        instructions: AnnotationJobInstructions,
    ) -> Self {
        Self::SingleChar(AnnotationJobSingleChar::new(id, range, kind, instructions))
    }

    fn id(&self) -> uuid::Uuid {
        match self {
            Self::SingleChar(job) => job.id(),
        }
    }

    fn compute_bounds(
        &mut self,
        visible_text_range: &TextRange,
        editor_window_uid: EditorWindowUid,
    ) -> Result<AnnotationResult, AnnotationError> {
        match self {
            Self::SingleChar(job) => job.compute_bounds(visible_text_range, editor_window_uid),
        }
    }

    fn compute_bounds_if_missing(
        &mut self,
        visible_text_range: &TextRange,
        editor_window_uid: EditorWindowUid,
    ) -> Result<AnnotationResult, AnnotationError> {
        match self {
            Self::SingleChar(job) => {
                job.compute_bounds_if_missing(visible_text_range, editor_window_uid)
            }
        }
    }

    fn get_annotation(&self) -> Option<Annotation> {
        match self {
            Self::SingleChar(job) => job.get_annotation(),
        }
    }
}
