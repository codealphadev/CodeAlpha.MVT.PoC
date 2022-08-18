<script lang="ts">
	import { listen, Event } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventDocsGeneration } from '../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
	import type { MatchRectangle } from '../../src-tauri/bindings/rules/utils/MatchRectangle';
	import type { RuleName } from '../../src-tauri/bindings/rules/RuleName';
	import type { CodeAnnotationMessage } from '../../src-tauri/bindings/features/docs_generation/CodeAnnotationMessage';
	import DocsAnnotations from '../components/code-overlay/docs-generation/docs-annotations.svelte';
	import BracketHighlight from '../components/code-overlay/bracket-highlight/bracket-highlight.svelte';
	import type { LogicalFrame } from '../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';

	let code_overlay_rectangle: LogicalFrame | null = null;
	let docs_gen_annotations: CodeAnnotationMessage | null = null;

	const listenTauriEvents = async () => {
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (event) => {
			const event_window_controls = JSON.parse(event.payload as string) as EventWindowControls;

			switch (event_window_controls.event) {
				case 'CodeOverlayDimensionsUpdate':
					code_overlay_rectangle = event_window_controls.payload as unknown as LogicalFrame;
					break;
				default:
					break;
			}
		});

		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (event) => {
			const event_window_controls = JSON.parse(event.payload as string) as EventWindowControls;

			switch (event_window_controls.event) {
				case 'CodeOverlayDimensionsUpdate':
					const updated_code_overlay_dim = event_window_controls.payload as unknown as LogicalFrame;
					outerSize = updated_code_overlay_dim.size;
					outerPosition = updated_code_overlay_dim.origin;
					height = updated_code_overlay_dim.size.height;

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
