import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
import type { LogicalPosition } from '../../../../src-tauri/bindings/geometry/LogicalPosition';
import type { LogicalSize } from '@tauri-apps/api/window';
export const BORDER_WIDTH = 1;
import type { Elbow } from '../../../../src-tauri/bindings/features/bracket_highlighting/Elbow';
import type { BracketHighlightResults } from '../../../../src-tauri/bindings/features/bracket_highlighting/BracketHighlightResults';

export const compute_bracket_highlight_rects = (
	lines: BracketHighlightResults['lines'],
	codeoverlay_rect_height: LogicalSize['height']
): { line_rect: LogicalFrame; elbow_rect: LogicalFrame | null } => {
	const { start, end, elbow } = lines;

	const origin_x = Math.min(
		// TODO: Arrayify to get rid of these infinities
		start?.x ?? Infinity,
		end?.x ?? Infinity,
		get_elbow_x(elbow) ?? Infinity // TODO: Simplify
	);

	const bottom_y = end?.y ?? codeoverlay_rect_height;

	return {
		line_rect: {
			origin: {
				x: origin_x,
				y: start?.y ?? 0 //-  BORDER_WIDTH
			},
			size: {
				width: Math.max(0, (start?.x ?? 0) - origin_x), // TODO: Should this be absolute instead of max?
				height: bottom_y - (start?.y ?? 0)
			}
		},
		elbow_rect: end
			? {
					origin: {
						x: origin_x,
						y: end.y
					},
					size: {
						height: 0,
						width: end.x - origin_x
					}
			  }
			: null
	};
};

function is_known_elbow(elbow: Elbow): elbow is { KnownElbow: LogicalPosition } {
	return elbow.hasOwnProperty('KnownElbow');
}

function get_elbow_x(elbow: Elbow | null): number | null {
	if (elbow === null) {
		return null;
	}
	return is_known_elbow(elbow) ? elbow.KnownElbow.x : elbow.EstimatedElbowOffset;
}
