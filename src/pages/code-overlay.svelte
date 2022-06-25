<script lang="ts">
	import { listen } from '@tauri-apps/api/event';

	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { RuleResults } from '../../src-tauri/bindings/rules/RuleResults';
	import type { MatchRectangle } from '../../src-tauri/bindings/rules/utils/MatchRectangle';

	import { onMount } from 'svelte';
	import type { LogicalSize } from '../../src-tauri/bindings/geometry/LogicalSize';
	import type { LogicalPosition } from '../../src-tauri/bindings/geometry/LogicalPosition';

	let rn;
	let visible = false;
	onMount(() => {
		setTimeout(() => {
			visible = true; // or rn.visible = true or rn.show()
		}, 1000);
	});

	let height = 0;
	let outerSize: LogicalSize | null = null;
	let outerPosition: LogicalPosition | null = null;

	let results: RuleResults | null = null;

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
			const tauriEvent = event as Event<RuleResults>;
			results = tauriEvent.payload;

			compute_rects();
		});
	};

	let rectangles: Array<MatchRectangle> = [];

	const compute_rects = () => {
		if (results === null) {
			return;
		}

		let new_rectangles: Array<MatchRectangle> = [];
		for (const result of results.results) {
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
					new_rectangles.push(rect_adjusted);
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

<div style="height: {height}px;" class=" h-full w-full">
	{#each rectangles as rect}
		<div
			style="position: absolute; top: {Math.round(rect.origin.y)}px; left: {Math.round(
				rect.origin.x
			)}px; width: {Math.round(rect.size.width)}px;height: {Math.round(
				rect.size.height
			)}px; background-color: rgba(255,0,0,0.2)"
		/>
	{/each}
</div>
