import type { LogicalFrame } from "../src-tauri/bindings/geometry/LogicalFrame";

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