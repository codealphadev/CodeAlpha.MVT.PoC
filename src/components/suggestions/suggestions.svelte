<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { SuggestionEvent } from '../../../src-tauri/bindings/features/refactoring/SuggestionEvent';
	import Suggestion from './suggestion.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { afterUpdate } from 'svelte';
	import type { AppWindow } from '../../../src-tauri/bindings/AppWindow';
	import NoSuggestions from '../suggestions/no-suggestions.svelte';
	import type { ReplaceSuggestionsMessage } from '../../../src-tauri/bindings/features/refactoring/ReplaceSuggestionsMessage';

	let window_width: number | null = null;
	let window_height: number | null = null;
	export let window_dom_id: string;
	export let active_window_uid: number;

	let tail_height_px = 12;

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
		let element = document.getElementById(window_dom_id);

		if (element === null) {
			return;
		}

		let positionInfo = element.getBoundingClientRect();

		window_width = positionInfo.width;
		window_height = positionInfo.height;
	};
	let suggestions: ReplaceSuggestionsMessage['suggestions'] = {};
	$: filtered_suggestions = Object.entries(suggestions[active_window_uid] ?? {}).sort((a, b) =>
		a[0].localeCompare(b[0])
	);

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
</script>

<div class="flex flex-col gap-5">
	{#if filtered_suggestions.length > 0}
		{#each filtered_suggestions as [id, suggestion]}
			{#key id}
				<Suggestion {suggestion} suggestion_id={id} window_uid={active_window_uid} />
			{/key}
		{/each}
	{:else}
		<NoSuggestions />
	{/if}
</div>
