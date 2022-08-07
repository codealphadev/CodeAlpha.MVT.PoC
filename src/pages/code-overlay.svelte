<script lang="ts">
	import { listen, Event } from '@tauri-apps/api/event';

	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { RuleResults } from '../../src-tauri/bindings/rules/RuleResults';
	import type { BracketHighlightResults } from '../../src-tauri/bindings/bracket_highlight/BracketHighlightResults';
	import type { MatchRectangle } from '../../src-tauri/bindings/rules/utils/MatchRectangle';

	import type { LogicalSize } from '../../src-tauri/bindings/geometry/LogicalSize';
	import type { LogicalPosition } from '../../src-tauri/bindings/geometry/LogicalPosition';
	import type { RuleName } from '../../src-tauri/bindings/rules/RuleName';
	import {
		compute_bracket_highlight_box_rects,
		compute_bracket_highlight_line_rect,
		compute_bracket_highlight_thickness
	} from './bracket_highlight';

	const ADJUST_BRACKET_HIGHLIGHT_Y = 3;

	type MatchId = string;

	let height = 0;
	let outerSize: LogicalSize | null = null;
	let outerPosition: LogicalPosition | null = null;

	let rule_results_arr: Array<RuleResults> | null = null;
	let highlightedRectangleMatchId = null;
	let bracket_highlight_line_rectangle: MatchRectangle = null;
	let bracket_highlight_box_rectangle_first: MatchRectangle = null;
	let bracket_highlight_box_rectangle_last: MatchRectangle = null;
	let bracket_highlight_thickness = 17;

	const listenTauriEvents = async () => {
		await listen('event-compute-height', (event) => {
			const tauriEvent = event as Event<MatchRectangle>;
			const payload: MatchRectangle = tauriEvent.payload;

			outerSize = payload.size;
			outerPosition = payload.origin;
			height = payload.size.height;

			compute_rule_rects();
		});

		let ResultsChannel: ChannelList = 'RuleResults';
		await listen(ResultsChannel, (event) => {
			const tauriEvent = event as Event<Array<RuleResults>>;
			rule_results_arr = tauriEvent.payload;
			compute_rule_rects();
		});

		let BracketHighlightChannel: ChannelList = 'BracketHighlightResults';
		await listen(BracketHighlightChannel, (event) => {
			const bracket_highlight_results = (event as Event<BracketHighlightResults>).payload;
			console.log('bracket_highlight_results', bracket_highlight_results);

			bracket_highlight_line_rectangle = null;
			bracket_highlight_box_rectangle_first = null;
			bracket_highlight_box_rectangle_last = null;

			if (bracket_highlight_results) {
				bracket_highlight_thickness = compute_bracket_highlight_thickness(
					bracket_highlight_results.lines
				);
				[bracket_highlight_box_rectangle_first, bracket_highlight_box_rectangle_last] =
					compute_bracket_highlight_box_rects(bracket_highlight_results.boxes);
				bracket_highlight_line_rectangle = compute_bracket_highlight_line_rect(
					bracket_highlight_results.lines
				);
				console.log('bracket_highlight_box_rectangle_first', bracket_highlight_box_rectangle_first);
			}
		});

		await listen('alert-selected', (event) => {
			const tauriEvent = event as Event<any>;
			highlightedRectangleMatchId = tauriEvent.payload;

			compute_rule_rects();
		});

		await listen('alert-deselected', (_) => {
			highlightedRectangleMatchId = null;

			compute_rule_rects();
		});
	};

	let rectangles: Array<[RuleName, MatchId, MatchRectangle]> = [];

	const compute_rule_rects = () => {
		if (rule_results_arr === null) {
			return;
		}
		if (outerPosition === null) {
			return;
		}

		let new_rectangles: Array<[RuleName, MatchId, MatchRectangle]> = [];

		for (const rule_results of rule_results_arr) {
			for (const rule_match of rule_results.results) {
				// push all rectangles of result into rectangles
				for (const rect of rule_match.rectangles) {
					let rect_adjusted: MatchRectangle = {
						origin: {
							x: rect.origin.x - outerPosition!.x,
							y: rect.origin.y - outerPosition!.y
						},
						size: {
							width: rect.size.width,
							height: rect.size.height
						}
					};

					// only add rectangles that are inside the window
					if (
						rects_overlap(rect_adjusted, {
							origin: { x: 0, y: 0 },
							size: { width: outerSize!.width, height: outerSize!.height }
						})
					) {
						new_rectangles.push([rule_results.rule, rule_match.id, rect_adjusted]);
					}
				}
			}
		}

		rectangles = new_rectangles;
	};

	const rects_overlap = (rect1: MatchRectangle, rect2: MatchRectangle) => {
		return (
			rect1.origin.x < rect2.origin.x + rect2.size.width &&
			rect1.origin.x + rect1.size.width > rect2.origin.x &&
			rect1.origin.y < rect2.origin.y + rect2.size.height &&
			rect1.origin.y + rect1.size.height > rect2.origin.y
		);
	};

	listenTauriEvents();
</script>

<div
	style="height: {height}px; border-style: solid; border-width: 1px; border-color: rgba(0,255,0,0.5);"
	class=" h-full w-full"
	id="overlay"
>
	<!-- {#each rectangles as rect}
		{#if rect[1] === highlightedRectangleMatchId}
			<div
				style="position: absolute; top: {Math.round(rect[2].origin.y)}px; left: {Math.round(
					rect[2].origin.x
				)}px; width: {Math.round(rect[2].size.width)}px;height: {Math.round(
					rect[2].size.height
				)}px; background-color: rgba(0,0,255,0.2); border-style: solid; border-bottom-width: 2px; border-color: rgba(0,0,255,0.4)"
			/>
		{:else}
			<div
				class="border-indigo-500"
				style="position: absolute; top: {Math.round(rect[2].origin.y)}px; left: {Math.round(
					rect[2].origin.x
				)}px; width: {Math.round(rect[2].size.width)}px;height: {Math.round(
					rect[2].size.height
				)}px; border-style: solid; border-bottom-width: 2px; border-color: rgba({rect[0] ===
				'SearchAndReplace'
					? '255,0,255,0.6'
					: '0,0,255,0.6'})"
			/>
		{/if}
	{/each} -->
	{#if bracket_highlight_line_rectangle !== null}
		<div
			style="position: absolute; top: {Math.round(
				bracket_highlight_line_rectangle.origin.y
			)}px; left: {Math.round(bracket_highlight_line_rectangle.origin.x)}px; width: {Math.round(
				bracket_highlight_line_rectangle.size.width
			)}px;height: {Math.round(
				bracket_highlight_line_rectangle.size.height
			)}px; border-style: solid; border-top-width: {bracket_highlight_thickness}px; border-color: rgba(122,122,122,0.5); border-left-width: {bracket_highlight_thickness}px;"
		/>
	{/if}
	{#if bracket_highlight_box_rectangle_first !== null}
		<div
			style="position: absolute; top: {Math.round(
				bracket_highlight_box_rectangle_first.origin.y
			)}px; left: {Math.round(
				bracket_highlight_box_rectangle_first.origin.x
			)}px; width: {Math.round(
				bracket_highlight_box_rectangle_first.size.width
			)}px;height: {Math.round(
				bracket_highlight_box_rectangle_first.size.height
			)}px; border-style: solid; border-width: {bracket_highlight_thickness}px; border-color: rgba(182,182,182,0.7);"
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
			)}px; border-style: solid; border-width: {bracket_highlight_thickness}px; border-color: rgba(182,182,182,0.7);"
		/>
	{/if}
</div>
