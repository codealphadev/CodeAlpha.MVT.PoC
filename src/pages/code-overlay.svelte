<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { LogicalFrame } from '../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';
	import BracketHighlight from '../components/code-overlay/bracket-highlight/bracket-highlight.svelte';
	import DocsAnnotations from '../components/code-overlay/docs-generation/docs-annotations.svelte';

	import type { ViewportPropertiesUpdateMessage } from '../../src-tauri/bindings/macOS_specific/xcode/ViewportPropertiesUpdateMessage';

	import { getContext } from 'svelte';
	import { convert_global_frame_to_local } from '../utils';
	
	const { setTheme } = getContext("theme");

	let code_document_rect: LogicalFrame | null = null; // Relative to viewport
	let text_offset: number | null;

	const listenToViewportEvents = async () => {
		let WindowControlsChannel: ChannelList = 'EventViewport';
		await listen(WindowControlsChannel, (e) => {
			const {event, payload} = JSON.parse(e.payload as string) as {
				event: 'ViewportPropertiesUpdateMessage',
				payload: ViewportPropertiesUpdateMessage
			};

			switch (event) {
				case 'ViewportPropertiesUpdateMessage':
					const {
						viewport_properties, code_document_frame_properties
					} = payload;

					const code_document_rect_global = code_document_frame_properties.dimensions;
					text_offset = code_document_frame_properties.text_offset;
					console.log(text_offset);
					code_document_rect = convert_global_frame_to_local(code_document_rect_global, viewport_properties.dimensions.origin);

					break;
				
				default:
					break;
			}
		});
	}

	const listenToWindowEvents = async () => {
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (e) => {
			const {event, payload} = JSON.parse(e.payload as string) as EventWindowControls;

			switch (event) {
				case 'DarkModeUpdate':
					setTheme(payload.dark_mode ? 'dark' : 'light');
					break;
				
				default:
					break;
			}
		});
	}

	listenToWindowEvents();
	listenToViewportEvents();	

</script>

{#if code_document_rect}
	<div
		style="height: 100%; width: 100%;
			outline-style: solid; outline-width: 1px; outline-color: rgba(0,255,0,0.0);"
		class="h-full w-full overflow-hidden relative"
	>
		<div
			style="
			height: {code_document_rect.size.height}px;
			width: {code_document_rect.size.width}px;
			top: {code_document_rect.origin.y }px; 
			left:{code_document_rect.origin.x}px; position: relative"
			class="h-full w-full overflow-hidden relative"
		>
			<BracketHighlight code_document_rect={code_document_rect} />
			<DocsAnnotations/>
		</div>
	</div>
{/if}