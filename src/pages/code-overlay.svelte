<script lang="ts">
	import { listen, Event } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventDocsGeneration } from '../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
	import type { CodeAnnotationMessage } from '../../src-tauri/bindings/features/docs_generation/CodeAnnotationMessage';
	import DocsAnnotations from '../components/code-overlay/docs-generation/docs-annotations.svelte';
	import BracketHighlight from '../components/code-overlay/bracket-highlight/bracket-highlight.svelte';
	import type { LogicalFrame } from '../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';
	import type { DarkModeUpdateMessage} from '../../src-tauri/bindings/dark_mode_update/DarkModeUpdateMessage'
	type CodeAnnotation = CodeAnnotationMessage;

	let code_overlay_rectangle: LogicalFrame | null = null;
	let code_annotations: Array<CodeAnnotation> = [];


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

		let DarkModeUpdateChannel: ChannelList = 'DarkModeUpdate'
		await listen(DarkModeUpdateChannel,(event) => {
			const message = JSON.parse(event.payload as string) as DarkModeUpdateMessage;
			console.log(message)		})

		// Listener for docs generation feature
		let DocsGenerationChannel: ChannelList = 'EventDocsGeneration';
		await listen(DocsGenerationChannel, (event) => {
			const {payload, event: event_type} = JSON.parse(event.payload as string) as EventDocsGeneration;
			switch (event_type) {
				case 'UpdateCodeAnnotation':
					const existing_item_index = code_annotations.findIndex((annotation) => annotation.id === payload.id);
					if (existing_item_index !== -1) {
						code_annotations[existing_item_index] = payload;
					} else {
						code_annotations.push(payload);
					}
					code_annotations = code_annotations;
					break;
				case 'RemoveCodeAnnotation':
					code_annotations = code_annotations.filter(annotation => annotation.id !== payload.id)
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
		{#each code_annotations as annotation}
			<DocsAnnotations
				annotation={annotation}
				code_overlay_position={code_overlay_rectangle.origin}
			/>
		{/each}
	</div>
{/if}
