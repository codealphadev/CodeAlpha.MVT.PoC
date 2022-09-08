<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { BracketHighlightResults } from '../../../../src-tauri/bindings/features/bracket_highlighting/BracketHighlightResults';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import { colors } from '../../../themes';
	import { BORDER_WIDTH, compute_bracket_highlight_rects } from './bracket_highlight';

	export let code_document_rect: LogicalFrame;
	export let active_window_uid: number;

	let top_rectangle: LogicalFrame | null = null;
	let bottom_rectangle: LogicalFrame | null = null;

	let opening_bracket_box: LogicalFrame | null = null;
	let closing_bracket_box: LogicalFrame | null = null;

	let results_window_uid: number | null = null;

	const listenTauriEvents = async () => {
		let BracketHighlightChannel: ChannelList = 'BracketHighlightResults';
		await listen(BracketHighlightChannel, (event) => {
			const bracket_highlights = event.payload as BracketHighlightResults | null;
			if (bracket_highlights == null) {
				opening_bracket_box = null;
				closing_bracket_box = null;
				top_rectangle = null;
				bottom_rectangle = null;
				results_window_uid = null;
				return;
			}

			opening_bracket_box = bracket_highlights.boxes.opening_bracket;
			closing_bracket_box = bracket_highlights.boxes.closing_bracket;
			const rectangles = compute_bracket_highlight_rects(
				bracket_highlights.lines,
				code_document_rect.size.height
			);
			top_rectangle = rectangles.top_rect;
			bottom_rectangle = rectangles.bottom_rect;

			results_window_uid = bracket_highlights.window_uid;
		});
	};
	listenTauriEvents();

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};
</script>

{#if results_window_uid == active_window_uid}
	{#if opening_bracket_box}
		<div
			style="position: absolute; 
			top: {round_value(opening_bracket_box.origin.y, 2)}px; 
			left: {round_value(opening_bracket_box.origin.x, 2)}px; 
			width: {round_value(opening_bracket_box.size.width, 2)}px;
			height: {round_value(opening_bracket_box.size.height, 2)}px; 
			border-style: solid; border-width: {BORDER_WIDTH}px; border-color: {colors.inactive};"
		/>
	{/if}
	{#if closing_bracket_box !== null && code_document_rect !== null}
		<div
			style="position: absolute; 
			top: {round_value(closing_bracket_box.origin.y, 2)}px; 
			left: {round_value(closing_bracket_box.origin.x, 2)}px; 
			width: {round_value(closing_bracket_box.size.width, 2)}px;
			height: {round_value(closing_bracket_box.size.height, 2)}px; 
			border-style: solid; 
			border-width: {BORDER_WIDTH}px; 
			border-color: {colors.inactive};"
		/>
	{/if}
	{#if top_rectangle !== null}
		<div
			style="position: absolute; 
			top: {round_value(top_rectangle.origin.y, 2)}px; 
			left: {round_value(top_rectangle.origin.x, 2)}px; 
			width: {round_value(top_rectangle.size.width, 2)}px;
			height: {round_value(top_rectangle.size.height, 2)}px; 
			border-style: solid; 
			border-color: {colors.inactive}; 
			border-top-width: {BORDER_WIDTH}px; 
			border-left-width: {BORDER_WIDTH}px;
			border-right-width: 0; 
			border-bottom-width: 0;"
		/>
	{/if}
	{#if bottom_rectangle !== null}
		<div
			style="position: absolute; 
			top: {round_value(bottom_rectangle.origin.y, 2)}px; 
			left: {round_value(bottom_rectangle.origin.x, 2)}px; 
			width: {round_value(bottom_rectangle.size.width, 2)}px;
			height: {round_value(bottom_rectangle.size.height, 2)}px; 
			border-bottom-style: solid; 
			border-left-width: {BORDER_WIDTH}px;
			border-bottom-width: {BORDER_WIDTH}px; 
			border-color: {colors.inactive};"
		/>
	{/if}
{/if}
