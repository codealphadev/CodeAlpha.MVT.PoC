use accessibility::{AXAttribute, AXValue};
use cocoa::appkit::CGFloat;
use core_foundation::{
    attributed_string::{CFAttributedString, CFAttributedStringRef},
    base::{CFRange, TCFType},
    dictionary::{CFDictionary, CFDictionaryRef},
    mach_port::CFIndex,
    string::{CFString, CFStringRef},
};
use core_graphics::sys::CGColorRef;

use crate::{
    core_engine::TextRange,
    utils::geometry::{LogicalFrame, LogicalPosition},
};

use super::{
    ax_helpers::{ax_attribute, generate_axui_element_hash},
    internal::get_focused_uielement,
    textarea::get_textarea_uielement,
    GetVia, XcodeError,
};

// [09/22] - I could not find a way to programmatically get this value
pub fn get_menu_bar_height(monitor: &LogicalFrame) -> f64 {
    if monitor.origin == (LogicalPosition { x: 0.0, y: 0.0 }) {
        // Case: primary monitor -> menu bar is 22 on most screens, only the new screens with the notch have a height of 38
        38.
    } else {
        // Case: any secondary monitor
        24.
    }
}

pub fn get_focused_window() -> Result<usize, XcodeError> {
    let focused_uielement = get_focused_uielement(&GetVia::Current)?;

    if let Ok(window_uielement) = ax_attribute(&focused_uielement, AXAttribute::window()) {
        Ok(generate_axui_element_hash(&window_uielement))
    } else {
        Err(XcodeError::FocusedWindowNotXcode)
    }
}

pub fn get_selected_text_range(get_via: &GetVia) -> Result<TextRange, XcodeError> {
    let textarea_uielement = get_textarea_uielement(get_via)?;

    let selected_text_range_ax_value =
        ax_attribute(&textarea_uielement, AXAttribute::selected_text_range())?;

    match selected_text_range_ax_value.get_value::<CFRange>() {
        Ok(selected_text_range_cf_range) => Ok(TextRange {
            index: selected_text_range_cf_range.location as usize,
            length: selected_text_range_cf_range.length as usize,
        }),
        Err(ax_error) => Err(XcodeError::AXError(ax_error.into())),
    }
}

pub fn set_selected_text_range(text_range: &TextRange, get_via: &GetVia) -> Result<(), XcodeError> {
    let textarea_uielement = get_textarea_uielement(get_via)?;

    textarea_uielement
        .set_attribute(
            &AXAttribute::selected_text_range(),
            AXValue::from_CFRange(CFRange {
                location: text_range.index as isize,
                length: text_range.length as isize,
            })
            .unwrap(),
        )
        .map_err(|err| XcodeError::AXError(err.into()))
}

pub fn get_visible_text_range(get_via: GetVia) -> Result<TextRange, XcodeError> {
    let textarea_uielement = get_textarea_uielement(&get_via)?;

    let visible_text_range_ax =
        ax_attribute(&textarea_uielement, AXAttribute::visible_character_range())?;

    match visible_text_range_ax.get_value::<CFRange>() {
        Ok(visible_text_range_cf) => Ok(TextRange {
            index: visible_text_range_cf.location as usize,
            length: visible_text_range_cf.length as usize,
        }),
        Err(ax_error) => Err(XcodeError::AXError(ax_error.into())),
    }
}

pub fn get_number_of_characters(get_via: GetVia) -> Result<i64, XcodeError> {
    let textarea_uielement = get_textarea_uielement(&get_via)?;

    let number_of_characters =
        ax_attribute(&textarea_uielement, AXAttribute::number_of_characters())?;

    number_of_characters.to_i64().ok_or(XcodeError::CustomError(
        "Could not convert CFNumber to i64".to_string(),
    ))
}

pub fn get_dark_mode() -> Result<bool, &'static str> {
    let textarea_uielement = get_textarea_uielement(&GetVia::Current)
        .map_err(|_| "Could not get textarea ui_element")?;

    let range = CFRange {
        location: 0,
        length: 1,
    };

    let str: CFAttributedString = textarea_uielement
        .parameterized_attribute(
            &AXAttribute::attributed_string_for_range(),
            &AXValue::from_CFRange(range).map_err(|_| "Could not create CFRange")?,
        )
        .map_err(|_| "Could not get attributed string")?;

    let attributes_dict: CFDictionary = unsafe {
        CFDictionary::wrap_under_get_rule(CFAttributedStringGetAttributes(
            str.as_concrete_TypeRef(),
            0,
            std::ptr::null(),
        ))
    };

    let keys_and_value_ptrs = attributes_dict.get_keys_and_values();
    let mut background_color_ptr = None;

    for i in 0..keys_and_value_ptrs.0.len() {
        let key =
            unsafe { CFString::wrap_under_get_rule((keys_and_value_ptrs.0)[i] as CFStringRef) };
        if key.to_string() == "AXBackgroundColor" {
            background_color_ptr = Some((keys_and_value_ptrs.1)[i]);
            break;
        }
    }
    if background_color_ptr.is_none() {
        return Err("Could not find background color");
    }

    let components: *const CGFloat =
        unsafe { CGColorGetComponents(background_color_ptr.unwrap() as CGColorRef) };

    let [r, g, b, _]: [_; 4] = unsafe {
        std::slice::from_raw_parts(components as *const CGFloat, 4)
            .try_into()
            .map_err(|_| "Could not convert components to array")?
    };

    let lightness = (r + g + b) / 3.0;

    return Ok(lightness < 0.5);
}

extern "C" {
    pub fn CFAttributedStringGetAttributes(
        aStr: CFAttributedStringRef,
        loc: CFIndex,
        effectiveRange: *const CFRange,
    ) -> CFDictionaryRef;

    pub fn CGColorGetComponents(color: CGColorRef) -> *const CGFloat;
}
