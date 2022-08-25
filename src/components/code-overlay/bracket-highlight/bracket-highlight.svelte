<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { BracketHighlightResults } from '../../../../src-tauri/bindings/features/bracket_highlighting/BracketHighlightResults';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import {
		BORDER_WIDTH,
		compute_bracket_highlight_line_rect,
		correct_highlight_rectangles_with_elbow_point
	} from './bracket_highlight';
	import { colors } from '../../../themes';

	export let code_overlay_rectangle: LogicalFrame | null;

	let line_rectangle = reset_highlight_rectangle();
	let elbow_rectangle = reset_highlight_rectangle();

	let opening_bracket_box_highlight: LogicalFrame | null = null;
	let closing_bracket_box_highlight: LogicalFrame | null = null;

	const listenTauriEvents = async () => {
		let BracketHighlightChannel: ChannelList = 'BracketHighlightResults';
		await listen(BracketHighlightChannel, (event) => {
			const bracket_highlights = event.payload as BracketHighlightResults;

			if (bracket_highlights == null) {
				opening_bracket_box_highlight = null;
				closing_bracket_box_highlight = null;
				line_rectangle = reset_highlight_rectangle();
				elbow_rectangle = reset_highlight_rectangle();
				return;
			}

			opening_bracket_box_highlight = bracket_highlights.boxes.first
				? bracket_highlights.boxes.first.rectangle
				: null;

			closing_bracket_box_highlight = bracket_highlights.boxes.last
				? bracket_highlights.boxes.last.rectangle
				: null;

			// Calculate the line rectangle
			if (code_overlay_rectangle) {
				line_rectangle = compute_bracket_highlight_line_rect(
					bracket_highlights.lines.first ? bracket_highlights.lines.first.rectangle : null,
					bracket_highlights.lines.last ? bracket_highlights.lines.last.rectangle : null,
					code_overlay_rectangle
				);

				// Calculate the elbow rectangle
				if (bracket_highlights.elbow) {
					[line_rectangle, elbow_rectangle] = correct_highlight_rectangles_with_elbow_point(
						line_rectangle,
						bracket_highlights.lines.last ? bracket_highlights.lines.last.rectangle : null,
						code_overlay_rectangle,
						bracket_highlights.elbow.origin,
						bracket_highlights.elbow.origin_x_left_most,
						bracket_highlights.elbow.bottom_line_top
					);
				} else {
					[line_rectangle, elbow_rectangle] = correct_highlight_rectangles_with_elbow_point(
						line_rectangle,
						bracket_highlights.lines.last ? bracket_highlights.lines.last.rectangle : null,
						code_overlay_rectangle,
						line_rectangle.origin,
						false,
						true
					);
				}
			} else {
				line_rectangle = reset_highlight_rectangle();
				elbow_rectangle = reset_highlight_rectangle();
			}
		});
	};
	listenTauriEvents();

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};

	function reset_highlight_rectangle(): LogicalFrame {
		return {
			origin: {
				x: 0,
				y: 0
			},
			size: {
				width: 0,
				height: 0
			}
		};
	}
</script>

{#if opening_bracket_box_highlight !== null && code_overlay_rectangle !== null}
	<div
		style="position: absolute; 
		top: {round_value(opening_bracket_box_highlight.origin.y - code_overlay_rectangle.origin.y, 2)}px; 
		left: {round_value(opening_bracket_box_highlight.origin.x - code_overlay_rectangle.origin.x, 2)}px; 
		width: {round_value(opening_bracket_box_highlight.size.width, 2)}px;
		height: {round_value(opening_bracket_box_highlight.size.height, 2)}px; 
		border-style: solid; border-width: {BORDER_WIDTH}px; border-color: {colors.inactive};"
	/>
{/if}
{#if closing_bracket_box_highlight !== null && code_overlay_rectangle !== null}
	<div
		style="position: absolute; 
		top: {round_value(closing_bracket_box_highlight.origin.y - code_overlay_rectangle.origin.y, 2)}px; 
		left: {round_value(closing_bracket_box_highlight.origin.x - code_overlay_rectangle.origin.x, 2)}px; 
		width: {round_value(closing_bracket_box_highlight.size.width, 2)}px;
		height: {round_value(closing_bracket_box_highlight.size.height, 2)}px; 
		border-style: solid; 
		border-width: {BORDER_WIDTH}px; 
		border-color: {colors.inactive};"
	/>
{/if}
{#if line_rectangle !== null && code_overlay_rectangle !== null}
	<div
		style="position: absolute; 
		top: {round_value(line_rectangle.origin.y - code_overlay_rectangle.origin.y, 2)}px; 
		left: {round_value(line_rectangle.origin.x - code_overlay_rectangle.origin.x, 2)}px; 
		width: {round_value(line_rectangle.size.width, 2)}px;
		height: {round_value(line_rectangle.size.height, 2)}px; 
		border-style: solid; 
		border-color: {colors.inactive}; 
		border-top-width: {BORDER_WIDTH}px; 
		border-left-width: {BORDER_WIDTH}px;
		border-right-width: 0; 
		border-bottom-width: 0;"
	/>
{/if}
{#if elbow_rectangle !== null && code_overlay_rectangle !== null}
	<div
		style="position: absolute; 
		top: {round_value(elbow_rectangle.origin.y - code_overlay_rectangle.origin.y, 2)}px; 
		left: {round_value(elbow_rectangle.origin.x - code_overlay_rectangle.origin.x, 2)}px; 
		width: {round_value(elbow_rectangle.size.width, 2)}px;
		height: {round_value(elbow_rectangle.size.height, 2)}px; 
		border-bottom-style: solid; 
		border-left-width: {BORDER_WIDTH}px;
		border-bottom-width: {BORDER_WIDTH}px; 
		border-color: {colors.inactive};"
	/>
{/if}
