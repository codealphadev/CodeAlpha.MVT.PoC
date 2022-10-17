<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import WidgetBackground from '../components/widget/widget-background.svelte';
	import { emit, listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventUserInteraction } from '../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import type { CoreEngineState } from '../../src-tauri/bindings/app_state/CoreEngineState';
	import { toggle_main_window } from '../utils';
	import type { EventWindowControls } from '../../src-tauri/bindings/window_controls/EventWindowControls';
	import type { HideAppWindowMessage } from '../../src-tauri/bindings/window_controls/HideAppWindowMessage';
	import WidgetContent from '../components/widget/widget-content.svelte';
	import BadgeNoAiMode from '../components/widget/badge-no-ai-mode.svelte';
	import type { AiFeaturesStatusMessage } from '../../src-tauri/bindings/user_interaction/AiFeaturesStatusMessage';
	import { onMount } from 'svelte';

	let main_window_active = false;
	let ai_mode_active = true;

	onMount(() => {
		fetch_core_engine_state();
	});

	const fetch_core_engine_state = async () => {
		let core_engine_state: CoreEngineState | null = await invoke('cmd_get_core_engine_state');

		ai_mode_active = core_engine_state?.ai_features_active ?? true;
	};

	const clickAction = async () => {
		main_window_active = !main_window_active;
		toggle_main_window(main_window_active);
	};

	const listenToMainWindowHideEvents = async () => {
		let WindowControlsChannel: ChannelList = 'EventWindowControls';
		await listen(WindowControlsChannel, (e) => {
			const { event, payload } = JSON.parse(e.payload as string) as EventWindowControls;

			switch (event) {
				case 'AppWindowHide':
					const { app_windows } = payload as HideAppWindowMessage;
					if (app_windows.includes('Main')) {
						main_window_active = false;
					}
					break;
				default:
					break;
			}
		});
	};
	listenToMainWindowHideEvents();

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

	const handle_release_drag = async () => {
		const event: EventUserInteraction = {
			event: 'ToggleMainWindow',
			payload: main_window_active
		};
		const channel: ChannelList = 'EventUserInteractions';

		await emit(channel, event);

		// Rebind the MainWindow and WidgetWindow. Because of how MacOS works, we need to have some
		// delay between setting a new position and recreating the parent/child relationship.
		// Pausing the main thread is not possible. Also, running this task async is also not trivial.
		// We send a message to the main thread to run this task.
		// EventWindowControls::RebindMainAndWidget.publish_to_tauri(&app_handle());
		if (main_window_active) {
			setTimeout(() => {
				invoke('cmd_rebind_main_widget');
			}, 100);
		}
	};

	let startX: number | undefined = undefined;
	let startY: number | undefined = undefined;
	const minimum_move_distance_to_fire_click = 10;

	function handleMouseDown(event: MouseEvent) {
		startX = event.screenX;
		startY = event.screenY;
	}

	function handleMouseUp(event: MouseEvent) {
		if (startX === undefined || startY === undefined) {
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
	<WidgetBackground />
	<WidgetContent />
	{#if !ai_mode_active}
		<BadgeNoAiMode />
	{/if}
	<div
		data-tauri-drag-region
		class="absolute bottom-0 right-0 w-12 h-12"
		on:mousedown={handleMouseDown}
		on:mouseup={handleMouseUp}
	/>
</div>
