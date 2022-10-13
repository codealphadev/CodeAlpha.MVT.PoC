<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { AnnotationEvent } from '../../../../src-tauri/bindings/features/node_annotation/AnnotationEvent';
	import type { AnnotationGroup } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationGroup';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import { is_rectangle } from '../annotation_utils';
	import type { Annotation } from '../../../../src-tauri/bindings/features/code_annotations/Annotation';

	let annotation_extraction: LogicalFrame | null;
	let annotation_context: LogicalFrame | null;

	let annotation_group_id: string | null = null;
	let annotation_group_editor_window_uid: number | null = null;

	export let active_window_uid: number;
	export let annotation_section: LogicalFrame;
	export let code_document_rect: LogicalFrame;
	export let viewport_rect: LogicalFrame;

	const listen_to_node_annotation_events = async () => {
		let node_annotation_channel: ChannelList = 'NodeAnnotationEvent';
		await listen(node_annotation_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as AnnotationEvent;
			switch (event_type) {
				case 'AddAnnotationGroup':
				case 'UpdateAnnotationGroup':
					let group = payload;

					if (group.feature === 'ComplexityRefactoring') {
						console.log('ComplexityRefactoring', group);

						annotation_group_editor_window_uid = group.editor_window_uid;
						annotation_group_id = group.id;

						let extraction_codeblock = get_extract_block_frame_from_group(group);
						if (extraction_codeblock) {
							annotation_extraction = extraction_codeblock;
							console.log('extraction', annotation_extraction);
						}
						let context_codeblock = get_context_block_frame_from_group(group);
						if (context_codeblock) {
							annotation_context = context_codeblock;
							console.log('context', annotation_context);
						}
					}
					break;
				case 'RemoveAnnotationGroup':
					let group_id = payload as string;

					if (annotation_group_id === group_id) {
						annotation_group_editor_window_uid = null;
						annotation_group_id = null;

						annotation_extraction = null;
						annotation_context = null;
					}

					break;
				default:
					break;
			}
		});
	};

	const get_context_block_frame_from_group = (group: AnnotationGroup): LogicalFrame | undefined => {
		let annotation_start = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'CodeblockFirstChar'
		);

		let annotation_end = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'CodeblockLastChar'
		);

		if (!annotation_start || !annotation_end) {
			return;
		}

		return get_frame_from_annotation_pair(annotation_start, annotation_end);
	};

	const get_extract_block_frame_from_group = (group: AnnotationGroup): LogicalFrame | undefined => {
		let annotation_start = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'ExtractionStartChar'
		);

		let annotation_end = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'ExtractionEndChar'
		);

		if (!annotation_start || !annotation_end) {
			return;
		}

		return get_frame_from_annotation_pair(annotation_start, annotation_end);
	};

	const get_frame_from_annotation_pair = (
		start: Annotation,
		end: Annotation
	): LogicalFrame | undefined => {
		if (
			start.position_relative_to_viewport != 'Visible' &&
			end.position_relative_to_viewport != 'Visible' &&
			start.position_relative_to_viewport == end.position_relative_to_viewport
		) {
			return;
		}

		let codeblock_start_y = null;
		let codeblock_end_y = null;

		if (is_rectangle(start.shapes[0])) {
			codeblock_start_y = start.shapes[0].Rectangle.origin.y;
		} else {
			codeblock_start_y = 0;
		}

		if (is_rectangle(end.shapes[0])) {
			codeblock_end_y = end.shapes[0].Rectangle.origin.y + end.shapes[0].Rectangle.size.height;
		} else {
			codeblock_end_y = code_document_rect.size.height;
		}

		return {
			origin: {
				x: annotation_section.size.width,
				y: codeblock_start_y
			},
			size: {
				width: viewport_rect.size.width,
				height: codeblock_end_y - codeblock_start_y
			}
		};
	};

	listen_to_node_annotation_events();

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};
</script>

{#if annotation_group_editor_window_uid === active_window_uid}
	{#if annotation_extraction}
		<div
			style="position: absolute; 
			left: {round_value(annotation_extraction.origin.x, 2)}px; 
			top: {round_value(annotation_extraction.origin.y, 2)}px; 
			width: {round_value(annotation_extraction.size.width, 2)}px; 
			height: {round_value(annotation_extraction.size.height, 2)}px;
			background-image: linear-gradient(to right,  rgba(253, 88, 58, 1), rgba(253, 88, 58, 0));
			opacity: 0.2;"
		/>
	{/if}

	{#if annotation_context}
		<div
			style="position: absolute; 
			left: {round_value(annotation_context.origin.x, 2)}px; 
			top: {round_value(annotation_context.origin.y, 2)}px; 
			width: {round_value(annotation_context.size.width, 2)}px; 
			height: {round_value(annotation_context.size.height, 2)}px;
			clip-path: circle(50%);

			background-image: linear-gradient(to right,rgba(58, 136, 253, 1), rgba(58, 136, 253, 0);
			opacity: 0.1;"
		/>
		{#if annotation_extraction}
			<div
				style="position: absolute; 
			left: {round_value(annotation_extraction.origin.x, 2)}px; 
			top: {round_value(annotation_extraction.origin.y, 2)}px; 
			width: {round_value(annotation_extraction.size.width, 2)}px; 
			height: {round_value(annotation_extraction.size.height, 2)}px;
			background-image: linear-gradient(to right,rgba(253, 88, 58, 1), rgba(253, 88, 58, 0));
			opacity: 0.2;"
			/>
		{/if}
	{/if}
{/if}
