<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { BracketHighlightResults } from '../../../../src-tauri/bindings/bracket_highlight/BracketHighlightResults';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import {
		adjust_bracket_results_for_overlay,
		BORDER_WIDTH,
		compute_bracket_highlight_box_rects,
		compute_bracket_highlight_line_rect
	} from './bracket_highlight';
	import { colors } from '../../../themes';

	export let code_overlay_rectangle: LogicalFrame | null;

	let bracket_highlight_line_rectangle: LogicalFrame | null = null;
	let bracket_highlight_box_rectangle_first: LogicalFrame | null = null;
	let bracket_highlight_box_rectangle_last: LogicalFrame | null = null;
	let bottom_elbow_rectangle: LogicalFrame | null = null;

	const listenTauriEvents = async () => {
		let BracketHighlightChannel: ChannelList = 'BracketHighlightResults';
		await listen(BracketHighlightChannel, (event) => {
			if (code_overlay_rectangle) {
				const adjusted_bracket_highlight_results = adjust_bracket_results_for_overlay(
					event.payload as BracketHighlightResults,
					code_overlay_rectangle.origin
				);

				bracket_highlight_line_rectangle = null;
				bracket_highlight_box_rectangle_first = null;
				bracket_highlight_box_rectangle_last = null;
				bottom_elbow_rectangle = null;

				if (adjusted_bracket_highlight_results) {
					[bracket_highlight_box_rectangle_first, bracket_highlight_box_rectangle_last] =
						compute_bracket_highlight_box_rects(adjusted_bracket_highlight_results.boxes);

					[bracket_highlight_line_rectangle, bottom_elbow_rectangle] =
						compute_bracket_highlight_line_rect(
							adjusted_bracket_highlight_results,
							code_overlay_rectangle.size
						);
				}
			}
		});
	};

	listenTauriEvents();

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};
</script>

{#if bracket_highlight_line_rectangle !== null}
	<div
		style="position: absolute; 
		top: {round_value(bracket_highlight_line_rectangle.origin.y, 2)}px; 
		left: {round_value(bracket_highlight_line_rectangle.origin.x, 1)}px; 
		width: {round_value(bracket_highlight_line_rectangle.size.width, 2)}px;
		height: {round_value(bracket_highlight_line_rectangle.size.height, 2)}px; 
		border-style: solid; border-top-width: {BORDER_WIDTH}px; border-color: {colors.inactive}; border-left-width: {BORDER_WIDTH}px; border-right-width: 0; border-bottom-width: 0;"
	/>
{/if}
{#if bracket_highlight_box_rectangle_first !== null}
	<div
		style="position: absolute; 
		top: {round_value(bracket_highlight_box_rectangle_first.origin.y, 2)}px; 
		left: {round_value(bracket_highlight_box_rectangle_first.origin.x, 2)}px; 
		width: {round_value(bracket_highlight_box_rectangle_first.size.width, 2)}px;
		height: {round_value(bracket_highlight_box_rectangle_first.size.height, 2)}px; 
		border-style: solid; border-width: {BORDER_WIDTH}px; border-color: {colors.inactive};"
	/>
{/if}
{#if bracket_highlight_box_rectangle_last !== null}
	<div
		style="position: absolute; 
		top: {round_value(bracket_highlight_box_rectangle_last.origin.y, 2)}px; 
		left: {round_value(bracket_highlight_box_rectangle_last.origin.x, 2)}px; 
		width: {round_value(bracket_highlight_box_rectangle_last.size.width, 2)}px;
		height: {round_value(bracket_highlight_box_rectangle_last.size.height, 2)}px; 
		border-style: solid; border-width: {BORDER_WIDTH}px; border-color: {colors.inactive};"
	/>
{/if}
{#if bottom_elbow_rectangle !== null}
	<div
		style="position: absolute; 
		top: {round_value(bottom_elbow_rectangle.origin.y, 2)}px; 
		left: {round_value(bottom_elbow_rectangle.origin.x, 1) / 10}px; 
		width: {round_value(bottom_elbow_rectangle.size.width, 2) / 10}px;
		height: {round_value(bottom_elbow_rectangle.size.height, 2)}px; 
		border-bottom-style: solid; border-bottom-width: {BORDER_WIDTH}px; border-color: {colors.inactive};"
	/>
{/if}
