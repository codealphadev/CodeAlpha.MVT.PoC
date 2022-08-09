import type { LogicalSize } from '../../src-tauri/bindings/geometry/LogicalSize';
import type { BracketHighlightBracketPair } from '../../src-tauri/bindings/bracket_highlight/BracketHighlightBracketPair';
import type { BracketHighlightResults } from '../../src-tauri/bindings/bracket_highlight/BracketHighlightResults';
import type { LogicalPosition } from '../../src-tauri/bindings/geometry/LogicalPosition';
import type { MatchRectangle } from '../../src-tauri/bindings/rules/utils/MatchRectangle';
import type { BracketHighlightElbow } from '../../src-tauri/bindings/bracket_highlight/BracketHighlightElbow';

export const BORDER_WIDTH = 1;
const LEFT_MOST_LINE_X = 5;

export const compute_bracket_highlight_box_rects = (
	bracket_highlight_boxes: BracketHighlightBracketPair,
	outerPosition: LogicalPosition
): [MatchRectangle, MatchRectangle] => {
	let first_box_rect = bracket_highlight_boxes.first
		? bracket_highlight_boxes.first.rectangle
		: null;
	let last_box_rect = bracket_highlight_boxes.last ? bracket_highlight_boxes.last.rectangle : null;
	return [
		adjust_rectangle(first_box_rect, outerPosition),
		adjust_rectangle(last_box_rect, outerPosition)
	];
};

export const compute_bracket_highlight_line_rect = (
	bracket_results: BracketHighlightResults,
	outerPosition: LogicalPosition,
	outerSize: LogicalSize
): [MatchRectangle, MatchRectangle] => {
	let lines_pair = bracket_results.lines;

	// Adjust positions to overlay
	let first_line_rect = adjust_rectangle(
		lines_pair.first ? lines_pair.first.rectangle : null,
		outerPosition
	);
	let last_line_rect = adjust_rectangle(
		lines_pair.last ? lines_pair.last.rectangle : null,
		outerPosition
	);
	let elbow: BracketHighlightElbow = bracket_results.elbow
		? {
				origin_x: bracket_results.elbow.origin_x - outerPosition.x,
				origin_x_left_most: bracket_results.elbow.origin_x_left_most,
				bottom_line_top: bracket_results.elbow.bottom_line_top
		  }
		: null;

	// Check if last and first bracket are visible
	let is_last_bracket_visible = !!lines_pair.last;
	let is_on_same_line =
		first_line_rect && last_line_rect && first_line_rect.origin.y === last_line_rect.origin.y;

	let line_rectangle = null;
	if (is_on_same_line) {
		line_rectangle = {
			origin: {
				x: first_line_rect.origin.x + first_line_rect.size.width,
				y: first_line_rect.origin.y + first_line_rect.size.height - BORDER_WIDTH
			},
			size: {
				width: last_line_rect.origin.x - first_line_rect.origin.x - first_line_rect.size.width,
				height: 0
			}
		};
	} else {
		if (!is_last_bracket_visible) {
			if (lines_pair.first) {
				// Only first bracket is visible
				line_rectangle = {
					origin: {
						x: LEFT_MOST_LINE_X,
						y: first_line_rect.origin.y + first_line_rect.size.height - BORDER_WIDTH
					},
					size: {
						width: first_line_rect.origin.x - LEFT_MOST_LINE_X + first_line_rect.size.width,
						height: outerSize.height - first_line_rect.origin.y + first_line_rect.size.height
					}
				};
			} else {
				// no brackets visible
				line_rectangle = {
					origin: {
						x: LEFT_MOST_LINE_X,
						y: 0
					},
					size: {
						width: 0,
						height: outerSize.height
					}
				};
			}
		} else if (!lines_pair.first) {
			// Only last bracket visible
			line_rectangle = {
				origin: {
					x: last_line_rect.origin.x,
					y: 0
				},
				size: {
					width: 0,
					height: outerSize.height - last_line_rect.origin.y
				}
			};
		} else if (first_line_rect && last_line_rect) {
			// Both brackets visible
			line_rectangle = {
				origin: {
					x: last_line_rect.origin.x,
					y: first_line_rect.origin.y + first_line_rect.size.height - BORDER_WIDTH
				},
				size: {
					width: first_line_rect.origin.x - last_line_rect.origin.x + last_line_rect.size.width,
					height: last_line_rect.origin.y - first_line_rect.origin.y
				}
			};
		}
	}

	let bottom_line_rectangle = null;

	if (elbow) {
		let elbow_x = elbow.origin_x_left_most ? LEFT_MOST_LINE_X : elbow.origin_x;
		line_rectangle.origin.x = elbow_x;
		line_rectangle.size.width = first_line_rect.origin.x - elbow_x;
		if (last_line_rect) {
			bottom_line_rectangle = {
				origin: {
					x: elbow_x,
					y: last_line_rect.origin.y + last_line_rect.size.height - BORDER_WIDTH
				},
				size: {
					width: last_line_rect.origin.x + BORDER_WIDTH - elbow_x,
					height: 0
				}
			};
		}
	}

	if (elbow && elbow.bottom_line_top && last_line_rect) {
		bottom_line_rectangle.origin.y = last_line_rect.origin.y;
		line_rectangle.size.height -= last_line_rect.size.height - BORDER_WIDTH;
	}

	return [line_rectangle, bottom_line_rectangle];
};

const adjust_rectangle = (rectangle: MatchRectangle, position: LogicalPosition) => {
	if (!rectangle || !position) {
		return null;
	}

	return {
		origin: {
			x: rectangle.origin.x - position.x,
			y: rectangle.origin.y - position.y
		},
		size: rectangle.size
	};
};
