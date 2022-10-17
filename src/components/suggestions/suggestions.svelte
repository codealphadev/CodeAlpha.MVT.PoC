<script lang="ts">
	import { emit, listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { SuggestionEvent } from '../../../src-tauri/bindings/features/refactoring/SuggestionEvent';
	import Suggestion from './suggestion.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { afterUpdate } from 'svelte';
	import type { AppWindow } from '../../../src-tauri/bindings/AppWindow';
	import NoSuggestions from '../suggestions/no-suggestions.svelte';
	import type { ReplaceSuggestionsMessage } from '../../../src-tauri/bindings/features/refactoring/ReplaceSuggestionsMessage';
	import type { EventUserInteraction } from '../../../src-tauri/bindings/user_interaction/EventUserInteraction';
	import { filter_and_sort_suggestions } from './suggestions';
	import type { FERefactoringSuggestion } from '../../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';

	export let active_window_uid: number;
	export let CONTAINER_DOM_ID: string;

	let selected_suggestion_id: string | null = null;
	let window_width: number | null = null;
	let window_height: number | null = null;
	let filtered_suggestions: [string, FERefactoringSuggestion][] = [];
	let tail_height_px = 12;

	const select_suggestion = async (suggestion_id: string | null, editor_window_uid: number) => {
		selected_suggestion_id = suggestion_id;
		const event: EventUserInteraction = {
			event: 'UpdateSelectedSuggestion',
			payload: { id: suggestion_id, editor_window_uid }
		};
		const channel: ChannelList = 'EventUserInteractions';
		await emit(channel, event);
	};

	afterUpdate(() => {
		updateDimensions();

		if (window_width && window_height) {
			let appWindow: AppWindow = 'Main';
			invoke('cmd_resize_window', {
				appWindow: appWindow,
				sizeY: window_height + tail_height_px,

				sizeX: window_width
			});
		}
	});

	const updateDimensions = () => {
		let element = document.getElementById(CONTAINER_DOM_ID);

		if (element === null) {
			return;
		}

		let positionInfo = element.getBoundingClientRect();

		window_width = positionInfo.width;
		window_height = positionInfo.height;
	};
	let suggestions: ReplaceSuggestionsMessage['suggestions'] = {};
	$: filtered_suggestions = filter_and_sort_suggestions(suggestions, active_window_uid);

	const listenToSuggestionEvents = async () => {
		let suggestion_channel: ChannelList = 'SuggestionEvent';
		await listen(suggestion_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as SuggestionEvent;

			switch (event_type) {
				case 'ReplaceSuggestions':
					suggestions = payload.suggestions;
					if (
						selected_suggestion_id &&
						!Object.keys(suggestions[active_window_uid] ?? []).includes(selected_suggestion_id)
					) {
						selected_suggestion_id = null;
					}
					break;
				default:
					break;
			}
		});
	};
	listenToSuggestionEvents();
</script>

{#if filtered_suggestions?.length > 0}
	<div
		class="flex bg-background flex-col gap-5 shrink-0 rounded-b-xl max-h-[700px] overflow-y-auto overscroll-none mt-9 px-4 pt-3 pb-4"
	>
		{#each filtered_suggestions as [id, suggestion]}
			{#key id}
				<Suggestion
					on:click={() => {
						if (selected_suggestion_id === id) {
							select_suggestion(null, active_window_uid);
						} else {
							select_suggestion(id, active_window_uid);
						}
					}}
					expanded={id == selected_suggestion_id}
					{suggestion}
					suggestion_id={id}
					window_uid={active_window_uid}
				/>
			{/key}
		{/each}
	</div>
{:else}
	<NoSuggestions />
{/if}
