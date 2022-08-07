use accessibility::{AXAttribute, AXUIElement, AXUIElementAttributes, AXValue, Error};
use accessibility_sys::{
    kAXErrorNoValue, kAXTrustedCheckOptionPrompt, pid_t, AXIsProcessTrusted,
    AXIsProcessTrustedWithOptions,
};
use cocoa::appkit::CGPoint;
use core_foundation::{
    base::{CFHash, CFRange, TCFType},
    boolean::CFBoolean,
    dictionary::CFDictionary,
    string::CFString,
};
use core_graphics_types::geometry::CGSize;
use rdev::{simulate, EventType};

use crate::core_engine::text_types::TextRange;

static EDITOR_NAME: &str = "Xcode";

// Method to get the focused AXUIElement's top-level window
pub fn currently_focused_app() -> Result<AXUIElement, Error> {
    let system_wide_element = AXUIElement::system_wide();
    let focused_ui_element = system_wide_element.attribute(&AXAttribute::focused_uielement())?;
    let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;
    if let Ok(parent) = focused_window.attribute(&AXAttribute::parent()) {
        return Ok(parent);
    } else {
        return Ok(focused_ui_element);
    }
}

// Method to get the focused AXUIElement's top-level window
pub fn currently_focused_window() -> Result<AXUIElement, Error> {
    let system_wide_element = AXUIElement::system_wide();
    let focused_ui_element = system_wide_element.attribute(&AXAttribute::focused_uielement())?;
    let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;

    Ok(focused_window)
}

pub fn is_currently_focused_app_editor() -> Option<bool> {
    if let Ok(focused_app) = currently_focused_app() {
        if let Ok(app_title) = focused_app.attribute(&AXAttribute::title()) {
            if app_title == EDITOR_NAME {
                return Some(true);
            } else {
                return Some(false);
            }
        }
    }

    None
}

pub fn is_currently_focused_app_our_app() -> Option<bool> {
    let system_wide_element = AXUIElement::system_wide();

    if let Ok(focused_ui_element) = system_wide_element.attribute(&AXAttribute::focused_uielement())
    {
        if let Ok(app_pid) = focused_ui_element.pid() {
            let our_app_pid: i32 = std::process::id().try_into().unwrap();
            if app_pid == our_app_pid {
                return Some(true);
            } else {
                return Some(false);
            }
        }
    }

    None
}

pub fn focused_uielement_of_app(app_pid: pid_t) -> Result<AXUIElement, Error> {
    let application = AXUIElement::application(app_pid);
    let focused_ui_element = application.focused_uielement()?;

    Ok(focused_ui_element)
}

pub fn _get_xcode_editor_content(pid: pid_t) -> Result<Option<String>, Error> {
    if is_focused_uielement_of_app_xcode_editor_field(pid)? {
        let editor_element = focused_uielement_of_app(pid)?;

        let content = editor_element.value()?;
        let content_str = content.downcast::<CFString>();

        if let Some(cf_str) = content_str {
            return Ok(Some(cf_str.to_string()));
        }
    }

    Ok(None)
}

pub fn _get_textarea_origin(pid: pid_t) -> Result<Option<tauri::LogicalPosition<f64>>, Error> {
    if is_focused_uielement_of_app_xcode_editor_field(pid)? {
        let editor_element = focused_uielement_of_app(pid)?;

        let point = (editor_element.position()?).get_value::<CGPoint>()?;
        let scroll_position = tauri::LogicalPosition {
            x: point.x as f64,
            y: point.y as f64,
        };
        return Ok(Some(scroll_position));
    }
    Ok(None)
}

pub fn send_event_mouse_wheel(pid: pid_t, delta: tauri::LogicalSize<f64>) -> Result<bool, Error> {
    if is_focused_uielement_of_app_xcode_editor_field(pid)? {
        let event_type = EventType::Wheel {
            delta_x: delta.width as i64,
            delta_y: delta.height as i64,
        };

        match simulate(&event_type) {
            Ok(()) => {
                return Ok(true);
            }
            Err(_) => {
                println!("We could not send {:?}", event_type);
            }
        }
    }
    Ok(false)
}

pub fn update_xcode_editor_content(pid: pid_t, content: &str) -> Result<bool, Error> {
    if is_focused_uielement_of_app_xcode_editor_field(pid)? {
        let editor_element = focused_uielement_of_app(pid)?;

        let content_cf_str: CFString = content.into();

        editor_element.set_attribute(&AXAttribute::value(), content_cf_str.as_CFType())?;

        return Ok(true);
    }

    Ok(false)
}

