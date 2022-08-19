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

	export let code_overlay_rectangle: LogicalFrame | null;

	let bracket_highlight_line_rectangle: LogicalFrame = null;
	let bracket_highlight_box_rectangle_first: LogicalFrame = null;
	let bracket_highlight_box_rectangle_last: LogicalFrame = null;
	let bottom_elbow_rectangle: LogicalFrame = null;

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
</script>

{#if bracket_highlight_line_rectangle !== null}
	<div
		style="position: absolute; top: {Math.round(
			bracket_highlight_line_rectangle.origin.y
		)}px; left: {Math.round(bracket_highlight_line_rectangle.origin.x)}px; width: {Math.round(
			bracket_highlight_line_rectangle.size.width
		)}px;height: {Math.round(
			bracket_highlight_line_rectangle.size.height
		)}px; border-style: solid; border-top-width: {BORDER_WIDTH}px; border-color: var(--theme-inactive); border-left-width: {BORDER_WIDTH}px; border-right-width: 0; border-bottom-width: 0;"
	/>
{/if}
{#if bracket_highlight_box_rectangle_first !== null}
	<div
		style="position: absolute; top: {Math.round(
			bracket_highlight_box_rectangle_first.origin.y
		)}px; left: {Math.round(bracket_highlight_box_rectangle_first.origin.x)}px; width: {Math.round(
			bracket_highlight_box_rectangle_first.size.width
		)}px;height: {Math.round(
			bracket_highlight_box_rectangle_first.size.height
		)}px; border-style: solid; border-width: {BORDER_WIDTH}px; border-color: var(--theme-inactive);"
	/>
{/if}
{#if bracket_highlight_box_rectangle_last !== null}
	<div
		style="position: absolute; top: {Math.round(
			bracket_highlight_box_rectangle_last.origin.y
		)}px; left: {Math.round(bracket_highlight_box_rectangle_last.origin.x)}px; width: {Math.round(
			bracket_highlight_box_rectangle_last.size.width
		)}px;height: {Math.round(
			bracket_highlight_box_rectangle_last.size.height
		)}px; border-style: solid; border-width: {BORDER_WIDTH}px; border-color: var(--theme-inactive);"
	/>
{/if}
{#if bottom_elbow_rectangle !== null}
	<div
		style="position: absolute; top: {Math.round(
			bottom_elbow_rectangle.origin.y
		)}px; left: {Math.round(bottom_elbow_rectangle.origin.x)}px; width: {Math.round(
			bottom_elbow_rectangle.size.width
		)}px;height: {Math.round(
			bottom_elbow_rectangle.size.height
		)}px; border-bottom-style: solid; border-bottom-width: {BORDER_WIDTH}px; border-color: var(--theme-inactive);"
	/>
{/if}
