<script lang="ts">
	import { listen, Event } from '@tauri-apps/api/event';

	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { RuleResults } from '../../src-tauri/bindings/rules/RuleResults';
	import type { MatchRectangle } from '../../src-tauri/bindings/rules/utils/MatchRectangle';

	import type { LogicalSize } from '../../src-tauri/bindings/geometry/LogicalSize';
	import type { LogicalPosition } from '../../src-tauri/bindings/geometry/LogicalPosition';
	import type { RuleName } from '../../src-tauri/bindings/rules/RuleName';
	import type { RuleMatch } from '../../src-tauri/bindings/rules/RuleMatch';

	type MatchId = string;

	let height = 0;
	let outerSize: LogicalSize | null = null;
	let outerPosition: LogicalPosition | null = null;

	let rule_results_arr: Array<RuleResults> | null = null;
	let highlightedRectangleMatchId = null;

	const listenTauriEvents = async () => {
		await listen('event-compute-height', (event) => {
			const tauriEvent = event as Event<MatchRectangle>;
			const payload: MatchRectangle = tauriEvent.payload;

			outerSize = payload.size;
			outerPosition = payload.origin;
			height = payload.size.height;

			compute_rects();
		});

		let ResultsChannel: ChannelList = 'RuleResults';
		await listen(ResultsChannel, (event) => {
			const tauriEvent = event as Event<Array<RuleResults>>;
			rule_results_arr = tauriEvent.payload;

			compute_rects();
		});

		await listen('alert-selected', (event) => {
			const tauriEvent = event as Event<any>;
			highlightedRectangleMatchId = tauriEvent.payload;

			compute_rects();
		});

		await listen('alert-deselected', (_) => {
			highlightedRectangleMatchId = null;

			compute_rects();
		});
	};

	let rectangles: Array<[RuleName, MatchId, MatchRectangle]> = [];

	const compute_rects = () => {
		if (rule_results_arr === null) {
			return;
		}
		if (outerPosition === null) {
			return;
		}

		let new_rectangles: Array<[RuleName, MatchId, MatchRectangle]> = [];
		for (const rule_results of rule_results_arr) {
			for (const result of rule_results.results) {
				// push all rectangles of result into rectangles
				for (const rect of result.rectangles) {
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
						new_rectangles.push([rule_results.rule, result.id, rect_adjusted]);
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

<div style="height: {height}px; background-color: rgba(125,125,125,0.1);" class=" h-full w-full">
	{#each rectangles as rect}
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
					? '255,0,255,0.4'
					: '0,0,255,0.1'})"
			/>
		{/if}
	{/each}
</div>