pub fn set_selected_text_range(pid: pid_t, index: usize, length: usize) -> Result<bool, Error> {
    if is_focused_uielement_of_app_xcode_editor_field(pid)? {
        let editor_element = focused_uielement_of_app(pid)?;
        let range = CFRange {
            location: index as isize,
            length: length as isize,
        };

        editor_element.set_attribute(
            &AXAttribute::selected_text_range(),
            AXValue::from_CFRange(range).unwrap(),
        )?;

        return Ok(true);
    }

    Ok(false)
}

pub fn get_selected_text_range(pid: pid_t) -> Result<Option<TextRange>, Error> {
    if is_focused_uielement_of_app_xcode_editor_field(pid)? {
        let editor_element = focused_uielement_of_app(pid)?;

        let selected_text_range_ax_value = editor_element.selected_text_range()?;
        let selected_text_range_cf_range = selected_text_range_ax_value.get_value::<CFRange>()?;

        return Ok(Some(TextRange {
            index: selected_text_range_cf_range.location as usize,
            length: selected_text_range_cf_range.length as usize,
        }));
    }

    Ok(None)
}

pub fn is_focused_uielement_of_app_xcode_editor_field(app_pid: pid_t) -> Result<bool, Error> {
    let focused_ui_element = focused_uielement_of_app(app_pid)?;
    // let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;
    // let parent = focused_window.attribute(&AXAttribute::parent())?;
    // let title = parent.attribute(&AXAttribute::title())?;

    let role = focused_ui_element.attribute(&AXAttribute::role())?;

    if role == "AXTextArea" {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Checks whether or not this application is a trusted accessibility client.
pub fn application_is_trusted() -> bool {
    unsafe {
        return AXIsProcessTrusted();
    }
}

/// Same as [application_is_trusted], but also shows the user a prompt asking
/// them to allow accessibility API access if it hasn't already been given.
pub fn application_is_trusted_with_prompt() -> bool {
    unsafe {
        let option_prompt = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
        let dict: CFDictionary<CFString, CFBoolean> =
            CFDictionary::from_CFType_pairs(&[(option_prompt, CFBoolean::true_value())]);
        return AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef());
    }
}

/// If a window hash is provided, get the focused element of this window, otherwise, get the focused
/// element of the editor application and proceed if it is a textarea
///
/// Arguments:
///
/// * `editor_app_pid`: The process ID of the editor application.
/// * `editor_window_hash`: The hash of the editor window that the textarea is in.
///
/// Returns:
///
/// An Option<AXUIElement>
pub fn get_textarea_uielement(editor_app_pid: i32) -> Option<AXUIElement> {
    let focused_uielement = if let Ok(focused_uielement) = focused_uielement_of_app(editor_app_pid)
    {
        focused_uielement
    } else {
        return None;
    };

    // Only proceed if focused UI element is a textarea
    if let Ok(role) = focused_uielement.role() {
        if role == "AXTextArea" {
            return Some(focused_uielement);
        }
    }

    None
}

pub fn get_file_path_from_window(window: &AXUIElement) -> Result<String, Error> {
    let full_file_path = window.document()?.to_string();
    let (_, file_path) = full_file_path.split_at(7);

    Ok(file_path.to_string())
}

pub fn generate_axui_element_hash(ui_element: &AXUIElement) -> usize {
    unsafe { CFHash(ui_element.as_CFTypeRef()) }
}

pub fn _window_ui_element_from_hash(pid: pid_t, hash: usize) -> Result<AXUIElement, Error> {
    let application = AXUIElement::application(pid);

    let app_windows = application.windows()?;

    for window in &app_windows {
        if generate_axui_element_hash(&window) == hash {
            return Ok(window.clone());
        }
    }

    Err(Error::Ax(kAXErrorNoValue))
}

/// Method takes the AXUIElement of the editor's textarea and returns its size without the scrollbars
pub fn derive_xcode_textarea_dimensions(
    child_element: &AXUIElement,
) -> Result<(tauri::LogicalPosition<f64>, tauri::LogicalSize<f64>), Error> {
    let scrollarea_element = child_element.attribute(&AXAttribute::parent())?;

    // Get Size and Origin of AXScrollArea
    let scrollarea_pos_ax_value = scrollarea_element.attribute(&AXAttribute::position())?;
    let scrollarea_size_ax_value = scrollarea_element.attribute(&AXAttribute::size())?;

    let scrollarea_origin = scrollarea_pos_ax_value.get_value::<CGPoint>()?;
    let scrollarea_size = scrollarea_size_ax_value.get_value::<CGSize>()?;

    // Get all children
    let mut updated_width = scrollarea_size.width;
    let mut updated_origin_x = scrollarea_origin.x;
    let children_elements = scrollarea_element.attribute(&AXAttribute::children())?;

    for child in &children_elements {
        if let Ok(identifier) = child.attribute(&AXAttribute::identifier()) {
            let identifier_list: [&str; 3] = [
                "Source Editor Change Gutter",
                "Source Editor Gutter",
                "Source Editor Minimap",
            ];

            if identifier_list.contains(&identifier.to_string().as_str()) {
                updated_width -= child
                    .attribute(&AXAttribute::size())?
                    .get_value::<CGSize>()?
                    .width;

                if identifier.to_string() != "Source Editor Minimap" {
                    updated_origin_x += child
                        .attribute(&AXAttribute::size())?
                        .get_value::<CGSize>()?
                        .width;
                }
            }
        }
    }

    // Update EditorWindowResizedMessage
    let position = tauri::LogicalPosition {
        x: updated_origin_x,
        y: scrollarea_origin.y,
    };

    let size = tauri::LogicalSize {
        width: updated_width,
        height: scrollarea_size.height,
    };

    return Ok((position, size));
}

#[derive(Debug, Clone)]
pub struct XCodeObserverState {
    pub app_handle: tauri::AppHandle,
    pub window_list: Vec<(
        uuid::Uuid,
        AXUIElement,
        Option<tauri::LogicalSize<f64>>,
        usize,
    )>,
}

#[derive(Debug, Clone)]
pub struct AppObserverState {
    pub app_handle: tauri::AppHandle,
}

pub mod DebugUtils {

    use accessibility::{
        AXAttribute, AXUIElement, AXUIElementAttributes, TreeVisitor, TreeWalker, TreeWalkerFlow,
    };
    use colored::*;
    use core_foundation::{array::CFArray, string::CFString};
    use std::cell::Cell;

    pub fn _print_element_ax_properties(element: &AXUIElement) {
        let walker = TreeWalker::new();

        println!("=============== Tree Print Run Start ===============\n");
        walker.walk(element, &AXTreePrinter::_new(0));
        println!("\n================ Tree Print Run End ================\n");
    }

    pub fn _print_ax_tree(element: &AXUIElement, max_tree_level: usize) {
        let walker = TreeWalker::new();

        println!("=============== Tree Print Run Start ===============\n");
        walker.walk(element, &AXTreePrinter::_new(max_tree_level));
        println!("================ Tree Print Run End ================\n");
    }

    // A class that prints the AX tree in a 'pretty way' to stdout.
    // How it works:
    struct AXTreePrinter {
        indent: String,
        children: AXAttribute<CFArray<AXUIElement>>,

        // Using a Cell instead of a simple usize because I don't want to define the self
        // argument in the trait functions for enter & exit as mutable
        tree_level: Cell<usize>,
        max_tree_level: Cell<usize>,
    }

    impl AXTreePrinter {
        pub fn _new(max_tree_level: usize) -> Self {
            Self {
                tree_level: Cell::new(0),
                indent: " ".repeat(4),
                children: AXAttribute::children(),
                max_tree_level: Cell::new(max_tree_level),
            }
        }

        pub fn _new_with_indentation(indent: usize, max_tree_level: usize) -> Self {
            Self {
                tree_level: Cell::new(0),
                indent: " ".repeat(indent),
                children: AXAttribute::children(),
                max_tree_level: Cell::new(max_tree_level),
            }
        }
    }

    impl TreeVisitor for AXTreePrinter {
        fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
            let indent = self.indent.repeat(self.tree_level.get());
            let role = element.role().unwrap_or_else(|_| CFString::new(""));

            self.tree_level.replace(self.tree_level.get() + 1);
            println![
                "{}- {} ({} children)",
                indent,
                role.to_string().bright_yellow().bold(),
                element.children().unwrap().len()
            ];

            if let Ok(names) = element.attribute_names() {
                for name in names.into_iter() {
                    if &*name == self.children.as_CFString() {
                        continue;
                    }

                    if let Ok(value) = element.attribute(&AXAttribute::new(&*name)) {
                        let value_str = format!("{:?}", value);
                        println![
                            "{}|. {}: {}",
                            indent,
                            (*name).to_string().bold(),
                            value_str.green()
                        ];
                    }
                }
            }

            if self.tree_level.get() > self.max_tree_level.get() {
                TreeWalkerFlow::SkipSubtree
            } else {
                TreeWalkerFlow::Continue
            }
        }

        fn exit_element(&self, _element: &AXUIElement) {
            self.tree_level.replace(self.tree_level.get() - 1);
        }
    }
}
