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
	import { convert_global_frame_to_local_frame } from '../utils';
	
	const { setTheme } = getContext("theme");

	type CodeAnnotation = UpdateCodeAnnotationMessage;

	let code_viewport_rectangle: LogicalFrame | null = null;
	let code_document_rectangle: LogicalFrame | null = null;

	let code_annotation: CodeAnnotation | undefined = undefined;


	const listenWindowChannels = async () => {
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (e) => {
			const {event, payload} = JSON.parse(e.payload as string) as EventWindowControls;

			switch (event) {
				case 'CodeOverlayDimensionsUpdate':
					code_viewport_rectangle = payload.code_viewport_rect;
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

	function mapUpdateCodeAnnotationPayloadToCodeAnnotation(payload: UpdateCodeAnnotationMessage): CodeAnnotation | undefined {
		if (!code_document_rectangle) {
			return undefined;
		}
		return {
			id: payload.id,
			annotation_icon: payload.annotation_icon ? convert_global_frame_to_local_frame(payload.annotation_icon, code_document_rectangle) : null,
			annotation_codeblock: payload.annotation_codeblock ? convert_global_frame_to_local_frame(payload.annotation_codeblock, code_document_rectangle) : null
		}
	
	}
	const listenCodeAnnotations = async () => {
		// Listener for docs generation feature
		let DocsGenerationChannel: ChannelList = 'EventDocsGeneration';
		await listen(DocsGenerationChannel, (event) => {
			const {payload, event: event_type} = JSON.parse(event.payload as string) as EventDocsGeneration;
			switch (event_type) {
				case 'UpdateCodeAnnotation':
					code_annotation = mapUpdateCodeAnnotationPayloadToCodeAnnotation(payload);
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
			{#if code_annotation}
				<DocsAnnotations
					annotation={code_annotation}
					code_overlay_position={code_document_rectangle.origin}
				/>
			{/if}
		</div>
	</div>
{/if}
