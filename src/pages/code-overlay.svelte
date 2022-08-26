<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { LogicalFrame } from '../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';
	import BracketHighlight from '../components/code-overlay/bracket-highlight/bracket-highlight.svelte';
	import DocsAnnotations from '../components/code-overlay/docs-generation/docs-annotations.svelte';

	import { getContext } from 'svelte';
	import {  convert_local_position_to_global, convert_global_position_to_local } from '../utils';
	
	const { setTheme } = getContext("theme");


	let code_viewport_global: LogicalFrame | null = null; // needs to be global (FOR NOW) because docs annotations and pair bracket etc. use global coordinates in the events.
	let code_document_local: LogicalFrame | null = null; // Relative to viewport

	const listenWindowChannels = async () => {
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (e) => {
			const {event, payload} = JSON.parse(e.payload as string) as EventWindowControls;

			switch (event) {
				case 'CodeOverlayDimensionsUpdate':
					if (payload.code_viewport_rect) {
						code_viewport_global = payload.code_viewport_rect;
					}
					if (code_viewport_global) {
						code_document_local = {
							size: payload.code_document_rect.size,
							origin: convert_global_position_to_local(payload.code_document_rect.origin, code_viewport_global.origin)
						}
					}
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

{#if code_viewport_global && code_document_local}
	<div
		style="height: {code_viewport_global.size
			.height}px;width: {code_viewport_global.size.width}px;
		 outline-style: solid; outline-width: 1px; outline-color: rgba(0,255,0,0.0);"
		class="h-full w-full overflow-hidden relative"
	>
		<div
			style="
			height: {code_document_local.size.height}px;
			width: {code_document_local.size.width}px;
			top: {code_document_local.origin.y }px; 
			left:{code_document_local.origin.x}px; position: relative"
			class="h-full w-full overflow-hidden relative"
		>
			<BracketHighlight code_document_height={code_document_local.size.height} code_document_global={convert_local_position_to_global(code_document_local.origin, code_viewport_global.origin)} />
			<DocsAnnotations
				code_document_global={convert_local_position_to_global(code_document_local.origin, code_viewport_global.origin)}
			/>
		</div>
	</div>
{/if}
