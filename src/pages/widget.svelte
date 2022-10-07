<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import WidgetBackground from '../components/widget/widget-background.svelte';
	import IconLogo from '../components/widget/icons/icon-logo.svelte';
	import IconLogoGreyscale from '../components/widget/icons/icon-logo-greyscale.svelte';
	import { emit, listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventRuleExecutionState } from '../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import WidgetProcessing from '../components/widget/widget-processing.svelte';
	import WidgetBackgroundGreyscale from '../components/widget/widget-background-greyscale.svelte';
	import { fade } from 'svelte/transition';
	import SwiftFormat from '../components/widget/swift-format.svelte';
	import DocsGeneration from '../components/widget/docs-generation.svelte';
	import type { FERefactoringSuggestion } from '../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';
	import type { SuggestionEvent } from '../../src-tauri/bindings/features/refactoring/SuggestionEvent';
	import SuggestionsNumber from '../components/widget/suggestions-number.svelte';
	import type { EventUserInteraction } from '../../src-tauri/bindings/user_interaction/EventUserInteraction';

	let main_window_active = false;
	let ruleExecutionState: EventRuleExecutionState | null = null;
	let processing_timeout = 15000; // ms
	let show_alternate_icon_duration = 2000; // ms

	// Important logic! Don't change this unless you know what you're doing. :-P
	// Should be moved into a separate file.
	const clickAction = async () => {
		main_window_active = !main_window_active;
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

	const handle_release_drag = async () => {
		// invoke('cmd_toggle_app_activation', { appActive: app_active });

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

	let suggestions: { [id: string]: FERefactoringSuggestion } = {};

	const listenToSuggestionEvents = async () => {
		let suggestion_channel: ChannelList = 'SuggestionEvent';
		await listen(suggestion_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as SuggestionEvent;

			switch (event_type) {
				case 'ReplaceSuggestions':
					suggestions = payload.suggestions;
					break;

				default:
					break;
			}
		});
	};

	listenToSuggestionEvents();

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
	type WidgetMode =
		| 'idle'
		| 'processing'
		| number
		| 'inactive'
		| 'SwiftFormatFailed'
		| 'SwiftFormatFinished'
		| 'NodeExplanationFailed'
		| 'NodeExplanationFetched';

	function get_widget_mode(
		rule_execution_state: EventRuleExecutionState | null,
		suggestions: { [id: string]: FERefactoringSuggestion }
	): WidgetMode {
		switch (rule_execution_state?.event) {
			case 'NodeExplanationFailed':
			case 'NodeExplanationFetched':
			case 'SwiftFormatFailed':
			case 'SwiftFormatFinished':
				return rule_execution_state.event;
			case 'NodeExplanationStarted':
				return 'processing';
		}
		let suggestions_count = Object.keys(suggestions).length;
		if (suggestions_count > 0) {
			return suggestions_count;
		}
		return 'idle';
	}
	$: widget_mode = get_widget_mode(ruleExecutionState, suggestions);
</script>

<div class="relative overflow-hidden w-full h-full">
	{#if main_window_active === false}
		<WidgetBackgroundGreyscale />
	{:else}
		<WidgetBackground />
	{/if}

	<div class="absolute bottom-0 right-0 w-12 h-12">
		<div data-tauri-drag-region class="flex items-center justify-center h-screen">
			{#key widget_mode}
				<div
					class="w-[38px] h-[38px] rounded-full overflow-hidden"
					in:fade={{
						duration: 200
					}}
				>
					{#if widget_mode === 'inactive'}
						<IconLogoGreyscale />
					{:else if widget_mode === 'SwiftFormatFinished' || widget_mode === 'SwiftFormatFailed'}
						<SwiftFormat state={widget_mode} />
					{:else if widget_mode === 'NodeExplanationFetched' || widget_mode === 'NodeExplanationFailed'}
						<DocsGeneration state={widget_mode} />
					{:else if widget_mode === 'processing'}
						<WidgetProcessing />
					{:else if typeof widget_mode === 'number'}
						<SuggestionsNumber count={widget_mode} />
					{:else}
						<IconLogo />
					{/if}
				</div>
			{/key}
		</div>
	</div>
	<div
		data-tauri-drag-region
		class="absolute bottom-0 right-0 w-12 h-12"
		on:mousedown={handleMouseDown}
		on:mouseup={handleMouseUp}
	/>
</div>
