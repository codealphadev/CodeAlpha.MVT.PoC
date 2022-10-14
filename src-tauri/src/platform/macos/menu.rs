use tauri::{Menu, MenuEntry, MenuItem, Submenu};

pub fn mac_os_task_bar_menu() -> MenuEntry {
    #[cfg(target_os = "macos")]
    MenuEntry::Submenu(Submenu::new(
        "dummy-menu-for-shortcuts-to-work-on-input-fields-see-github-issue-#-1055",
        Menu::with_items([
            MenuItem::Undo.into(),
            MenuItem::Redo.into(),
            MenuItem::Cut.into(),
            MenuItem::Copy.into(),
            MenuItem::Paste.into(),
            MenuItem::SelectAll.into(),
        ]),
    ))
}
