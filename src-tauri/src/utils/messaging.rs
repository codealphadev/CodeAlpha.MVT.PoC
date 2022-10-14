#![allow(unused)]
use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ChannelList {
    AXEventApp,
    AXEventXcode,
    BracketHighlightResults,
    EventInputDevice,
    EventRuleExecutionState,
    EventTrackingAreas,
    EventUserInteractions,
    EventWindowControls,
    AnnotationEvent,
    RuleResults,
    EventViewport,
    NodeExplanationEvent,
    SuggestionEvent,
}
impl fmt::Display for ChannelList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChannelList::AXEventApp => write!(f, "AXEventApp"),
            ChannelList::AXEventXcode => write!(f, "AXEventXcode"),
            ChannelList::BracketHighlightResults => write!(f, "BracketHighlightResults"),
            ChannelList::EventInputDevice => write!(f, "EventInputDevice"),
            ChannelList::EventRuleExecutionState => write!(f, "EventRuleExecutionState"),
            ChannelList::EventTrackingAreas => write!(f, "EventTrackingAreas"),
            ChannelList::EventUserInteractions => write!(f, "EventUserInteractions"),
            ChannelList::EventWindowControls => write!(f, "EventWindowControls"),
            ChannelList::AnnotationEvent => write!(f, "AnnotationEvent"),
            ChannelList::RuleResults => write!(f, "RuleResults"),
            ChannelList::EventViewport => write!(f, "EventViewport"),
            ChannelList::NodeExplanationEvent => write!(f, "NodeExplanationEvent"),
            ChannelList::SuggestionEvent => write!(f, "SuggestionEvent"),
        }
    }
}
