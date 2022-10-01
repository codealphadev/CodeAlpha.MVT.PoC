<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
	import type { RefactoringOperation } from '../../../src-tauri/bindings/features/refactoring/RefactoringOperation';
	import type { SuggestionEvent } from '../../../src-tauri/bindings/features/refactoring/SuggestionEvent';
	import Suggestion from './suggestion.svelte';
	

    let suggestions: RefactoringOperation[] | null = null;

	const listenToSuggestionEvents = async () => {
		let suggestion_channel: ChannelList = 'SuggestionEvent';
		await listen(suggestion_channel, (event) => {
			console.log(event);
			const { payload, event: event_type } = JSON.parse(
				event.payload as string
			) as SuggestionEvent;

			switch (event_type) {
				case 'UpdateSuggestions':
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
	{#if suggestions}
		{#each suggestions as suggestion}
			{#key suggestion.id}
				<Suggestion suggestion={suggestion}/>
			{/key}
		{/each}
	{/if}
</div>