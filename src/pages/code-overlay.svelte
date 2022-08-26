<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { LogicalFrame } from '../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';
	import BracketHighlight from '../components/code-overlay/bracket-highlight/bracket-highlight.svelte';
	import DocsAnnotations from '../components/code-overlay/docs-generation/docs-annotations.svelte';

	import { getContext } from 'svelte';
	
	const { setTheme } = getContext("theme");


	let code_viewport_rectangle: LogicalFrame | null = null;
	let code_document_rectangle: LogicalFrame | null = null;


	const listenWindowChannels = async () => {
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (e) => {
			const {event, payload} = JSON.parse(e.payload as string) as EventWindowControls;

			switch (event) {
				case 'CodeOverlayDimensionsUpdate':
					if (payload.code_viewport_rect) {
						code_viewport_rectangle = payload.code_viewport_rect;
					}
					code_document_rectangle = payload.code_document_rect;
					break;
				case 'DarkModeUpdate':
					setTheme(payload.dark_mode ? 'dark' : 'light');
					break;
				
				default:
					break;
			}
		});
	}

	listenWindowChannels();

</script>

{#if code_viewport_rectangle && code_document_rectangle}
	<div
		style="height: {code_viewport_rectangle.size
			.height}px;width: {code_viewport_rectangle.size.width}px;
		 outline-style: solid; outline-width: 1px; outline-color: rgba(0,255,0,0.0);"
		class="h-full w-full overflow-hidden relative"
	>
		<div
			style="
			height: {code_document_rectangle.size.height}px;
			width: {code_document_rectangle.size.width}px;
			top: {code_document_rectangle.origin.y - code_viewport_rectangle.origin.y}px; 
			left:{code_document_rectangle.origin.x - code_viewport_rectangle.origin.x}px; position: relative"
			class="h-full w-full overflow-hidden relative"
		>
			<BracketHighlight code_document_rectangle={code_document_rectangle} />
			<DocsAnnotations
				code_document_origin={code_document_rectangle.origin}
			/>
		</div>
	</div>
{/if}
