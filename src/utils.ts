import type { LogicalFrame } from "../src-tauri/bindings/geometry/LogicalFrame";
import type { LogicalPosition } from "../src-tauri/bindings/geometry/LogicalPosition";

export function convert_global_frame_to_local_frame(global_frame: LogicalFrame, document_rectangle_position: LogicalPosition): LogicalFrame {
    return {
        origin: {
            x: global_frame.origin.x - document_rectangle_position.x,
            y: global_frame.origin.y - document_rectangle_position.y
        },
        size: {
            width: global_frame.size.width,
            height: global_frame.size.height
        }
    };
}

export function convert_global_position_to_local_position(global_position: LogicalPosition, document_rectangle_position: LogicalPosition): LogicalPosition {
    return {
        x: global_position.x - document_rectangle_position.x,
        y: global_position.y - document_rectangle_position.y
    }
}