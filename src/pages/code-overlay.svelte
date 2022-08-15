<script lang="ts">
	import { listen, Event } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventDocsGeneration } from '../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
	import type { MatchRectangle } from '../../src-tauri/bindings/rules/utils/MatchRectangle';
	import type { CodeAnnotationMessage } from '../../src-tauri/bindings/features/docs_generation/CodeAnnotationMessage';
	import type { DocsGeneratedMessage } from '../../src-tauri/bindings/features/docs_generation/DocsGeneratedMessage';
	import DocsAnnotations from '../components/code-ovelay/docs-generation/docs-annotations.svelte';
import BracketHighlight from '../components/code-ovelay/bracket-highlight/bracket-highlight.svelte';

	let code_overlay_rectangle: MatchRectangle | null = null;
	let docs_gen_annotations: CodeAnnotationMessage | null = null;

	const listenTauriEvents = async () => {
		await listen('event-compute-height', (event) => {
			const tauriEvent = event as Event<MatchRectangle>;
			const payload: MatchRectangle = tauriEvent.payload;
			code_overlay_rectangle = payload;
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
