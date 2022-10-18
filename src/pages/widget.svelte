<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import WidgetBackground from '../components/widget/widget-background.svelte';
	import { toggle_main_window } from '../utils';
	import WidgetContent from '../components/widget/widget-content.svelte';
	import { main_window_active_store } from '../state';
	import BadgeNoAiMode from '../components/widget/badge-no-ai-mode.svelte';
	import { onMount } from 'svelte';
	import type { CoreEngineState } from '../../src-tauri/bindings/app_state/CoreEngineState';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventUserInteraction } from '../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import { listen } from '@tauri-apps/api/event';
	import type { AiFeaturesStatusMessage } from '../../src-tauri/bindings/user_interaction/AiFeaturesStatusMessage';

	let main_window_active = false;

	main_window_active_store.subscribe((value: boolean) => {
		main_window_active = value;
	});

	let ai_mode_active = true;

	onMount(() => {
		fetch_core_engine_state();
	});

	const fetch_core_engine_state = async () => {
		let core_engine_state: CoreEngineState | null = await invoke('cmd_get_core_engine_state');

		ai_mode_active = core_engine_state?.ai_features_active ?? true;
	};

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

	const clickAction = async () => {
		toggle_main_window(!main_window_active);
	};

	const handle_release_drag = async () => {
		// We do this call to cope with the fact that macOS unpredictably
		// repositions the main window when it is being dragged into the menu bar.
		toggle_main_window(main_window_active);
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
