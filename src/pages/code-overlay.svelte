<script lang="ts">
	import { listen } from '@tauri-apps/api/event';

	import { appWindow, getCurrent } from '@tauri-apps/api/window';
	import { afterUpdate } from 'svelte';

	let height = 0;

	const fetchWindowHeight = async () => {
		const outerSize = await getCurrent().outerSize();
		const scaleFactor = await getCurrent().scaleFactor();
		const logicalSize = outerSize.toLogical(scaleFactor);

		height = logicalSize.height;

		console.log(height);
	};

	const listenTauriEvents = async () => {
		await listen('event-compute-height', (event) => {
			fetchWindowHeight();
		});
	};

	listenTauriEvents();
</script>

<div style="height: {height}px" class="bg-teal-300 opacity-20 h-full w-full " />
