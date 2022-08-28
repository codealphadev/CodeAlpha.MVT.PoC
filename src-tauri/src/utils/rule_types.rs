#![allow(unused)]

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core_engine::{TextRange, XcodeText},
    utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize},
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct MatchRectangle {
    pub origin: LogicalPosition,
    pub size: LogicalSize,
}

pub type LineMatch = (MatchRange, Vec<LogicalFrame>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/rules/utils/")]
pub struct MatchRange {
    pub string: XcodeText,
    pub range: TextRange,
}

impl MatchRange {
    pub fn from_text_and_range(text: &XcodeText, range: &TextRange) -> Option<Self> {
        if text.len() < range.index + range.length {
            return None;
        }
        Some(Self {
            string: XcodeText::from_array(&text[(range.index)..(range.index + range.length)]),
            range,
        })
    }
}

#[cfg(test)]
mod tests_MatchRange {

    use crate::core_engine::{TextRange, XcodeText};

    use super::MatchRange;

    #[test]
    fn test_from_text_and_range() {
        let s = &XcodeText::from_str(&"0123456789");

        let match_range = MatchRange::from_text_and_range(
            s,
            TextRange {
                index: 2,
                length: 5,
            },
        );

        assert_eq!(
            match_range,
            Some(MatchRange {
                string: XcodeText::from_str("23456"),
                range: TextRange {
                    index: 2,
                    length: 5,
                },
            })
        );

        let match_range_out_of_range = MatchRange::from_text_and_range(
            s,
            TextRange {
                index: 10,
                length: 5,
            },
        );
        assert_eq!(match_range_out_of_range, None);
    }
}
