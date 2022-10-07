<script lang="ts">
	import { emit, listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { AnnotationEvent } from '../../../../src-tauri/bindings/features/node_annotation/AnnotationEvent';
	import type { AnnotationGroup } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationGroup';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventRuleExecutionState } from '../../../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import type { EventUserInteraction } from '../../../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import AnnotationIcon from './annotation-icon.svelte';
	import AnnotationLine from './annotation-line.svelte';
	import type { AnnotationShape } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationShape';

	let annotation_group: AnnotationGroup | undefined;
	let annotation_icon: LogicalFrame | null;
	let annotation_codeblock: LogicalFrame | null;

	export let active_window_uid: number;
	export let annotation_section: LogicalFrame;
	export let code_document_rect: LogicalFrame;

	let is_hovered = false;
	let is_processing = false;

	let processing_timeout = 15000; // ms

	const listen_to_node_annotation_events = async () => {
		let node_annotation_channel: ChannelList = 'NodeAnnotationEvent';
		await listen(node_annotation_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as AnnotationEvent;
			switch (event_type) {
				case 'AddAnnotationGroup':
					let added_group = payload as AnnotationGroup;

					if (added_group.feature === 'DocsGeneration') {
						annotation_group = added_group;

						derive_annotation_icon();
						derive_annotation_codeblock();
					}

					break;
				case 'UpdateAnnotationGroup':
					let updated_group = payload as AnnotationGroup;

					if (updated_group.feature === 'DocsGeneration') {
						annotation_group = updated_group;

						derive_annotation_icon();
						derive_annotation_codeblock();
					}

					break;
				case 'RemoveAnnotationGroup':
					let group_id = payload as string;

					if (annotation_group?.id === group_id) {
						annotation_group = undefined;
						annotation_icon = null;
						annotation_codeblock = null;
					}

					break;
				default:
					break;
			}
		});
	};

	function is_rectangle(shape: AnnotationShape): shape is { Rectangle: LogicalFrame } {
		return shape.hasOwnProperty('Rectangle');
	}

	const derive_annotation_codeblock = () => {
		annotation_codeblock = null;

		if (!annotation_group) {
			return;
		}

		let annotation_start = annotation_group.annotations.find(
			(annotation) => annotation.kind === 'CodeblockFirstChar'
		);

		let annotation_end = annotation_group.annotations.find(
			(annotation) => annotation.kind === 'CodeblockLastChar'
		);

		if (!annotation_start || !annotation_end) {
			return;
		}

		if (annotation_start.shapes[0] === undefined) {
			return;
		}

		if (!is_rectangle(annotation_start.shapes[0])) {
			return;
		}

		let codeblock_start_y =
			annotation_start.shapes[0].Rectangle.origin.y +
			annotation_start.shapes[0].Rectangle.size.height;

		let codeblock_end_y = codeblock_start_y;
		if (annotation_end.position_relative_to_viewport != 'Visible') {
			codeblock_end_y = code_document_rect.size.height;
		} else {
			if (annotation_end.shapes[0] === undefined || !is_rectangle(annotation_end.shapes[0])) {
				return;
			} else {
				codeblock_end_y =
					annotation_end.shapes[0].Rectangle.origin.y +
					annotation_end.shapes[0].Rectangle.size.height;
			}
		}

		annotation_codeblock = {
			origin: {
				x: annotation_section.origin.x,
				y: codeblock_start_y
			},
			size: {
				width: annotation_section.size.width,
				height: codeblock_end_y - codeblock_start_y
			}
		};
	};

	const derive_annotation_icon = () => {
		annotation_icon = null;

		if (!annotation_group) {
			return;
		}

		let annotation = annotation_group.annotations.find(
			(annotation) => annotation.kind === 'CodeblockFirstChar'
		);

		if (!annotation) {
			return;
		}

		if (annotation.shapes[0] === undefined) {
			return;
		}

		if (!is_rectangle(annotation.shapes[0])) {
			return;
		}

		annotation_icon = {
			origin: {
				x: annotation_section.origin.x,
				y: annotation.shapes[0].Rectangle.origin.y
			},
			size: {
				width: annotation_section.size.width,
				height: annotation.shapes[0].Rectangle.size.height
			}
		};
	};

	listen_to_node_annotation_events();

	const listen_to_docs_generation_events = async () => {
		// Listen for rule execution events to determine if the processing icon should be displayed
		await listen('EventRuleExecutionState' as ChannelList, (event) => {
			const ruleExecutionState = JSON.parse(event.payload as string) as EventRuleExecutionState;

			switch (ruleExecutionState.event) {
				case 'NodeExplanationStarted':
					is_processing = true;
					setTimeout(async () => {
						is_processing = false;
					}, processing_timeout);
					break;
				case 'NodeExplanationFetched':
					is_processing = false;
					break;
				case 'NodeExplanationFailed':
					is_processing = false;
					break;
				default:
					break;
			}
		});
	};
	listen_to_docs_generation_events();

	const annotation_click = async () => {
		is_hovered = false;

		// invoke click on annotation
		if (annotation_group) {
			const event: EventUserInteraction = {
				event: 'NodeAnnotationClicked',
				payload: {
					annotation_id: annotation_group.id,
					editor_window_uid: annotation_group.editor_window_uid
				}
			};
			const channel: ChannelList = 'EventUserInteractions';
			await emit(channel, event);
		}
	};

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};
</script>

{#if annotation_group && annotation_group.editor_window_uid == active_window_uid}
	{#if annotation_icon !== null}
		<div
			style="position: absolute; 
			top: {round_value(annotation_icon.origin.y, 2)}px; 
			width: {round_value(annotation_icon.size.width, 2)}px; 
			height: {round_value(annotation_icon.size.height, 2)}px;"
			on:mouseenter={() => (is_hovered = true)}
			on:mouseleave={() => (is_hovered = false)}
			on:mousedown={annotation_click}
			on:focus={null}
		>
			<AnnotationIcon {is_hovered} {is_processing} />
		</div>
	{/if}

	{#if annotation_codeblock !== null}
		<div
			style="position: absolute; 
			top: {round_value(annotation_codeblock.origin.y, 2)}px; 
			width: {round_value(annotation_codeblock.size.width, 2)}px; 
			height: {round_value(annotation_codeblock.size.height, 2)}px;"
		>
			<AnnotationLine
				visible={is_hovered || is_processing}
				highlighted={is_hovered && !is_processing}
			/>
		</div>
	{/if}
{/if}
