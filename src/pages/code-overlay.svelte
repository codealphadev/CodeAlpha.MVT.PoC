<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { LogicalFrame } from '../../src-tauri/bindings/geometry/LogicalFrame';
	import BracketHighlight from '../components/code-overlay/bracket-highlight/bracket-highlight.svelte';
	import DocsAnnotations from '../components/code-overlay/docs-generation/node-annotations.svelte';
	import type { EventViewport } from '../../src-tauri/bindings/macOS_specific/EventViewport';

	import { convert_global_frame_to_local } from '../utils';
	import type { AnnotationEvent } from '../../src-tauri/bindings/features/node_annotation/AnnotationEvent';

	let code_document_rect: LogicalFrame | null = null; // Relative to viewport
	let annotation_section: LogicalFrame | null = null; // Relative to viewport
	let active_window_uid: number | null = null;

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
					const { viewport_properties, code_document_frame_properties } = payload;

					active_window_uid = viewport_properties.window_uid;

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

	listenToViewportEvents();

	let annotation_group_id: string | null = null;

	const listen_to_node_annotation_events = async () => {
		let node_annotation_channel: ChannelList = 'NodeAnnotationEvent';
		await listen(node_annotation_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as AnnotationEvent;
			switch (event_type) {
				case 'AddAnnotationGroup':
				case 'UpdateAnnotationGroup':
					let group = payload;

					if (group.feature === 'ComplexityRefactoring') {
						console.log('ComplexityRefactoring', event_type, group);
						annotation_group_id = group.id;
					}
					break;
				case 'RemoveAnnotationGroup':
					let group_id = payload as string;

					if (annotation_group_id === group_id) {
						console.log('ComplexityRefactoring', event_type, group_id);
					}

					break;
				default:
					break;
			}
		});
	};
	listen_to_node_annotation_events();
</script>

{#if code_document_rect && annotation_section && active_window_uid}
	<div
		style="height: 100%; width: 100%;
			border-style: solid; border-width: 0px; border-color: rgba(255,20,255);"
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
			<BracketHighlight {code_document_rect} {annotation_section} {active_window_uid} />
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
			<DocsAnnotations {code_document_rect} {annotation_section} {active_window_uid} />
		</div>
	</div>
{/if}
