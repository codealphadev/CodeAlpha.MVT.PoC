<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { AnnotationEvent } from '../../../../src-tauri/bindings/features/node_annotation/AnnotationEvent';
	import type { AnnotationGroup } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationGroup';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import { is_point } from '../annotation_utils';
	import type { Annotation } from '../../../../src-tauri/bindings/features/code_annotations/Annotation';
	import type { EventUserInteraction } from '../../../../src-tauri/bindings/user_interaction/EventUserInteraction';

	let annotation_groups: AnnotationGroup[] = [];
	let selected_suggestion_id: string | null = null;

	export let active_window_uid: number;
	export let annotation_section_rect: LogicalFrame;
	export let code_document_rect: LogicalFrame;
	export let viewport_rect: LogicalFrame;

	// TODO
	let annotation_extraction: LogicalFrame | null;
	let annotation_context_pt1: LogicalFrame | null;
	let annotation_context_pt2: LogicalFrame | null;

	function derive_annotations(group: AnnotationGroup | undefined) {
		if (group === undefined) {
			annotation_extraction = null;
			annotation_context_pt1 = null;
			annotation_context_pt2 = null;
			return;
		}

		let extraction_codeblock = derive_extract_block_frame(group);
		if (extraction_codeblock) {
			annotation_extraction = extraction_codeblock;
		}

		let context_codeblock_pt1 = derive_context_block_frame_pt1(group);
		let context_codeblock_pt2 = derive_context_block_frame_pt2(group);
		if (context_codeblock_pt1) {
			annotation_context_pt1 = context_codeblock_pt1;
		}
		if (context_codeblock_pt2) {
			annotation_context_pt2 = context_codeblock_pt2;
		}
	}

	$: current_annotation_group = annotation_groups.find((grp) => grp.id === selected_suggestion_id);

	$: derive_annotations(current_annotation_group);

	const listen_to_suggestion_selection_events = async () => {
		let user_interaction_channel: ChannelList = 'EventUserInteractions';
		await listen(user_interaction_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(
				event.payload as string
			) as EventUserInteraction;
			console.log(payload, event_type);

			switch (event_type) {
				case 'UpdateSelectedSuggestion':
					const { id } = payload;
					selected_suggestion_id = id;
			}
		});
	};

	listen_to_suggestion_selection_events();

	const listen_to_annotation_events = async () => {
		let annotation_channel: ChannelList = 'AnnotationEvent';
		await listen(annotation_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as AnnotationEvent;
			console.log(payload, event_type);
			switch (event_type) {
				case 'AddAnnotationGroup':
				case 'UpdateAnnotationGroup':
					let group = payload;

					if (group.feature === 'ComplexityRefactoring') {
						const group_index = annotation_groups.findIndex((grp) => grp.id === group.id);

						if (group_index === -1) {
							annotation_groups.push(group);
						} else {
							annotation_groups[group_index] = group;
						}
						annotation_groups = annotation_groups;
					}
					break;
				case 'RemoveAnnotationGroup':
					let group_id = payload as string;
					annotation_groups = annotation_groups.filter((group) => group.id !== group_id);

					break;
				default:
					break;
			}
		});
	};

	const derive_context_block_frame_pt1 = (group: AnnotationGroup): LogicalFrame | undefined => {
		let annotation_start = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'CodeblockFirstChar'
		);

		let annotation_end = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'ExtractionStartChar'
		);

		if (!annotation_start || !annotation_end) {
			return;
		}

		return get_frame_from_annotation_pair(annotation_start, annotation_end);
	};

	const derive_context_block_frame_pt2 = (group: AnnotationGroup): LogicalFrame | undefined => {
		let annotation_start = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'ExtractionEndChar'
		);

		let annotation_end = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'CodeblockLastChar'
		);

		if (!annotation_start || !annotation_end) {
			return;
		}

		return get_frame_from_annotation_pair(annotation_start, annotation_end);
	};

	const derive_extract_block_frame = (group: AnnotationGroup): LogicalFrame | undefined => {
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

		const codeblock_start_y = is_point(start.shapes[0]) ? start.shapes[0].Point.y : 0;

		const codeblock_end_y = is_point(end.shapes[0])
			? end.shapes[0].Point.y
			: code_document_rect.size.height;

		return {
			origin: {
				x: annotation_section_rect.size.width,
				y: codeblock_start_y
			},
			size: {
				width: viewport_rect.size.width,
				height: codeblock_end_y - codeblock_start_y
			}
		};
	};

	listen_to_annotation_events();

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};
</script>

{#if current_annotation_group?.editor_window_uid === active_window_uid}
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

	{#if annotation_context_pt1}
		<div
			style="position: absolute; 
			left: {round_value(annotation_context_pt1.origin.x, 2)}px; 
			top: {round_value(annotation_context_pt1.origin.y, 2)}px; 
			width: {round_value(annotation_context_pt1.size.width, 2)}px; 
			height: {round_value(annotation_context_pt1.size.height, 2)}px;
			background-image: linear-gradient(to right,rgba(58, 136, 253, 1), rgba(58, 136, 253, 0);
			opacity: 0.1;"
		/>
	{/if}

	{#if annotation_context_pt2}
		<div
			style="position: absolute; 
			left: {round_value(annotation_context_pt2.origin.x, 2)}px; 
			top: {round_value(annotation_context_pt2.origin.y, 2)}px; 
			width: {round_value(annotation_context_pt2.size.width, 2)}px; 
			height: {round_value(annotation_context_pt2.size.height, 2)}px;
			background-image: linear-gradient(to right,rgba(58, 136, 253, 1), rgba(58, 136, 253, 0);
			opacity: 0.1;"
		/>
	{/if}
{/if}
