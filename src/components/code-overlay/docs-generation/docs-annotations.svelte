<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { UpdateCodeAnnotationMessage } from '../../../../src-tauri/bindings/features/docs_generation/UpdateCodeAnnotationMessage';
	import type { LogicalPosition } from '../../../../src-tauri/bindings/geometry/LogicalPosition';
	import type { EventRuleExecutionState } from '../../../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import type { EventWindowControls } from '../../../../src-tauri/bindings/window_controls/EventWindowControls';
	import type { TrackingAreaClickedMessage } from '../../../../src-tauri/bindings/window_controls/TrackingAreaClickedMessage';
	import type { TrackingAreaEnteredMessage } from '../../../../src-tauri/bindings/window_controls/TrackingAreaEnteredMessage';
	import type { TrackingAreaExitedMessage } from '../../../../src-tauri/bindings/window_controls/TrackingAreaExitedMessage';
	import AnnotationIcon from './annotation-icon.svelte';
	import AnnotationLine from './annotation-line.svelte';

	export let annotation: UpdateCodeAnnotationMessage;
	export let code_overlay_position: LogicalPosition | null;

	let is_hovered = false;
	let is_processing = false;

	let processing_timeout = 15000; // ms

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

	const listenToTrackingAreaEvents = async () => {
		// Listen for click & hover events on the tracking area
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (event) => {
			const tracking_area_event = JSON.parse(event.payload as string) as EventWindowControls;
			switch (tracking_area_event.event) {
				case 'TrackingAreaClicked':
					let clicked_msg = tracking_area_event.payload as unknown as TrackingAreaClickedMessage;
					if (clicked_msg.id === annotation.id) {
						is_hovered = true;
					}
					break;
				case 'TrackingAreaEntered':
					let entered_msg = tracking_area_event.payload as unknown as TrackingAreaEnteredMessage;
					if (entered_msg.id === annotation.id) {
						is_hovered = true;
					}
					break;
				case 'TrackingAreaExited':
					let exited_msg = tracking_area_event.payload as unknown as TrackingAreaExitedMessage;
					if (exited_msg.id === annotation.id) {
						is_hovered = false;
					}
					break;
				default:
					break;
			}
		});
	};

	listenToTrackingAreaEvents();
	listenToDocsGenerationEvents();
</script>

{#if code_overlay_position !== null && annotation.annotation_icon !== null}
	<div
		style="position: absolute; top: {Math.round(
			annotation.annotation_icon.origin.y - code_overlay_position.y
		)}px; left: {Math.round(
			annotation.annotation_icon.origin.x - code_overlay_position.x
		)}px; width: {Math.round(annotation.annotation_icon.size.width)}px;height: {Math.round(
			annotation.annotation_icon.size.height
		)}px;"
	>
		<AnnotationIcon {is_hovered} {is_processing} />
		
	</div>
{/if}

{#if code_overlay_position !== null && annotation.annotation_codeblock !== null }
	<div
		style="position: absolute; top: {Math.round(
			annotation.annotation_codeblock.origin.y - code_overlay_position.y
		)}px; left: {Math.round(
			annotation.annotation_codeblock.origin.x - code_overlay_position.x
		)}px; width: {Math.round(annotation.annotation_codeblock.size.width)}px;height: {Math.round(
			annotation.annotation_codeblock.size.height
		)}px;"
	>
		<AnnotationLine
			visible={is_hovered || is_processing}
			highlighted={is_hovered && !is_processing}
		/>
	</div>
{/if}
