#![allow(dead_code)]

use accessibility::{AXAttribute, AXUIElement, Error};
use accessibility_sys::{
    kAXTrustedCheckOptionPrompt, pid_t, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
};
use core_foundation::{
    base::TCFType, boolean::CFBoolean, dictionary::CFDictionary, string::CFString,
};

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
    } else {
        return Some(false);
    }

    None
}

pub fn focused_uielement_of_app(app_pid: pid_t) -> Result<AXUIElement, Error> {
    let application = AXUIElement::application(app_pid);
    let focused_ui_element = application.attribute(&AXAttribute::focused_uielement())?;

    Ok(focused_ui_element)
}

pub fn is_focused_uielement_of_app_xcode_editor_field(app_pid: pid_t) -> Result<bool, Error> {
    let focused_ui_element = focused_uielement_of_app(app_pid)?;
    let focused_window = focused_ui_element.attribute(&AXAttribute::top_level_ui_element())?;
    let parent = focused_window.attribute(&AXAttribute::parent())?;
    let title = parent.attribute(&AXAttribute::title())?;

    let role = focused_ui_element.attribute(&AXAttribute::role())?;

    if role == "AXTextArea" && title == EDITOR_NAME {
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

#[derive(Debug, Clone)]
pub struct XCodeObserverState {
    pub app_handle: tauri::AppHandle,
    pub window_list: Vec<(uuid::Uuid, AXUIElement, Option<tauri::LogicalSize<f64>>)>,
}

#[derive(Debug, Clone)]
pub struct AppObserverState {
    pub app_handle: tauri::AppHandle,
}

pub mod DebugUtils {

    #![allow(dead_code)]

    use accessibility::{
        AXAttribute, AXUIElement, AXUIElementAttributes, TreeVisitor, TreeWalker, TreeWalkerFlow,
    };
    use colored::*;
    use core_foundation::{array::CFArray, string::CFString};
    use std::cell::Cell;

    pub fn print_element_ax_properties(element: &AXUIElement) {
        let walker = TreeWalker::new();

        println!("=============== Tree Print Run Start ===============\n");
        walker.walk(element, &AXTreePrinter::new(0));
        println!("\n================ Tree Print Run End ================\n");
    }

    pub fn _print_ax_tree(element: &AXUIElement, max_tree_level: usize) {
        let walker = TreeWalker::new();

        println!("=============== Tree Print Run Start ===============\n");
        walker.walk(element, &AXTreePrinter::new(max_tree_level));
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
        pub fn new(max_tree_level: usize) -> Self {
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
