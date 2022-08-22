<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventDocsGeneration } from '../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
	import type { CodeAnnotationMessage } from '../../src-tauri/bindings/features/docs_generation/CodeAnnotationMessage';
	import DocsAnnotations from '../components/code-overlay/docs-generation/docs-annotations.svelte';
	import BracketHighlight from '../components/code-overlay/bracket-highlight/bracket-highlight.svelte';
	import type { LogicalFrame } from '../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';
	import type { DarkModeUpdateMessage} from '../../src-tauri/bindings/dark_mode_update/DarkModeUpdateMessage'
	import { getContext } from 'svelte';
	const { setTheme } = getContext("theme");

	type CodeAnnotation = CodeAnnotationMessage;

	let code_overlay_rectangle: LogicalFrame | null = null;
	let code_annotation: CodeAnnotation | undefined = undefined;


	const listenDarkModeEvents = async () => {
		let DarkModeUpdateChannel: ChannelList = 'DarkModeUpdate'
		await listen(DarkModeUpdateChannel, (event) => {
			const payload = event.payload as DarkModeUpdateMessage;
			setTheme(payload.dark_mode ? 'dark' : 'light');	 //TODO: Type safety
		})
	}

	const listenWindowChannels = async () => {
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
	listenDarkModeEvents();

</script>

{#if code_overlay_rectangle}
	<div
		style="height: {code_overlay_rectangle.size
			.height}px; border-style: solid; border-width: 1px; border-color: rgba(0,255,0,0.0);"
		class=" h-full w-full"
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
