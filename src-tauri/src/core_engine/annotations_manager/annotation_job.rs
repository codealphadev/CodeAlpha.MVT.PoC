use crate::core_engine::{features::FeatureKind, TextRange};

pub enum AnnotationKind {
    OpeningBracket,
    ClosingBracket,
    LineStart,
    LineEnd,
    Elbow,
    FirstCharCodeblock,
    LastCharCodeblock,
}

// Wrapped lines are tricky to handle using the macOS AX API. Lines wrapping always yield a rectangle that stretches
// to the fill width of the code editor. This enum specifies if the consumer accepts this or wants the AnnotationManager
// to put more effort into finding a better approximating rectangle for the underlying code block.
pub enum InstructionWrappedLines {
    None,                    // Return the pure AX API results
    LeftWhitespaceCorrected, // Check how much whitespace is on the left side of the line and correct the rectangle accordingly
    Accurate, // Figure out on which characters the wrapped line splits and return the rectangle for each of them; only works `InstructionBounds::RectCollection`
}

// Specify which of the properties of the resulting rectangles should be sent to the frontend. It can be either the rectangle frame or one of its cornor positions.
// The feature "BracketHighlighting" is a user for this.
pub enum InstructionBoundsPropertyOfInterest {
    Frame,       // Return the frame of the rectangle
    PosTopLeft,  // Return the top left position of the rectangle
    PosTopRight, // Return the top right position of the rectangle
    PosBotLeft,  // Return the bottom left position of the rectangle
    PosBotRight, // Return the bottom right position of the rectangle
}

// If a result would span multiple lines, it will be split into multiple rects, each one line high. The rects will be ordered from top to bottom.
// If selected `SingleRect` and the result spans multiple lines, the rect will be the union of all the lines.
pub enum InstructionBounds {
    SingleRect,
    RectCollection,
}

// MacOS AXAPI only returns bounds for character indexes if these are within "VisibleTextRange". The VisibleTextRange is bigger than the actual viewport, which allows
// the AnnotationManager to fetch the bounds for characters that are not visible in the viewport yet and send it to the frontend. For the CodeOverlay window it is helpful
// information to know if the missing property is above or below the viewport.
pub enum ViewportPositioning {
    Visible,
    InvisibleAbove,
    InvisibleBelow,
}

pub struct AnnotationJobInstructions {
    pub bounds: InstructionBounds,
    pub bounds_property_of_interest: InstructionBoundsPropertyOfInterest,
    pub wrapped_lines: InstructionWrappedLines,
}

pub struct AnnotationJob {
    pub id: uuid::Uuid,
    pub range: TextRange,
    pub kind: AnnotationKind,
    pub feature: FeatureKind,
    pub instructions: AnnotationJobInstructions,
}
