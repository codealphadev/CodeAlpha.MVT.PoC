<script lang="ts">
	import { Event, listen } from '@tauri-apps/api/event';
	import Tail from '../components/main/tail.svelte';
	import Suggestions from '../components/suggestions/suggestions.svelte';
	import type { TailOrientation } from '../../src-tauri/bindings/window_controls/TailOrientation';

	let dom_id = 'main-window-container';

	let tail_orientation: TailOrientation = 'Right';
	const listenToGlobalEvents = async () => {
		await listen('tail-orientation-flipped', (event) => {
			const tauriEvent = event as Event<any>;

			console.log('tauriEvent', tauriEvent.payload);

			if ((tauriEvent.payload as TailOrientation) == 'Left') {
				tail_orientation = 'Left';
			} else {
				tail_orientation = 'Right';
			}
		});
	};

	listenToGlobalEvents();
</script>

<div class="flex flex-col bg-transparent shrink-0 justify-end transform overflow-hidden h-full">
	<div
		id={dom_id}
		class="flex bg-background flex-col shrink-0 rounded-xl max-h-[800px] overflow-y-auto overscroll-none"
	>
		<Suggestions window_dom_id={dom_id} />
	</div>
	<div class="h-3  {`${tail_orientation == 'Left' ? 'mr-auto ml-[18px]' : 'ml-auto mr-[18px]'}`}">
		<Tail />
	</div>
</div>
