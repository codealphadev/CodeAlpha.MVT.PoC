<script lang="ts">
	import { Event, listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/tauri';
	import { afterUpdate } from 'svelte';
	import type { AppWindow } from '../../src-tauri/bindings/AppWindow';
	import BubbleIcon from '../components/main/bubble-icon.svelte';
	import Suggestions from '../components/suggestions/suggestions.svelte';

	let window_width: number | null = null;
	let window_height: number | null = null;

	let dom_id = 'main-window-container';

	afterUpdate(() => {
		updateDimensions();

		if (window_width && window_height) {
			let appWindow: AppWindow = 'Content'; // TODO

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



	let bubbleOrientationRight = true;
	const listenToGlobalEvents = async () => {
		await listen('evt-content-window-orientation', (event) => {
			const tauriEvent = event as Event<any>;
			bubbleOrientationRight = tauriEvent.payload.orientation_right;
		});
	};

	listenToGlobalEvents();
</script>

<div id={dom_id} class="flex flex-col justify-end transform overflow-hidden h-full">
	<div class="flex flex-col bg-background rounded-xl">
		<Suggestions/>
	</div>
	<div class="h-6 mr-4 ml-4 {`${bubbleOrientationRight ? 'ml-auto' : 'ml-4'}`}">
		<BubbleIcon />
	</div>
</div>
