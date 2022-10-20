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
	import { every_suggestion_is_new, filter_and_sort_suggestions } from './suggestions';
	import type { FERefactoringSuggestion } from '../../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';
	import type { EventViewport } from '../../../src-tauri/bindings/macOS_specific/EventViewport';
	import LoadingSuggestions from './loading-suggestions.svelte';

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
		const old_window_width = window_width;
		const old_window_height = window_height;
		updateDimensions();

		if (window_width && window_height && old_window_height && old_window_width) {
			if (window_height === old_window_height && window_width === old_window_width) {
				return;
			}
			let appWindow: AppWindow = 'Main';
			invoke('cmd_resize_window', {
				appWindow: appWindow,
				sizeY: window_height + tail_height_px,

				sizeX: window_width
			});
		}
	});

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

	$: is_loading_suggestions = every_suggestion_is_new(suggestions, active_window_uid);

	const listenToSuggestionEvents = async () => {
		let suggestion_channel: ChannelList = 'SuggestionEvent';
		await listen(suggestion_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(event.payload as string) as SuggestionEvent;

			switch (event_type) {
				case 'ReplaceSuggestions':
					suggestions = payload.suggestions;
					if (
						selected_suggestion_id &&
						active_window_uid &&
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

{#if active_window_uid !== null && filtered_suggestions?.length > 0}
	<div
		class="flex bg-background flex-col gap-5 shrink-0 rounded-b-xl max-h-[700px] overflow-y-auto overscroll-none mt-9 px-4 pt-3 pb-4"
	>
		{#each filtered_suggestions as [id, suggestion]}
			{#key id}
				<Suggestion
					on:click={() => {
						if (active_window_uid === null) {
							return;
						}
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
{:else if is_loading_suggestions == true}
	<LoadingSuggestions />
{:else}
	<NoSuggestions />
{/if}
