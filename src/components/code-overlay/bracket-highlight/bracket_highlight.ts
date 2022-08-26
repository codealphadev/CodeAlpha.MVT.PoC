import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
import type { LogicalPosition } from '../../../../src-tauri/bindings/geometry/LogicalPosition';
import type { LogicalSize } from '@tauri-apps/api/window';
export const BORDER_WIDTH = 1;
const LEFT_MOST_LINE_X = 16 + 55;


export const compute_bracket_highlight_line_rect = (
	opening_bracket: LogicalFrame | null,
	closing_bracket: LogicalFrame | null,
	codeoverlay_rect_height: LogicalSize['height']
): LogicalFrame => {
	let line_rectangle: LogicalFrame = line_rect_if_no_brackets_visible(codeoverlay_rect_height);

	// If both brackets are on the same line, drawing a rectangle between the opening and closing brackets.
	if (opening_bracket && closing_bracket && opening_bracket.origin.y === closing_bracket.origin.y) {
		line_rectangle = line_rect_if_both_brackets_on_same_line(opening_bracket, closing_bracket);
	}

	// Only first bracket is visible
	else if (opening_bracket && !closing_bracket) {
		line_rectangle = line_rect_if_only_opening_bracket_visible(opening_bracket, codeoverlay_rect_height);
	}

	// Only second bracket is visible
	else if (!opening_bracket && closing_bracket) {
		line_rectangle = line_rect_if_only_closing_bracket_visible(closing_bracket);
	}

	// Both brackets are visible
	else if (opening_bracket && closing_bracket) {
		line_rectangle = line_rect_if_both_brackets_visible(opening_bracket, closing_bracket);
	}

	return line_rectangle;
};

const line_rect_if_both_brackets_on_same_line = (
	opening_bracket: LogicalFrame,
	closing_bracket: LogicalFrame
): LogicalFrame => {
	return {
		origin: {
			x: opening_bracket.origin.x + opening_bracket.size.width,
			y: opening_bracket.origin.y + opening_bracket.size.height - BORDER_WIDTH
		},
		size: {
			width: Math.max(
				0,
				closing_bracket.origin.x - opening_bracket.origin.x - opening_bracket.size.width
			),
			height: 0
		}
	};
};

const line_rect_if_only_opening_bracket_visible = (
	opening_bracket: LogicalFrame,
	codeoverlay_rect_height: LogicalSize['height']
): LogicalFrame => {
	return {
		origin: {
			x: Math.min(opening_bracket.origin.x,  LEFT_MOST_LINE_X),
			y: opening_bracket.origin.y + opening_bracket.size.height - BORDER_WIDTH
		},
		size: {
			width: Math.max(0, opening_bracket.origin.x - ( LEFT_MOST_LINE_X)),
			height: Math.max(
				0,
				codeoverlay_rect_height - opening_bracket.origin.y
			)
		}
	};
};

const line_rect_if_only_closing_bracket_visible = (
	closing_bracket: LogicalFrame,
): LogicalFrame => {
	return {
		origin: {
			x: closing_bracket.origin.x,
			y: 0
		},
		size: {
			width: 0,
			height: Math.max(0, closing_bracket.origin.y)
		}
	};
};

const line_rect_if_no_brackets_visible = (code_document_frame_height: LogicalSize['height']): LogicalFrame => {
	return {
		origin: {
			x: LEFT_MOST_LINE_X,
			y: 0
		},
		size: {
			width: 0,
			height: code_document_frame_height
		}
	};
};

const line_rect_if_both_brackets_visible = (
	opening_bracket: LogicalFrame,
	closing_bracket: LogicalFrame
): LogicalFrame => {
	return {
		origin: {
			x: Math.min(opening_bracket.origin.x, closing_bracket.origin.x),
			y: opening_bracket.origin.y + opening_bracket.size.height - BORDER_WIDTH
		},
		size: {
			width: Math.max(0, opening_bracket.origin.x - closing_bracket.origin.x),
			height: closing_bracket.origin.y - opening_bracket.origin.y
		}
	};
};

export const correct_highlight_rectangles_with_elbow_point = (
	line_rectangle: LogicalFrame,
	closing_bracket: LogicalFrame | null,
	codeoverlay_rect_height: LogicalSize['height'],
	elbow_origin: LogicalPosition | null,
	elbow_origin_x_left_most: boolean,
	bottom_line_top: boolean
): [LogicalFrame, LogicalFrame] => {
	// If 'origin_x_left_most' is false, 'origin' is always Some() -> backend logic
	const updated_elbow_origin = elbow_origin_x_left_most
		? { x: LEFT_MOST_LINE_X, y: 0 }
		: elbow_origin!;

	// Update the line_rectangle size to include the diff between elbow_x and the line_rectangle origin_x
	line_rectangle.size.width += Math.max(0, line_rectangle.origin.x - updated_elbow_origin.x);
	line_rectangle.size.height = Math.max(0, updated_elbow_origin.y - line_rectangle.origin.y);

	// Update the line_rectangle origin to the elbow_x
	line_rectangle.origin.x = updated_elbow_origin.x;

	let elbow_rectangle: LogicalFrame = {
		origin: {
			x: updated_elbow_origin.x,
			y: line_rectangle.origin.y + line_rectangle.size.height
		},
		size: {
			width: closing_bracket ? Math.max(0, closing_bracket.origin.x - updated_elbow_origin.x) : 0,
			height: closing_bracket
				? Math.max(
						0,
						closing_bracket.origin.y - (line_rectangle.origin.y + line_rectangle.size.height)
				  )
				: codeoverlay_rect_height - updated_elbow_origin.y
		}
	};

	// Special case: we need to draw the bottom line at height of the closing_bracket's bottom left corner
	// instead of the closing_bracket's top left corner
	if (!bottom_line_top) {
		elbow_rectangle.size.height += closing_bracket ? closing_bracket.size.height : 0;
	} else {
		elbow_rectangle.size.height += BORDER_WIDTH;
	}

	return [line_rectangle, elbow_rectangle];
};
