<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { CodeAnnotationMessage } from '../../../../src-tauri/bindings/features/docs_generation/CodeAnnotationMessage';
	import type { LogicalPosition } from '../../../../src-tauri/bindings/geometry/LogicalPosition';
	import type { EventWindowControls } from '../../../../src-tauri/bindings/window_controls/EventWindowControls';
	import type { TrackingAreaClickedMessage } from '../../../../src-tauri/bindings/window_controls/TrackingAreaClickedMessage';
	import type { TrackingAreaEnteredMessage } from '../../../../src-tauri/bindings/window_controls/TrackingAreaEnteredMessage';
	import type { TrackingAreaExitedMessage } from '../../../../src-tauri/bindings/window_controls/TrackingAreaExitedMessage';
	import AnnotationIcon from './annotation-icon.svelte';

	export let annotation_msg: CodeAnnotationMessage | null;
	export let code_overlay_position: LogicalPosition | null;

	let is_hovered = false;
	let was_clicked = false;

	const listenTauriEvents = async () => {
		// Listen for click & hover events on the tracking area
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (event) => {
			const tracking_area_event = JSON.parse(event.payload as string) as EventWindowControls;

			if (annotation_msg !== null) {
				switch (tracking_area_event.event) {
					case 'TrackingAreaClicked':
						let clicked_msg = tracking_area_event.payload as unknown as TrackingAreaClickedMessage;
						if (clicked_msg.id === annotation_msg.id) {
							was_clicked = false;
						}
						break;
					case 'TrackingAreaEntered':
						console.log('TrackingAreaEntered');
						let entered_msg = tracking_area_event.payload as unknown as TrackingAreaEnteredMessage;
						if (entered_msg.id === annotation_msg.id) {
							is_hovered = true;
						}
						break;
					case 'TrackingAreaExited':
						console.log('TrackingAreaExited');
						let exited_msg = tracking_area_event.payload as unknown as TrackingAreaExitedMessage;
						if (exited_msg.id === annotation_msg.id) {
							is_hovered = false;
						}
						break;
					default:
						break;
				}
			}
		});
	};

	listenTauriEvents();

	$: show_highlighted = was_clicked ? false : is_hovered;
</script>

{#if annotation_msg !== null && code_overlay_position !== null && annotation_msg.annotation_icon !== null}
	<div
		style="position: absolute; top: {Math.round(
			annotation_msg.annotation_icon.origin.y - code_overlay_position.y
		)}px; left: {Math.round(
			annotation_msg.annotation_icon.origin.x - code_overlay_position.x
		)}px; width: {Math.round(annotation_msg.annotation_icon.size.width)}px;height: {Math.round(
			annotation_msg.annotation_icon.size.height
		)}px;"
	>
		<AnnotationIcon {show_highlighted} />
	</div>
{/if}
