<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { EventDocsGeneration } from '../../../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
	import type { UpdateCodeAnnotationMessage } from '../../../../src-tauri/bindings/features/docs_generation/UpdateCodeAnnotationMessage';
	import type { LogicalPosition } from '../../../../src-tauri/bindings/geometry/LogicalPosition';
	import type { EventRuleExecutionState } from '../../../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import type { EventWindowControls } from '../../../../src-tauri/bindings/window_controls/EventWindowControls';
	import type { TrackingAreaClickedMessage } from '../../../../src-tauri/bindings/window_controls/TrackingAreaClickedMessage';
	import type { TrackingAreaEnteredMessage } from '../../../../src-tauri/bindings/window_controls/TrackingAreaEnteredMessage';
	import type { TrackingAreaExitedMessage } from '../../../../src-tauri/bindings/window_controls/TrackingAreaExitedMessage';
	import { convert_global_frame_to_local_frame } from '../../../utils';
	import AnnotationIcon from './annotation-icon.svelte';
	import AnnotationLine from './annotation-line.svelte';
	
	
	export let code_document_origin: LogicalPosition;
	
	type CodeAnnotation = UpdateCodeAnnotationMessage;
	let annotation: CodeAnnotation | undefined;

	let is_hovered = false;
	let is_processing = false;

	let processing_timeout = 15000; // ms


	function mapUpdateCodeAnnotationPayloadToCodeAnnotation(payload: UpdateCodeAnnotationMessage): CodeAnnotation | undefined {
		return {
			id: payload.id,
			annotation_icon: payload.annotation_icon ? convert_global_frame_to_local_frame(payload.annotation_icon, code_document_origin) : null,
			annotation_codeblock: payload.annotation_codeblock ? convert_global_frame_to_local_frame(payload.annotation_codeblock, code_document_origin) : null
		}
	
	}
	const listenToCodeAnnotationEvents = async () => {
		// Listener for docs generation feature
		let DocsGenerationChannel: ChannelList = 'EventDocsGeneration';
		await listen(DocsGenerationChannel, (event) => {
			const {payload, event: event_type} = JSON.parse(event.payload as string) as EventDocsGeneration;
			switch (event_type) {
				case 'UpdateCodeAnnotation':
					annotation = mapUpdateCodeAnnotationPayloadToCodeAnnotation(payload);
					break;
				case 'RemoveCodeAnnotation':
					if (annotation?.id === payload.id) {
						annotation = undefined;
					}
					break;
				default:
					break;
			}
		});
	};

	listenToCodeAnnotationEvents();

	const listenToDocsGenerationEvents = async () => {
		// Listen for rule execution events to determine if the processing icon should be displayed
		await listen('EventRuleExecutionState' as ChannelList, (event) => {
			const ruleExecutionState = JSON.parse(event.payload as string) as EventRuleExecutionState;

			switch (ruleExecutionState.event) {
				case 'DocsGenerationStarted':
					is_processing = true;
					setTimeout(async () => {
						is_processing = false;
					}, processing_timeout);
					break;
				case 'DocsGenerationFinished':
					is_processing = false;
					break;
				default:
					break;
			}
		});
	};
	listenToDocsGenerationEvents();

	const listenToTrackingAreaEvents = async () => {
		// Listen for click & hover events on the tracking area
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (event) => {
			const tracking_area_event = JSON.parse(event.payload as string) as EventWindowControls;
			switch (tracking_area_event.event) {
				case 'TrackingAreaClicked':
					let clicked_msg = tracking_area_event.payload as unknown as TrackingAreaClickedMessage;
					if (clicked_msg.id === annotation?.id) {
						is_hovered = true;
					}
					break;
				case 'TrackingAreaEntered':
					let entered_msg = tracking_area_event.payload as unknown as TrackingAreaEnteredMessage;
					if (entered_msg.id === annotation?.id) {
						is_hovered = true;
					}
					break;
				case 'TrackingAreaExited':
					let exited_msg = tracking_area_event.payload as unknown as TrackingAreaExitedMessage;
					if (exited_msg.id === annotation?.id) {
						is_hovered = false;
					}
					break;
				default:
					break;
			}
		});
	};

	listenToTrackingAreaEvents();

	const round_value = (value: number, precision: number): number => {
		const factor = Math.pow(10, precision || 0);
		return Math.round(value * factor) / factor;
	};
</script>
{#if annotation}
	{#if annotation.annotation_icon !== null}
		<div
			style="position: absolute; 
			top: {round_value(annotation.annotation_icon.origin.y, 2)}px; 
			left: {round_value(annotation.annotation_icon.origin.x, 2)}px; 
			width: {round_value(annotation.annotation_icon.size.width, 2)}px; 
			height: {round_value(annotation.annotation_icon.size.height, 2)}px;"
		>
			<AnnotationIcon {is_hovered} {is_processing} />
		</div>
	{/if}

	{#if annotation.annotation_codeblock !== null}
		<div
			style="position: absolute; 
			top: {round_value(annotation.annotation_codeblock.origin.y, 2)}px; 
			left: {round_value(annotation.annotation_codeblock.origin.x, 2)}px; 
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