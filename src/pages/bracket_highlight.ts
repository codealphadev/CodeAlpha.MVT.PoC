import { component_subscribe } from 'svelte/internal';
import type { BracketHighlightBracketPair } from '../../src-tauri/bindings/bracket_highlight/BracketHighlightBracketPair';
import type { MatchRectangle } from '../../src-tauri/bindings/rules/utils/MatchRectangle';

const ADJUST_Y = 3;
const THICKNESS_BASE = 17;

export const compute_bracket_highlight_thickness = (bracket_pair: BracketHighlightBracketPair) => {
	return Math.floor(bracket_pair.first.rectangle.size.height / THICKNESS_BASE);
};

export const compute_bracket_highlight_box_rects = (
	bracket_highlight_boxes: BracketHighlightBracketPair
): [MatchRectangle, MatchRectangle] => {
	return [bracket_highlight_boxes.first.rectangle, bracket_highlight_boxes.last.rectangle];
};

export const compute_bracket_highlight_line_rect = (
	lines_pair: BracketHighlightBracketPair
): MatchRectangle => {
	const thickness = compute_bracket_highlight_thickness(lines_pair);
	// Check if last and first bracket are visible
	let is_last_bracket_visible = true; // TODO: check if last bracket is visible
	let is_on_same_line = lines_pair.first.text_position.row === lines_pair.last.text_position.row;

	let line_rectangle = null;
	if (is_on_same_line) {
		line_rectangle = {
			origin: {
				x: lines_pair.first.rectangle.origin.x + lines_pair.first.rectangle.size.width,
				y: lines_pair.first.rectangle.origin.y + lines_pair.first.rectangle.size.height + thickness
			},
			size: {
				width:
					lines_pair.last.rectangle.origin.x -
					lines_pair.first.rectangle.origin.x -
					lines_pair.first.rectangle.size.width,
				height: 0
			}
		};
	} else {
		if (!is_last_bracket_visible) {
			// bracket_highlight_line_rectangle_last = {
			// 	origin: {
			// 		x: 0,
			// 		y: bracket_highlight_line_rectangle_first.origin.y + ADJUST_BRACKET_HIGHLIGHT_Y
			// 	},
			// 	size: {
			// 		width: bracket_highlight_line_rectangle_first.size.width,
			// 		height: null
			// 	}
			// };
		}
		line_rectangle = {
			origin: {
				x: lines_pair.last.rectangle.origin.x,
				y: lines_pair.first.rectangle.origin.y + lines_pair.first.rectangle.size.height + thickness
			},
			size: {
				width:
					lines_pair.first.rectangle.origin.x -
					lines_pair.last.rectangle.origin.x +
					lines_pair.last.rectangle.size.width,
				height: lines_pair.last.rectangle.origin.y - lines_pair.first.rectangle.origin.y - ADJUST_Y
			}
		};
	}
	// Remove line if last bracket is right of first bracket
	// if (
	// 	!is_on_same_line &&
	// 	bracket_highlight_line_rectangle_last &&
	// 	bracket_highlight_line_rectangle_first.origin.x < bracket_highlight_line_rectangle_last.origin.x
	// ) {
	// 	line_rectangle = null;
	// }
	return line_rectangle;
};
