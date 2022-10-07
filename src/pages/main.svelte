<script lang="ts">
	import { Event, listen } from '@tauri-apps/api/event';
	import BubbleIcon from '../components/main/bubble-icon.svelte';
	import Suggestions from '../components/suggestions/suggestions.svelte';

	let dom_id = 'main-window-container';

	let bubbleOrientationRight = true;
	const listenToGlobalEvents = async () => {
		await listen('evt-content-window-orientation', (event) => {
			const tauriEvent = event as Event<any>;
			bubbleOrientationRight = tauriEvent.payload.orientation_right;
		});
	};

	listenToGlobalEvents();
</script>

<div class="flex flex-col shrink-0 justify-end transform overflow-hidden h-full">
	<div
		id={dom_id}
		class="flex flex-col shrink-0 rounded-xl max-h-[800px] overflow-y-auto overscroll-none"
	>
		<Suggestions window_dom_id={dom_id} />
	</div>
	<div class="h-6 mr-4 ml-4 {`${bubbleOrientationRight ? 'ml-auto' : 'ml-4'}`}">
		<BubbleIcon />
	</div>
</div>
