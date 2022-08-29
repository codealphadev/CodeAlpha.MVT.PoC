#![allow(unused)]

use cocoa::appkit::CGPoint;
use core_graphics::geometry::CGSize;
use core_graphics_types::geometry::CGRect;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/geometry/")]
pub struct LogicalPosition {
    /// Vertical axis value.
    pub x: f64,
    /// Horizontal axis value.
    pub y: f64,
}

impl LogicalPosition {
    pub fn from_tauri_LogicalPosition(pos: &tauri::LogicalPosition<f64>) -> Self {
        Self { x: pos.x, y: pos.y }
    }

    pub fn from_CGPoint(point: &CGPoint) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }

    pub fn as_tauri_LogicalPosition(&self) -> tauri::LogicalPosition<f64> {
        tauri::LogicalPosition {
            x: self.x,
            y: self.y,
        }
    }

    pub fn as_CGPoint(&self) -> CGPoint {
        CGPoint {
            x: self.x,
            y: self.y,
        }
    }

    pub fn to_global(&self, local_origin: &LogicalPosition) -> LogicalPosition {
        LogicalPosition {
            x: self.x + local_origin.x,
            y: self.y + local_origin.y,
        }
    }

    pub fn to_local(&self, global_origin: &LogicalPosition) -> LogicalPosition {
        LogicalPosition {
            x: self.x - global_origin.x,
            y: self.y - global_origin.y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/geometry/")]
pub struct LogicalSize {
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

impl LogicalSize {
    pub fn from_tauri_LogicalSize(size: &tauri::LogicalSize<f64>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }

    pub fn from_CGSize(size: &CGSize) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }

    pub fn as_tauri_LogicalSize(&self) -> tauri::LogicalSize<f64> {
        tauri::LogicalSize {
            width: self.width,
            height: self.height,
        }
    }

    pub fn as_CGSize(&self) -> CGSize {
        CGSize {
            width: self.width,
            height: self.height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "bindings/geometry/")]
pub struct LogicalFrame {
    pub origin: LogicalPosition,
    pub size: LogicalSize,
}

impl LogicalFrame {
    pub fn new(origin: LogicalPosition, size: LogicalSize) -> Self {
        Self { origin, size }
    }

    pub fn from_CGRect(rect: &CGRect) -> Self {
        Self {
            origin: LogicalPosition::from_CGPoint(&rect.origin),
            size: LogicalSize::from_CGSize(&rect.size),
        }
    }

    pub fn as_CGRect(&self) -> CGRect {
        CGRect {
            origin: self.origin.as_CGPoint(),
            size: self.size.as_CGSize(),
        }
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        // Check if mouse_x and mouse_y are within the bounds of the rectangle.
        let x_in_bounds = x >= self.origin.x && x <= self.origin.x + self.size.width;
        let y_in_bounds = y >= self.origin.y && y <= self.origin.y + self.size.height;
        x_in_bounds && y_in_bounds
    }

    pub fn bottom_right(&self) -> LogicalPosition {
        LogicalPosition {
            x: self.origin.x + self.size.width - 1.0,
            y: self.origin.y + self.size.height - 1.0,
        }
    }

    pub fn bottom_left(&self) -> LogicalPosition {
        LogicalPosition {
            x: self.origin.x,
            y: self.origin.y + self.size.height - 1.0,
        }
    }

    pub fn top_left(&self) -> LogicalPosition {
        LogicalPosition {
            x: self.origin.x,
            y: self.origin.y,
        }
    }

    pub fn top_right(&self) -> LogicalPosition {
        LogicalPosition {
            x: self.origin.x + self.size.width - 1.0,
            y: self.origin.y,
        }
    }

    pub fn to_global(&self, local_origin: &LogicalPosition) -> LogicalFrame {
        LogicalFrame {
            origin: self.origin.to_global(local_origin),
            size: self.size,
        }
    }

    pub fn to_local(&self, global_origin: &LogicalPosition) -> LogicalFrame {
        LogicalFrame {
            origin: self.origin.to_local(global_origin),
            size: self.size,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::utils::geometry::{LogicalFrame, LogicalPosition, LogicalSize};

    #[test]
    fn contains_point() {
        let rectangle = LogicalFrame {
            origin: LogicalPosition { x: 0.0, y: 0.0 },
            size: LogicalSize {
                width: 100.0,
                height: 100.0,
            },
        };

        assert!(rectangle.contains_point(50., 50.));
        assert!(rectangle.contains_point(0., 0.));
        assert!(rectangle.contains_point(100., 100.));
        assert!(!rectangle.contains_point(101., 100.));
        assert!(!rectangle.contains_point(100., 101.));
        assert!(!rectangle.contains_point(150., 150.));
    }
}
