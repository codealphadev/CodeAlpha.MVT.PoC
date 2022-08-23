<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventDocsGeneration } from '../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
	import type { LogicalFrame } from '../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';
	import BracketHighlight from '../components/code-overlay/bracket-highlight/bracket-highlight.svelte';
	import DocsAnnotations from '../components/code-overlay/docs-generation/docs-annotations.svelte';

	import { getContext } from 'svelte';
	import type { UpdateCodeAnnotationMessage } from '../../src-tauri/bindings/features/docs_generation/UpdateCodeAnnotationMessage';
	
	const { setTheme } = getContext("theme");

	type CodeAnnotation = UpdateCodeAnnotationMessage;

	let code_overlay_rectangle: LogicalFrame | null = null;
	let code_annotation: CodeAnnotation | undefined = undefined;

	const listenWindowChannels = async () => {
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (e) => {
			const {event, payload} = JSON.parse(e.payload as string) as EventWindowControls;

			switch (event) {
				case 'CodeOverlayDimensionsUpdate':
					code_overlay_rectangle = payload;
					break;
				case 'DarkModeUpdate':
					setTheme(payload.dark_mode ? 'dark' : 'light');
					break;
				
				default:
					break;
			}
		});
	}

	const listenCodeAnnotations = async () => {
		// Listener for docs generation feature
		let DocsGenerationChannel: ChannelList = 'EventDocsGeneration';
		await listen(DocsGenerationChannel, (event) => {
			const {payload, event: event_type} = JSON.parse(event.payload as string) as EventDocsGeneration;
			switch (event_type) {
				case 'UpdateCodeAnnotation':
					code_annotation = payload;
					break;
				case 'RemoveCodeAnnotation':
					if (code_annotation?.id === payload.id) {
						code_annotation = undefined;
					}
					break;
				default:
					break;
			}
		});
	};

	listenCodeAnnotations();
	listenWindowChannels();
</script>

{#if code_overlay_rectangle}
	<div
		style="height: {code_overlay_rectangle.size
			.height}px; outline-style: solid; outline-width: 1px; outline-color: rgba(0,255,0,0.0);"
		class="h-full w-full overflow-hidden relative"
		id="overlay"
	>
		<BracketHighlight {code_overlay_rectangle} />
		{#if code_annotation}
			<DocsAnnotations
				annotation={code_annotation}
				code_overlay_position={code_overlay_rectangle.origin}
			/>
		{/if}
	</div>
{/if}
