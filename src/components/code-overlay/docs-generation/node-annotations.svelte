<script lang="ts">
	import { emit, listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { NodeAnnotationEvent } from '../../../../src-tauri/bindings/features/node_annotation/NodeAnnotationEvent';
	import type { UpdateNodeAnnotationMessage } from '../../../../src-tauri/bindings/features/node_annotation/UpdateNodeAnnotationMessage';
	import type { EventRuleExecutionState } from '../../../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import type { EventUserInteraction } from '../../../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import AnnotationIcon from './annotation-icon.svelte';
	import AnnotationLine from './annotation-line.svelte';

	type NodeAnnotation = UpdateNodeAnnotationMessage;
	let annotation: NodeAnnotation | undefined;

	export let active_window_uid: number;

	let is_hovered = false;
	let is_processing = false;

	let processing_timeout = 15000; // ms

	const listen_to_node_annotation_events = async () => {
		let node_annotation_channel: ChannelList = 'NodeAnnotationEvent';
		await listen(node_annotation_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(
				event.payload as string
			) as NodeAnnotationEvent;
			switch (event_type) {
				case 'UpdateNodeAnnotation':
					annotation = payload;
					break;
				case 'RemoveNodeAnnotation':
					if (annotation?.id === payload.id) {
						annotation = undefined;
					}
					break;
				default:
					break;
			}
		});
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
		if (annotation) {
			const event: EventUserInteraction = {
				event: 'NodeAnnotationClicked',
				payload: {
					annotation_id: annotation.id,
					window_uid: annotation.window_uid
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

{#if annotation && annotation.window_uid == active_window_uid}
	{#if annotation.annotation_icon !== null}
		<div
			style="position: absolute; 
			top: {round_value(annotation.annotation_icon.origin.y, 2)}px; 
			width: {round_value(annotation.annotation_icon.size.width, 2)}px; 
			height: {round_value(annotation.annotation_icon.size.height, 2)}px;"
			on:mouseenter={() => (is_hovered = true)}
			on:mouseleave={() => (is_hovered = false)}
			on:click={annotation_click}
			on:focus={null}
		>
			<AnnotationIcon {is_hovered} {is_processing} />
		</div>
	{/if}

	{#if annotation.annotation_codeblock !== null}
		<div
			style="position: absolute; 
			top: {round_value(annotation.annotation_codeblock.origin.y, 2)}px; 
			width: {round_value(annotation.annotation_codeblock.size.width, 2)}px; 
			height: {round_value(annotation.annotation_codeblock.size.height, 2)}px;"
		>
			<AnnotationLine
				visible={is_hovered || is_processing}
				highlighted={is_hovered && !is_processing}
			/>
		</div>
	{/if}
{/if}
