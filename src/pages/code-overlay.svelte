<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { LogicalFrame } from '../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';
	import BracketHighlight from '../components/code-overlay/bracket-highlight/bracket-highlight.svelte';
	import DocsAnnotations from '../components/code-overlay/docs-generation/docs-annotations.svelte';
	import type { EventViewport } from '../../src-tauri/bindings/macOS_specific/EventViewport';

	import { getContext } from 'svelte';
	import { convert_global_frame_to_local } from '../utils';

	const { setTheme } = getContext('theme');

	let code_document_rect: LogicalFrame | null = null; // Relative to viewport
	let annotation_section: LogicalFrame | null = null; // Relative to viewport

	let text_offset: number | null;
	$: annotations_opacity = get_annotations_opacity(code_document_rect, text_offset);

	function get_annotations_opacity(
		code_document_rect: LogicalFrame | null,
		text_offset: number | null
	): number {
		if (code_document_rect === null || text_offset === null || annotation_section === null) {
			return 1.0;
		} else {
			return code_document_rect.origin.x + text_offset <
				annotation_section.origin.x - 1 + annotation_section.size.width
				? 0.5
				: 1.0;
		}
	}
	const listenToViewportEvents = async () => {
		let ViewportChannel: ChannelList = 'EventViewport';
		await listen(ViewportChannel, (e) => {
			const { event, payload } = JSON.parse(e.payload as string) as EventViewport;

			switch (event) {
				case 'XcodeViewportUpdate':
					//console.log(new Date().toISOString(), 'XcodeViewportUpdate', payload);
					const { viewport_properties, code_document_frame_properties } = payload;

					if (code_document_frame_properties.text_offset) {
						text_offset = code_document_frame_properties.text_offset;
					}

					const code_document_rect_global = code_document_frame_properties.dimensions;
					code_document_rect = convert_global_frame_to_local(
						code_document_rect_global,
						viewport_properties.dimensions.origin
					);
					if (viewport_properties.annotation_section !== null) {
						annotation_section = convert_global_frame_to_local(
							viewport_properties.annotation_section,
							viewport_properties.dimensions.origin
						);
					}

					break;

				default:
					break;
			}
		});
	};

	const listenToWindowEvents = async () => {
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (e) => {
			const { event, payload } = JSON.parse(e.payload as string) as EventWindowControls;

			switch (event) {
				case 'DarkModeUpdate':
					setTheme(payload.dark_mode ? 'dark' : 'light');
					break;

				default:
					break;
			}
		});
	};

	listenToWindowEvents();
	listenToViewportEvents();
</script>

{#if code_document_rect}
	<div
		style="height: 100%; width: 100%;
			border-style: solid; border-width: 1px; border-color: rgba(0,255,0);"
		class="h-full w-full overflow-hidden relative"
	>
		<div
			style="
			height: {code_document_rect.size.height}px;
			width: {code_document_rect.size.width}px;
			top: {code_document_rect.origin.y}px; 
			left:{code_document_rect.origin.x}px; position: relative"
			class="h-full w-full overflow-hidden relative"
		>
			<BracketHighlight {code_document_rect} />
		</div>
		<div
			style="
			opacity: {annotations_opacity};
			height: {code_document_rect.size.height}px;
			width: {code_document_rect.size.width}px;
			top: {code_document_rect.origin.y}px; 
			left:0px; position: absolute"
			class="h-full w-full overflow-hidden absolute"
		>
			<DocsAnnotations />
		</div>
	</div>
{/if}
