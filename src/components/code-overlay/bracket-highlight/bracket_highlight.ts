import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
import type { LogicalSize } from '@tauri-apps/api/window';
export const BORDER_WIDTH = 1; // TODO: Do we need this? Can we just use an outline?
import type { Elbow } from '../../../../src-tauri/bindings/features/bracket_highlighting/Elbow';
import type { BracketHighlightResults } from '../../../../src-tauri/bindings/features/bracket_highlighting/BracketHighlightResults';

export const compute_bracket_highlight_rects = (
	lines: BracketHighlightResults['lines'],
	codeoverlay_rect_height: LogicalSize['height']
): { top_rect: LogicalFrame | null; bottom_rect: LogicalFrame | null } => {
	const { start, end, elbow } = lines;

	const left_x = Math.min(
		// TODO: Arrayify to get rid of these infinities
		start?.x ?? Infinity,
		end?.x ?? Infinity,
		get_elbow_x(elbow) ?? Infinity // TODO: Simplify
	);

	const bottom_y = end?.y ?? codeoverlay_rect_height;

	const top_y = start?.y ?? 0;

	return {
		top_rect:
			start && start.x > left_x
				? {
						origin: {
							x: left_x + 1, // Add border width because vertical line is handled by bottom rect
							y: top_y
						},
						size: {
							width: start.x - left_x, // TODO: Should this be absolute instead of max?
							height: 0.0
						}
				  }
				: null,
		bottom_rect: end
			? {
					origin: {
						x: left_x,
						y: top_y
					},
					size: {
						width: end.x - left_x,
						height: bottom_y - top_y
					}
			  }
			: null
	};
};

function is_known_elbow(elbow: Elbow): elbow is { KnownElbow: number } {
	return elbow.hasOwnProperty('KnownElbow');
}

function get_elbow_x(elbow: Elbow | null): number | null {
	if (elbow === null) {
		return null;
	}
	return is_known_elbow(elbow) ? elbow.KnownElbow : elbow.EstimatedElbowOffset;
}
