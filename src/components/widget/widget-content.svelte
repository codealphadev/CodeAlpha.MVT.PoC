<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { fade } from 'svelte/transition';
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { FERefactoringSuggestion } from '../../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';
	import type { ReplaceSuggestionsMessage } from '../../../src-tauri/bindings/features/refactoring/ReplaceSuggestionsMessage';
	import type { SuggestionEvent } from '../../../src-tauri/bindings/features/refactoring/SuggestionEvent';
	import type { EventViewport } from '../../../src-tauri/bindings/macOS_specific/EventViewport';
	import type { EventRuleExecutionState } from '../../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import PretzlIcon from '../common/logo/pretzl-icon.svelte';
	import { filter_and_sort_suggestions } from '../suggestions/suggestions';
	import DocsGeneration from './docs-generation.svelte';
	import IconEllipse from './icons/icon-ellipse.svelte';
	import IconProcessing from './icons/icon-processing.svelte';
	import SuggestionsNumber from './suggestions-number.svelte';
	import SwiftFormat from './swift-format.svelte';

	let ruleExecutionState: EventRuleExecutionState | null = null;
	let PROCESSING_TIMEOUT = 15000; // ms
	let SHOW_ALTERNATE_ICON_DURATION = 2000; // ms

	let active_window_uid: number | null = null;
	const listenToViewportEvents = async () => {
		let ViewportChannel: ChannelList = 'EventViewport';
		await listen(ViewportChannel, (e) => {
			const { event, payload } = JSON.parse(e.payload as string) as EventViewport;

			switch (event) {
				case 'XcodeViewportUpdate':
					active_window_uid = payload.viewport_properties.window_uid;
					break;
				default:
					break;
			}
		});
	};
	listenToViewportEvents();

	let suggestions: ReplaceSuggestionsMessage['suggestions'] = {};
	$: filtered_suggestions = filter_and_sort_suggestions(suggestions, active_window_uid);
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

	const listenExecutionState = async () => {
		await listen('EventRuleExecutionState' as ChannelList, (event) => {
			ruleExecutionState = JSON.parse(event.payload as string) as EventRuleExecutionState;

			// In case we receive a "finished" event, register a timeout to reset the
			// widget icon to the logo after some delay
			switch (ruleExecutionState.event) {
				case 'SwiftFormatFinished':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, SHOW_ALTERNATE_ICON_DURATION);
					break;
				case 'SwiftFormatFailed':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, SHOW_ALTERNATE_ICON_DURATION);
					break;
				case 'NodeExplanationStarted':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, PROCESSING_TIMEOUT);
					break;
				case 'NodeExplanationFailed':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, SHOW_ALTERNATE_ICON_DURATION);
					break;
				case 'NodeExplanationFetched':
					setTimeout(async () => {
						ruleExecutionState = null;
					}, SHOW_ALTERNATE_ICON_DURATION);
					break;
				default:
					break;
			}
		});
	};
	listenExecutionState();

	type WidgetMode =
		| 'default'
		| 'NodeExplanationFailed'
		| 'NodeExplanationFetched'
		| 'processing'
		| 'SwiftFormatFailed'
		| 'SwiftFormatFinished'
		| number;

	function get_widget_mode(
		rule_execution_state: EventRuleExecutionState | null,
		suggestions: [string, FERefactoringSuggestion][],
		window_uid: number | null
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
		if (window_uid) {
			let suggestions_count = suggestions.length;
			if (suggestions_count > 0) {
				return suggestions_count;
			}
		}
		return 'default';
	}
	$: widget_mode = get_widget_mode(ruleExecutionState, filtered_suggestions, active_window_uid);
</script>

<div class="absolute bottom-0 right-0 w-12 h-12">
	{#key widget_mode}
		<div
			data-tauri-drag-region
			class="flex items-center justify-center w-[32px] h-[32px] overflow-hidden m-auto absolute top-0 right-0 bottom-0 left-0"
			in:fade={{
				duration: 200
			}}
		>
			{#if widget_mode === 'SwiftFormatFinished' || widget_mode === 'SwiftFormatFailed'}
				<SwiftFormat state={widget_mode} />
			{:else if widget_mode === 'NodeExplanationFetched' || widget_mode === 'NodeExplanationFailed'}
				<DocsGeneration state={widget_mode} />
			{:else if widget_mode === 'processing'}
				<IconProcessing />
			{:else if typeof widget_mode === 'number'}
				<div class="fixed w-[32px] h-[32px]">
					<IconEllipse />
				</div>
				<SuggestionsNumber count={widget_mode} />
			{:else}
				<PretzlIcon />
			{/if}
		</div>
	{/key}
</div>
