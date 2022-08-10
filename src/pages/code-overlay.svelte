<script lang="ts">
	import { listen, Event } from '@tauri-apps/api/event';

	import {
		compute_bracket_highlight_box_rects,
		compute_bracket_highlight_line_rect,
		BORDER_WIDTH
	} from './bracket_highlight';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { RuleResults } from '../../src-tauri/bindings/rules/RuleResults';
	import type { BracketHighlightResults } from '../../src-tauri/bindings/bracket_highlight/BracketHighlightResults';
	import type { EventDocsGeneration } from '../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';
	import type { MatchRectangle } from '../../src-tauri/bindings/rules/utils/MatchRectangle';

	import type { LogicalSize } from '../../src-tauri/bindings/geometry/LogicalSize';
	import type { LogicalPosition } from '../../src-tauri/bindings/geometry/LogicalPosition';
	import type { RuleName } from '../../src-tauri/bindings/rules/RuleName';
	import type { CodeAnnotationMessage } from '../../src-tauri/bindings/features/docs_generation/CodeAnnotationMessage';
	import type { DocsGeneratedMessage } from '../../src-tauri/bindings/features/docs_generation/DocsGeneratedMessage';
	import IconLogo from '../components/widget/icons/icon-logo.svelte';

	type MatchId = string;

	let height = 0;
	let outerSize: LogicalSize | null = null;
	let outerPosition: LogicalPosition | null = null;

	let rule_results_arr: Array<RuleResults> | null = null;
	let highlightedRectangleMatchId = null;
	let bracket_highlight_line_rectangle: MatchRectangle = null;
	let bracket_highlight_box_rectangle_first: MatchRectangle = null;
	let bracket_highlight_box_rectangle_last: MatchRectangle = null;
	let bottom_elbow_rectangle: MatchRectangle = null;

	let docs_gen_annotations: CodeAnnotationMessage | null = null;
	let docs_gen_message: DocsGeneratedMessage | null = null;

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

			bracket_highlight_line_rectangle = null;
			bracket_highlight_box_rectangle_first = null;
			bracket_highlight_box_rectangle_last = null;
			bottom_elbow_rectangle = null;

			if (bracket_highlight_results) {
				[bracket_highlight_box_rectangle_first, bracket_highlight_box_rectangle_last] =
					compute_bracket_highlight_box_rects(bracket_highlight_results.boxes, outerPosition);

				[bracket_highlight_line_rectangle, bottom_elbow_rectangle] =
					compute_bracket_highlight_line_rect(bracket_highlight_results, outerPosition, outerSize);
			}
		});

		// Listener for docs generation feature
		let DocsGenerationChannel: ChannelList = 'EventDocsGeneration';
		await listen(DocsGenerationChannel, (event) => {
			const docs_gen_event = JSON.parse(event.payload as string) as EventDocsGeneration;

			switch (docs_gen_event.event) {
				case 'CodeAnnotations':
					docs_gen_annotations = docs_gen_event.payload as unknown as CodeAnnotationMessage;
					break;
				case 'DocsGenerated':
					docs_gen_message = docs_gen_event.payload as unknown as DocsGeneratedMessage;
					break;
				default:
					break;
			}

			if (docs_gen_annotations !== null && docs_gen_annotations.annotation_icon !== null) {
				const unadjusted_origin = docs_gen_annotations.annotation_icon.origin;

				docs_gen_annotations.annotation_icon.origin = {
					x: unadjusted_origin.x - outerPosition!.x,
					y: unadjusted_origin.y - outerPosition!.y
				};
			}

			if (docs_gen_annotations !== null && docs_gen_annotations.annotation_codeblock !== null) {
				const unadjusted_origin = docs_gen_annotations.annotation_codeblock.origin;

				docs_gen_annotations.annotation_codeblock.origin = {
					x: unadjusted_origin.x - outerPosition!.x,
					y: unadjusted_origin.y - outerPosition!.y
				};
			}
		});

		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (event) => {
			const tracking_area_event = JSON.parse(event.payload as string) as EventWindowControls;
			console.log(tracking_area_event);
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
	{#if bracket_highlight_line_rectangle !== null}
		<div
			style="position: absolute; top: {Math.round(
				bracket_highlight_line_rectangle.origin.y
			)}px; left: {Math.round(bracket_highlight_line_rectangle.origin.x)}px; width: {Math.round(
				bracket_highlight_line_rectangle.size.width
			)}px;height: {Math.round(
				bracket_highlight_line_rectangle.size.height
			)}px; border-style: solid; border-top-width: {BORDER_WIDTH}px; border-color: rgba(122,122,122,0.5); border-left-width: {BORDER_WIDTH}px; border-right-width: 0; border-bottom-width: 0;"
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
			)}px; border-style: solid; border-width: {BORDER_WIDTH}px; border-color: rgba(182,182,182,0.7);"
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
			)}px; border-style: solid; border-width: {BORDER_WIDTH}px; border-color: rgba(182,182,182,0.7);"
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
			)}px; border-style: solid; border-bottom-width: {BORDER_WIDTH}px; border-color: rgba(182,182,182,0.7);"
		/>
	{/if}
	{#if docs_gen_annotations !== null}
		{#if docs_gen_annotations.annotation_icon !== null}
			<div
				style="position: absolute; top: {Math.round(
					docs_gen_annotations.annotation_icon.origin.y
				)}px; left: {Math.round(
					docs_gen_annotations.annotation_icon.origin.x
				)}px; width: {Math.round(
					docs_gen_annotations.annotation_icon.size.width
				)}px;height: {Math.round(
					docs_gen_annotations.annotation_icon.size.height
				)}px; border-style: solid; background-color: rgba(0, 255,0, 0.7);;"
			/>
		{/if}
	{/if}
</div>
