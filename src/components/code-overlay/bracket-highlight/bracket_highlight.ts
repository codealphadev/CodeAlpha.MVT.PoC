import type { LogicalPosition } from '../../../../src-tauri/bindings/geometry/LogicalPosition';
import type { LogicalSize } from '../../../../src-tauri/bindings/geometry/LogicalSize';
import type { BracketHighlightResults } from '../../../../src-tauri/bindings/bracket_highlight/BracketHighlightResults';
import type { BracketHighlightBracketPair } from '../../../../src-tauri/bindings/bracket_highlight/BracketHighlightBracketPair';
import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';

export const BORDER_WIDTH = 1;
const LEFT_MOST_LINE_X = 16;

export const compute_bracket_highlight_box_rects = (
	bracket_highlight_boxes: BracketHighlightBracketPair
): [LogicalFrame, LogicalFrame] => {
	let first_box_rect = null;
	if (bracket_highlight_boxes.first) {
		const first_rect = bracket_highlight_boxes.first.rectangle;
		first_box_rect = {
			origin: {
				x: first_rect.origin.x + BORDER_WIDTH,
				y: first_rect.origin.y + BORDER_WIDTH
			},
			size: {
				width: first_rect.size.width - BORDER_WIDTH * 2,
				height: first_rect.size.height - BORDER_WIDTH * 2
			}
		};
	}

	let last_box_rect = null;
	if (bracket_highlight_boxes.last) {
		const last_rect = bracket_highlight_boxes.last.rectangle;
		last_box_rect = {
			origin: {
				x: last_rect.origin.x + BORDER_WIDTH,
				y: last_rect.origin.y + BORDER_WIDTH
			},
			size: {
				width: last_rect.size.width - BORDER_WIDTH * 2,
				height: last_rect.size.height - BORDER_WIDTH * 2
			}
		};
	}

	return [first_box_rect, last_box_rect];
};

export const compute_bracket_highlight_line_rect = (
	bracket_results: BracketHighlightResults,
	outerSize: LogicalSize
): [LogicalFrame, LogicalFrame] => {
	let lines_pair = bracket_results.lines;

	let first_line_rect = lines_pair.first ? lines_pair.first.rectangle : null;
	let last_line_rect = lines_pair.last ? lines_pair.last.rectangle : null;
	let elbow = bracket_results.elbow;

	// Check if last and first bracket are visible
	let is_last_bracket_visible = !!lines_pair.last;
	let is_on_same_line =
		first_line_rect && last_line_rect && first_line_rect.origin.y === last_line_rect.origin.y;

	let bottom_line_rectangle = null;

	let line_rectangle = null;
	if (is_on_same_line) {
		line_rectangle = {
			origin: {
				x: first_line_rect.origin.x + first_line_rect.size.width - BORDER_WIDTH,
				y: first_line_rect.origin.y + first_line_rect.size.height - BORDER_WIDTH * 2
			},
			size: {
				width:
					last_line_rect.origin.x -
					first_line_rect.origin.x -
					first_line_rect.size.width +
					BORDER_WIDTH * 3,
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
						y: first_line_rect.origin.y + first_line_rect.size.height - BORDER_WIDTH * 2
					},
					size: {
						width:
							first_line_rect.origin.x -
							LEFT_MOST_LINE_X +
							first_line_rect.size.width -
							BORDER_WIDTH * 2,
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
					x: last_line_rect.origin.x + BORDER_WIDTH,
					y: 0
				},
				size: {
					width: 0,
					height: last_line_rect.origin.y + BORDER_WIDTH
				}
			};
		} else if (first_line_rect && last_line_rect) {
			// Both brackets visible
			line_rectangle = {
				origin: {
					x: last_line_rect.origin.x + BORDER_WIDTH,
					y: first_line_rect.origin.y + first_line_rect.size.height - BORDER_WIDTH * 2
				},
				size: {
					width:
						first_line_rect.origin.x -
						last_line_rect.origin.x +
						last_line_rect.size.width -
						BORDER_WIDTH * 2,
					height: last_line_rect.origin.y - first_line_rect.origin.y + BORDER_WIDTH
				}
			};
		}

		if (elbow) {
			let elbow_x = elbow.origin_x_left_most ? LEFT_MOST_LINE_X : elbow.origin_x;
			line_rectangle.origin.x = elbow_x;
			line_rectangle.size.width = first_line_rect.origin.x - elbow_x + BORDER_WIDTH * 2;
			if (last_line_rect) {
				bottom_line_rectangle = {
					origin: {
						x: elbow_x,
						y: last_line_rect.origin.y + last_line_rect.size.height - BORDER_WIDTH * 2
					},
					size: {
						width: last_line_rect.origin.x + BORDER_WIDTH * 2 - elbow_x,
						height: 0
					}
				};
			}
		}

		if (elbow && elbow.bottom_line_top && last_line_rect) {
			bottom_line_rectangle.origin.y = last_line_rect.origin.y + BORDER_WIDTH;
			line_rectangle.size.height -= last_line_rect.size.height - BORDER_WIDTH * 2;
		}
	}

	return [line_rectangle, bottom_line_rectangle];
};

export const adjust_bracket_results_for_overlay = (
	bracket_results: BracketHighlightResults,
	outerPosition: LogicalPosition
): BracketHighlightResults => {
	if (!bracket_results) {
		return null;
	}
	if (bracket_results.lines.first) {
		bracket_results.lines.first.rectangle = adjust_rectangle(
			bracket_results.lines.first.rectangle,
			outerPosition
		);
	}
	if (bracket_results.lines.last) {
		bracket_results.lines.last.rectangle = adjust_rectangle(
			bracket_results.lines.last.rectangle,
			outerPosition
		);
	}
	if (bracket_results.boxes.first) {
		bracket_results.boxes.first.rectangle = adjust_rectangle(
			bracket_results.boxes.first.rectangle,
			outerPosition
		);
	}
	if (bracket_results.boxes.last) {
		bracket_results.boxes.last.rectangle = adjust_rectangle(
			bracket_results.boxes.last.rectangle,
			outerPosition
		);
	}

	if (bracket_results.elbow && bracket_results.elbow.origin_x) {
		bracket_results.elbow.origin_x = bracket_results.elbow.origin_x - outerPosition.x;
	}

	return bracket_results;
};

const adjust_rectangle = (rectangle: LogicalFrame, outerPosition: LogicalPosition) => {
	if (!rectangle) {
		return null;
	}

	return {
		origin: {
			x: rectangle.origin.x - outerPosition.x,
			y: rectangle.origin.y - outerPosition.y
		},
		size: rectangle.size
	};
};
