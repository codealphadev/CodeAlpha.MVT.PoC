import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
import type { LogicalSize } from '@tauri-apps/api/window';
export const BORDER_WIDTH = 1; // TODO: Do we need this? Can we just use an outline?
import type { LogicalPosition } from '../../../../src-tauri/bindings/geometry/LogicalPosition';

export const compute_bracket_highlight_lines = (
	start: LogicalPosition | null,
	end: LogicalPosition | null,
	elbow: LogicalPosition | null,
	codeoverlay_rect_height: LogicalSize['height']
): { top_rect: LogicalFrame | null; bottom_rect: LogicalFrame | null } => {
	const left_x = Math.min(
		// TODO: Arrayify to get rid of these infinities
		start?.x ?? Infinity,
		end?.x ?? Infinity,
		elbow?.x ?? Infinity // TODO: Simplify
	);

	const bottom_y = end?.y ?? codeoverlay_rect_height;

	const top_y = start?.y ?? 0;

	return {
		top_rect:
			start && start.x > left_x // No top_rect in one-line case
				? {
						origin: {
							x: left_x + (end ? 1 : 0), // Add border width because vertical line is handled by bottom rect
							y: top_y - 1 // -1 to correct top border by 1px
						},
						size: {
							width: start.x - left_x, // TODO: Should this be absolute instead of max?
							height: end ? 0.0 : bottom_y - top_y // If there is no bottom_rect, need to handle vertical line
						}
				  }
				: null,
		bottom_rect:
			end || !start // Draws the vertical line if there is no start and no end
				? {
						origin: {
							x: left_x,
							y: top_y - 1 // -1 to correct top border by 1px
						},
						size: {
							width: Math.max(0, (end?.x ?? 0) - left_x),
							height: bottom_y - top_y + 1
						}
				  }
				: null
	};
};
