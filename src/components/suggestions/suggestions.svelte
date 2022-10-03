<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { FERefactoringSuggestion } from '../../../src-tauri/bindings/features/refactoring/FERefactoringSuggestion';
	import type { SuggestionEvent } from '../../../src-tauri/bindings/features/refactoring/SuggestionEvent';
	import Suggestion from './suggestion.svelte';
	import omit from 'lodash/omit';

    let suggestions: {[id: string]: FERefactoringSuggestion}  = {};

	const listenToSuggestionEvents = async () => {
		let suggestion_channel: ChannelList = 'SuggestionEvent';
		await listen(suggestion_channel, (event) => {
			console.log(event);
			const { payload, event: event_type } = JSON.parse(
				event.payload as string
			) as SuggestionEvent;

			switch (event_type) {
				case 'UpdateSuggestion':
					suggestions[payload.suggestion.id] = payload.suggestion;
					suggestions = suggestions;
					break;

				case 'RemoveSuggestion':
					suggestions = omit(suggestions, payload.id);
					break;
				default:
					break;
			}
		});
	};

	listenToSuggestionEvents();
</script>

<div class="flex flex-col gap-5">
	{#each Object.values(suggestions) as suggestion}
		{#key suggestion.id}
			<Suggestion suggestion={suggestion}/>
		{/key}
	{/each}
</div>