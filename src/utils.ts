import type { LogicalFrame } from "../src-tauri/bindings/geometry/LogicalFrame";
import type { LogicalPosition } from "../src-tauri/bindings/geometry/LogicalPosition";

export function convert_global_frame_to_local(global_frame: LogicalFrame, reference_position_global: LogicalPosition): LogicalFrame {
    return {
        origin: {
            x: global_frame.origin.x - reference_position_global.x,
            y: global_frame.origin.y - reference_position_global.y
        },
        size: {
            width: global_frame.size.width,
            height: global_frame.size.height
        }
    };
}

export function convert_global_position_to_local(global_position: LogicalPosition, reference_position_global: LogicalPosition): LogicalPosition {
    return {
        x: global_position.x - reference_position_global.x,
        y: global_position.y - reference_position_global.y
    }
}

export function convert_local_position_to_global(local_position: LogicalPosition, reference_position_global: LogicalPosition): LogicalPosition {
    return {
        x: local_position.x + reference_position_global.x,
        y: local_position.y + reference_position_global.y
    }
}