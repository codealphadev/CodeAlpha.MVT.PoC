<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import WidgetBackground from '../components/widget/widget-background.svelte';
	import IconLogo from '../components/widget/icons/icon-logo.svelte';
	import IconLogoGreyscale from '../components/widget/icons/icon-logo-greyscale.svelte';
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventRuleExecutionState } from '../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import WidgetProcessing from '../components/widget/widget-processing.svelte';
	import WidgetBackgroundGreyscale from '../components/widget/widget-background-greyscale.svelte';
	import { fade } from 'svelte/transition';
	import SwiftFormat from '../components/widget/swift-format.svelte';
	import DocsGeneration from '../components/widget/docs-generation.svelte';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';

	let app_active = true;
	let ruleExecutionState: EventRuleExecutionState | null = null;
	let processing_timeout = 15000; // ms
	let show_alternate_icon_duration = 2000; // ms

	// Important logic! Don't change this unless you know what you're doing. :-P
	// Should be moved into a separate file.
	const clickAction = async () => {
		app_active = !app_active;
		invoke('cmd_toggle_app_activation', { appActive: app_active });

		// Rebind the MainWindow and WidgetWindow. Because of how MacOS works, we need to have some
		// delay between setting a new position and recreating the parent/child relationship.
		// Pausing the main thread is not possible. Also, running this task async is also not trivial.
		// We send a message to the main thread to run this task.
		// EventWindowControls::RebindMainAndWidget.publish_to_tauri(&app_handle());
		if (app_active) {
			setTimeout(() => {
				invoke('cmd_rebind_main_widget');
			}, 100);
		}
	};

	const handle_release_drag = async () => {
		invoke('cmd_toggle_app_activation', { appActive: app_active });

		// Rebind the MainWindow and WidgetWindow. Because of how MacOS works, we need to have some
		// delay between setting a new position and recreating the parent/child relationship.
		// Pausing the main thread is not possible. Also, running this task async is also not trivial.
		// We send a message to the main thread to run this task.
		// EventWindowControls::RebindMainAndWidget.publish_to_tauri(&app_handle());
		if (app_active) {
			setTimeout(() => {
				invoke('cmd_rebind_main_widget');
			}, 100);
		}
	};

	const listenTauriEvents = async () => {
		await listen('EventRuleExecutionState' as ChannelList, (event) => {
			ruleExecutionState = JSON.parse(event.payload as string) as EventRuleExecutionState;

			// In case we receive a "finished" event, register a timeout to reset the
			// widget icon to the logo after some delay
			switch (ruleExecutionState.event) {
				case 'SwiftFormatFinished':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, show_alternate_icon_duration);
					break;
				case 'SwiftFormatFailed':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, show_alternate_icon_duration);
					break;
				case 'NodeExplanationStarted':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, processing_timeout);
					break;
				case 'NodeExplanationFailed':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, show_alternate_icon_duration);
					break;
				case 'NodeExplanationFetched':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, show_alternate_icon_duration);
					break;
				default:
					break;
			}
		});
	};

	const listen_mouse_move = async () => {
		await listen('EventWindowControls' as ChannelList, (event) => {
			let tracking_event = JSON.parse(event.payload as string) as EventWindowControls;

			console.log('tracking_event', tracking_event);
		});
	};
	listen_mouse_move();

	let startX: number | undefined = undefined;
	let startY: number | undefined = undefined;
	const minimum_move_distance_to_fire_click = 10;
	listenTauriEvents();

	function handleMouseDown(event: MouseEvent) {
		startX = event.screenX;
		startY = event.screenY;
	}

	function handleMouseUp(event: MouseEvent) {
		if (startX === undefined || startY === undefined) {
			clickAction();
			return;
		}
		const diffX = Math.abs(event.screenX - startX);
		const diffY = Math.abs(event.screenY - startY);

		if (
			diffX < minimum_move_distance_to_fire_click &&
			diffY < minimum_move_distance_to_fire_click
		) {
			clickAction();
		} else {
			handle_release_drag();
		}
	}
</script>

<div class="relative overflow-hidden w-full h-full">
	{#if app_active === false}
		<WidgetBackgroundGreyscale />
	{:else}
		<WidgetBackground />
	{/if}

	<div class="absolute bottom-0 right-0 w-12 h-12">
		<div
			data-tauri-drag-region
			on:click={clickAction}
			class="flex items-center justify-center h-screen"
		>
			<div class="w-[36px]">
				{#if app_active === false}
					<div in:fade={{ duration: 200 }}>
						<IconLogoGreyscale />
					</div>
				{:else if ruleExecutionState != null && (ruleExecutionState.event === 'SwiftFormatFinished' || ruleExecutionState.event === 'SwiftFormatFailed')}
					<div in:fade={{ duration: 200 }}>
						<SwiftFormat event={ruleExecutionState.event} />
					</div>
				{:else if ruleExecutionState != null && (ruleExecutionState.event === 'NodeExplanationFetched' || ruleExecutionState.event === 'NodeExplanationFailed')}
					<div in:fade={{ duration: 200 }}>
						<DocsGeneration event={ruleExecutionState.event} />
					</div>
				{:else if ruleExecutionState != null && ruleExecutionState.event === 'NodeExplanationStarted'}
					<div in:fade={{ duration: 200 }}>
						<WidgetProcessing />
					</div>
				{:else}
					<div in:fade={{ duration: 200 }}>
						<IconLogo />
					</div>
				{/if}
			</div>
		</div>
	</div>
	<div
		data-tauri-drag-region
		class="absolute bottom-0 right-0 w-12 h-12"
		on:mousedown={handleMouseDown}
		on:mouseup={handleMouseUp}
	/>
</div>
