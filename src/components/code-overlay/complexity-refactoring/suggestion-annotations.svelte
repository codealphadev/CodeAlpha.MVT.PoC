<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { AnnotationEvent } from '../../../../src-tauri/bindings/annotations/AnnotationEvent';
	import type { AnnotationGroup } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationGroup';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import { is_point, round_value } from '../annotation_utils';
	import type { Annotation } from '../../../../src-tauri/bindings/features/code_annotations/Annotation';
	import type { EventUserInteraction } from '../../../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import type { AnnotationKind } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationKind';
	import { fade } from 'svelte/transition';
	import { main_window_active_store } from '../../../state';

	let main_window_active: boolean;

	main_window_active_store.subscribe((value: boolean) => {
		main_window_active = value;
	});

	const TRANSITION_DURATION = 100;

	let annotation_groups: AnnotationGroup[] = [];
	let selected_suggestion_id: string | null = null;

	export let active_window_uid: number;
	export let annotation_section_rect: LogicalFrame;
	export let code_document_rect: LogicalFrame;
	export let viewport_rect: LogicalFrame;

	let annotation_extraction: LogicalFrame | null;
	let annotation_context_before: LogicalFrame | null;
	let annotation_context_after: LogicalFrame | null;

	function derive_annotations(group: AnnotationGroup | undefined) {
		if (group === undefined) {
			annotation_extraction = null;
			annotation_context_before = null;
			annotation_context_after = null;
			return;
		}

		let extraction_codeblock = derive_block(group, 'ExtractionStartChar', 'ExtractionEndChar');
		if (extraction_codeblock) {
			annotation_extraction = extraction_codeblock;
		}

		let context_codeblock_before = derive_block(group, 'CodeblockFirstChar', 'ExtractionStartChar');
		if (context_codeblock_before) {
			annotation_context_before = context_codeblock_before;
		}

		let context_codeblock_after = derive_block(group, 'ExtractionEndChar', 'CodeblockLastChar');
		if (context_codeblock_after) {
			annotation_context_after = context_codeblock_after;
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
					console.log(payload, event_type);
					let group_id = payload as string;
					annotation_groups = annotation_groups.filter((group) => group.id !== group_id);

					break;
				default:
					break;
			}
		});
	};

	const derive_block = (
		group: AnnotationGroup,
		start_kind: AnnotationKind,
		end_kind: AnnotationKind
	): LogicalFrame | undefined => {
		let annotation_start = Object.values(group.annotations).find((a) => a.kind === start_kind);
		let annotation_end = Object.values(group.annotations).find((a) => a.kind === end_kind);

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
</script>

{#if main_window_active && current_annotation_group?.editor_window_uid === active_window_uid}
	{#if annotation_extraction}
		<div
			transition:fade={{
				duration: TRANSITION_DURATION
			}}
			style="position: absolute; 
			left: {round_value(annotation_extraction.origin.x, 2)}px; 
			top: {round_value(annotation_extraction.origin.y, 2)}px; 
			width: {round_value(annotation_extraction.size.width, 2)}px; 
			height: {round_value(annotation_extraction.size.height, 2)}px;
			background-image: linear-gradient(to right,  rgba(253, 88, 58, 0.2), rgba(253, 88, 58, 0));"
		/>
	{/if}

	{#if annotation_context_before}
		<div
			transition:fade={{
				duration: TRANSITION_DURATION
			}}
			style="position: absolute; 
			left: {round_value(annotation_context_before.origin.x, 2)}px; 
			top: {round_value(annotation_context_before.origin.y, 2)}px; 
			width: {round_value(annotation_context_before.size.width, 2)}px; 
			height: {round_value(annotation_context_before.size.height, 2)}px;
			background-image: linear-gradient(to right,rgba(58, 136, 253, 0.1), rgba(58, 136, 253, 0);"
		/>
	{/if}

	{#if annotation_context_after}
		<div
			transition:fade={{
				duration: TRANSITION_DURATION
			}}
			style="position: absolute; 
			left: {round_value(annotation_context_after.origin.x, 2)}px; 
			top: {round_value(annotation_context_after.origin.y, 2)}px; 
			width: {round_value(annotation_context_after.size.width, 2)}px; 
			height: {round_value(annotation_context_after.size.height, 2)}px;
			background-image: linear-gradient(to right,rgba(58, 136, 253, 0.1), rgba(58, 136, 253, 0);"
		/>
	{/if}
{/if}
