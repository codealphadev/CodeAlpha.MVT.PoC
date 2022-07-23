use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorShortcutPressedMessage {
    pub key: String,
    pub menu_item_title: String, // <-- expected to be "Save", unknown if it is different in other languages
    pub modifier: ModifierKey,
    pub ui_elem_hash: usize,
}
