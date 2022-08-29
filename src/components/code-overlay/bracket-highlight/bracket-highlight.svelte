<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { BracketHighlightResults } from '../../../../src-tauri/bindings/features/bracket_highlighting/BracketHighlightResults';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import { colors } from '../../../themes';
	import {
		BORDER_WIDTH,
		compute_bracket_highlight_rects,
	} from './bracket_highlight';
	
	export let code_document_rect: LogicalFrame;

	let line_rectangle: LogicalFrame | null = null;
	let elbow_rectangle: LogicalFrame | null = null;

	let opening_bracket_box: LogicalFrame | null = null;
	let closing_bracket_box: LogicalFrame | null = null;

	
	const listenTauriEvents = async () => {
		let BracketHighlightChannel: ChannelList = 'BracketHighlightResults';
		await listen(BracketHighlightChannel, (event) => {
			const payload = event.payload as BracketHighlightResults | null;
			if (payload == null) {
				opening_bracket_box = null;
				closing_bracket_box = null;
				line_rectangle = null;
				elbow_rectangle = null;
				return;
			}
			const bracket_highlights = payload;

			opening_bracket_box = bracket_highlights.boxes.opening_bracket;
			closing_bracket_box = bracket_highlights.boxes.closing_bracket;

			const rectangles = compute_bracket_highlight_rects(
				bracket_highlights.lines,
				code_document_rect.size.height
			);
			line_rectangle = rectangles.line_rect;
			elbow_rectangle = rectangles.elbow_rect;
	
		});
	};
	listenTauriEvents();

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};
</script>

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
{#if line_rectangle !== null}
	<div
		style="position: absolute; 
		top: {round_value(line_rectangle.origin.y, 2)}px; 
		left: {round_value(line_rectangle.origin.x, 2)}px; 
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
{#if elbow_rectangle !== null}
	<div
		style="position: absolute; 
		top: {round_value(elbow_rectangle.origin.y, 2)}px; 
		left: {round_value(elbow_rectangle.origin.x, 2)}px; 
		width: {round_value(elbow_rectangle.size.width, 2)}px;
		height: {round_value(elbow_rectangle.size.height, 2)}px; 
		border-bottom-style: solid; 
		border-left-width: {BORDER_WIDTH}px;
		border-bottom-width: {BORDER_WIDTH}px; 
		border-color: {colors.inactive};"
	/>
{/if}
