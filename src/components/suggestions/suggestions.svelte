<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { FERefactoringSuggestion } from '../../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';
	import type { SuggestionEvent } from '../../../src-tauri/bindings/features/refactoring/SuggestionEvent';
	import Suggestion from './suggestion.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { afterUpdate } from 'svelte';
	import type { AppWindow } from '../../../src-tauri/bindings/AppWindow';
	import NoSuggestions from '../suggestions/no-suggestions.svelte';
	let window_width: number | null = null;
	let window_height: number | null = null;
	export let window_dom_id: string;

	afterUpdate(() => {
		updateDimensions();

		if (window_width && window_height) {
			let appWindow: AppWindow = 'Main';
			invoke('cmd_resize_window', {
				appWindow: appWindow,
				sizeY: window_height + 24,

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
	let suggestions: { [id: string]: FERefactoringSuggestion } = {};
	$: sorted_suggestions = Object.values(suggestions).sort((a, b) => a.id.localeCompare(b.id));

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
	{#if sorted_suggestions.length > 0}
		{#each sorted_suggestions as suggestion}
			{#key suggestion.id}
				<Suggestion {suggestion} />
			{/key}
		{/each}
	{:else}
		<NoSuggestions />
	{/if}
</div>
