<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/tauri';
	import { afterUpdate } from 'svelte';
	import { fade } from 'svelte/transition';
	import type { AppWindow } from '../../src-tauri/bindings/AppWindow';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { RefactoringOperation } from '../../src-tauri/bindings/features/refactoring/RefactoringOperation';
	import type { SuggestionEvent } from '../../src-tauri/bindings/features/refactoring/SuggestionEvent';
	import Suggestion from '../components/suggestions/suggestion.svelte';
	

    let suggestions: RefactoringOperation[] | null = null;

	let dom_id = 'suggestions-window-container';

	let window_width: number | null = null;
	let window_height: number | null = null;

	// Logic to always resize the content window to the size of the HTML
	afterUpdate(() => {
		updateDimensions();

		if (window_width && window_height) {
			let appWindow: AppWindow = 'Explain'; // TODO

			invoke('cmd_resize_window', {
				appWindow: appWindow,
				sizeY: window_height,
				sizeX: window_width
			});
		}
	});

	const updateDimensions = async () => {
		let element = document.getElementById(dom_id);

		if (element === null) {
			return;
		}

		let positionInfo = element.getBoundingClientRect();

		window_width = positionInfo.width;
		window_height = positionInfo.height;
	};

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
					console.log(payload.suggestions[0]?.new_text_content_string)
					console.log(payload.suggestions[0]?.old_text_content_string)
					break;
				default:
					break;
			}
		});
	};

	listenToSuggestionEvents();
</script>

<div data-tauri-drag-region class="absolute w-full h-20"/>

<div id={dom_id} class="rounded-xl bg-background overflow-hidden p-4 flex flex-col items-start gap-3 border-none" in:fade="{{ duration: 100}}">
    {#if suggestions}
        {#each suggestions as suggestion}
            {#key suggestion.id}
                <Suggestion suggestion={suggestion}/>
            {/key}
        {/each}
    {/if}
</div>
