<script lang="ts">
	import { emit, listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { AnnotationEvent } from '../../../../src-tauri/bindings/annotations/AnnotationEvent';
	import type { AnnotationGroup } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationGroup';
	import type { LogicalFrame } from '../../../../src-tauri/bindings/geometry/LogicalFrame';
	import type { EventRuleExecutionState } from '../../../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import type { EventUserInteraction } from '../../../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import AnnotationIcon from './annotation-icon.svelte';
	import AnnotationLine from './annotation-line.svelte';
	import type { AnnotationKind } from '../../../../src-tauri/bindings/features/code_annotations/AnnotationKind';
	import { is_rectangle } from '../annotation_utils';
	import type { AiFeaturesStatusMessage } from '../../../../src-tauri/bindings/user_interaction/AiFeaturesStatusMessage';

	let annotation_icon: LogicalFrame | null;
	let annotation_codeblock: LogicalFrame | null;

	let annotation_group_id: string | null = null;
	let annotation_group_editor_window_uid: number | null = null;

	export let active_window_uid: number;
	export let annotation_section: LogicalFrame;
	export let code_document_rect: LogicalFrame;

	let is_hovering = false;
	let is_processing = false;

	let ai_mode_active = true;

	let processing_timeout = 15000; // ms

	const listen_to_node_annotation_events = async () => {
		let annotation_channel: ChannelList = 'AnnotationEvent';
		await listen(annotation_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as AnnotationEvent;
			switch (event_type) {
				case 'AddAnnotationGroup':
				case 'UpdateAnnotationGroup':
					let group = payload;

					if (group.feature === 'DocsGeneration') {
						annotation_group_editor_window_uid = group.editor_window_uid;
						annotation_group_id = group.id;

						let icon = get_icon_frame_from_group(group);
						if (icon) {
							annotation_icon = icon;
						}
						let codeblock = get_codeblock_frame_from_group(group);
						if (codeblock) {
							annotation_codeblock = codeblock;
						}
					}
					break;
				case 'RemoveAnnotationGroup':
					let group_id = payload as string;

					if (annotation_group_id === group_id) {
						annotation_group_editor_window_uid = null;
						annotation_group_id = null;

						annotation_icon = null;
						annotation_codeblock = null;
					}

					break;
				default:
					break;
			}
		});
	};

	const try_get_kind_as_rectangle = (
		group: AnnotationGroup,
		kind: AnnotationKind
	): LogicalFrame | undefined => {
		let result = Object.values(group.annotations).find((annotation) => annotation.kind === kind);
		if (!result || !result.shapes[0] || !is_rectangle(result.shapes[0])) {
			return;
		}
		return result.shapes[0].Rectangle;
	};

	const get_codeblock_frame_from_group = (group: AnnotationGroup): LogicalFrame | undefined => {
		let annotation_start = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'CodeblockFirstChar'
		);

		let annotation_end = Object.values(group.annotations).find(
			(annotation) => annotation.kind === 'CodeblockLastChar'
		);

		if (!annotation_start || !is_rectangle(annotation_start.shapes[0]) || !annotation_end) {
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

		return {
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

	const get_icon_frame_from_group = (group: AnnotationGroup): LogicalFrame | null => {
		let annotation_icon = try_get_kind_as_rectangle(group, 'CodeblockFirstChar');
		if (!annotation_icon) {
			return null;
		}

		return {
			origin: {
				x: annotation_section.origin.x,
				y: annotation_icon.origin.y
			},
			size: {
				width: annotation_section.size.width,
				height: annotation_icon.size.height
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

	const listenEventUserInteractions = async () => {
		let EventUserInteractionsChannel: ChannelList = 'EventUserInteractions';
		await listen(EventUserInteractionsChannel, (e) => {
			const { event, payload } = JSON.parse(e.payload as string) as EventUserInteraction;

			switch (event) {
				case 'AiFeaturesStatus':
					const { ai_features_active } = payload as AiFeaturesStatusMessage;
					ai_mode_active = ai_features_active;
					break;
				default:
					break;
			}
		});
	};
	listenEventUserInteractions();

	const annotation_click = async () => {
		is_hovering = false;

		// invoke click on annotation
		if (annotation_group_id && annotation_group_editor_window_uid) {
			const event: EventUserInteraction = {
				event: 'NodeAnnotationClicked',
				payload: {
					annotation_id: annotation_group_id,
					editor_window_uid: annotation_group_editor_window_uid
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

{#if ai_mode_active && annotation_group_editor_window_uid === active_window_uid}
	{#if annotation_icon}
		<div
			style="position: absolute;
			top: {round_value(annotation_icon.origin.y, 2)}px;
			width: {round_value(annotation_icon.size.width, 2)}px; 
			height: {round_value(annotation_icon.size.height, 2)}px;"
			on:mouseenter={() => (is_hovering = true)}
			on:mouseleave={() => (is_hovering = false)}
			on:mousedown={annotation_click}
			on:focus={null}
		>
			<AnnotationIcon {is_processing} {is_hovering} />
		</div>
	{/if}

	{#if annotation_codeblock}
		<div
			style="position: absolute; 
			top: {round_value(annotation_codeblock.origin.y, 2)}px; 
			width: {round_value(annotation_codeblock.size.width, 2)}px; 
			height: {round_value(annotation_codeblock.size.height, 2)}px;"
		>
			<AnnotationLine
				visible={is_hovering || is_processing}
				highlighted={is_hovering && !is_processing}
			/>
		</div>
	{/if}
{/if}
