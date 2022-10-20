use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub enum ModifierKey {
    Cmd = 0,
    ShiftCmd = 1,
    OptionCmd = 2,
    CtrlCmd = 4,
    OptionAlt,
    Ctrl,
    CtrlOption,
    Shift,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub struct EditorShortcutPressedMessage {
    pub window_uid: usize,
    pub key: String,
    pub menu_item_title: String, // <-- expected to be "Save", unknown if it is different in other languages
    pub modifier: ModifierKey,
}
