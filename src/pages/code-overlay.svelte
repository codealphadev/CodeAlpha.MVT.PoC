<script lang="ts">
	import { listen, Event } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { RuleResults } from '../../src-tauri/bindings/rules/RuleResults';
	import type { EventDocsGeneration } from '../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
	import type { MatchRectangle } from '../../src-tauri/bindings/rules/utils/MatchRectangle';
	import type { RuleName } from '../../src-tauri/bindings/rules/RuleName';
	import type { CodeAnnotationMessage } from '../../src-tauri/bindings/features/docs_generation/CodeAnnotationMessage';
	import DocsAnnotations from '../components/code-ovelay/docs-generation/docs-annotations.svelte';
	import BracketHighlight from '../components/code-ovelay/bracket-highlight/bracket-highlight.svelte';

	type MatchId = string;

	let code_overlay_rectangle: MatchRectangle | null = null;

	let rule_results_arr: Array<RuleResults> | null = null;
	let highlightedRectangleMatchId = null;

	let docs_gen_annotations: CodeAnnotationMessage | null = null;

	const listenTauriEvents = async () => {
		await listen('event-compute-height', (event) => {
			const tauriEvent = event as Event<MatchRectangle>;
			const payload: MatchRectangle = tauriEvent.payload;
			code_overlay_rectangle = payload;

			compute_rule_rects();
		});

		let ResultsChannel: ChannelList = 'RuleResults';
		await listen(ResultsChannel, (event) => {
			const tauriEvent = event as Event<Array<RuleResults>>;
			rule_results_arr = tauriEvent.payload;
			compute_rule_rects();
		});

		// Listener for docs generation feature
		let DocsGenerationChannel: ChannelList = 'EventDocsGeneration';
		await listen(DocsGenerationChannel, (event) => {
			const docs_gen_event = JSON.parse(event.payload as string) as EventDocsGeneration;

			switch (docs_gen_event.event) {
				case 'CodeAnnotations':
					docs_gen_annotations = docs_gen_event.payload as unknown as CodeAnnotationMessage;
					break;
				default:
					break;
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
		if (code_overlay_rectangle.origin === null) {
			return;
		}

		let new_rectangles: Array<[RuleName, MatchId, MatchRectangle]> = [];

		for (const rule_results of rule_results_arr) {
			for (const rule_match of rule_results.results) {
				// push all rectangles of result into rectangles
				for (const rect of rule_match.rectangles) {
					let rect_adjusted: MatchRectangle = {
						origin: {
							x: rect.origin.x - code_overlay_rectangle.origin!.x,
							y: rect.origin.y - code_overlay_rectangle.origin!.y
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
							size: {
								width: code_overlay_rectangle.size!.width,
								height: code_overlay_rectangle.size!.height
							}
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

{#if code_overlay_rectangle}
	<div
		style="height: {code_overlay_rectangle.size
			.height}px; border-style: solid; border-width: 1px; border-color: rgba(0,255,0,0.0);"
		class=" h-full w-full"
		id="overlay"
	>
		<BracketHighlight {code_overlay_rectangle} />
		<DocsAnnotations
			annotation_msg={docs_gen_annotations}
			code_overlay_position={code_overlay_rectangle.origin}
		/>
	</div>
{/if}
