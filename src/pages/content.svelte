<script lang="ts">
	import { afterUpdate } from 'svelte';
	import { Event, listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import SearchReplace from '../components/content/search-replace.svelte';
	import OptionsMenu from '../components/content/options-menu.svelte';
	import BubbleIcon from '../components/content/bubble-icon.svelte';

	appWindow.setAlwaysOnTop(true);

	// Logic to always resize the content window to the size of the HTML
	afterUpdate(() => {
		setTimeout(async () => {
			let contentRootContainerHeight: number = parseInt(
				window.getComputedStyle(document.body).getPropertyValue('height')
			);
			let contentRootContainerWidth: number = parseInt(
				window.getComputedStyle(document.body).getPropertyValue('width')
			);

			await invoke('cmd_resize_window', {
				windowLabel: 'Content',
				sizeX: +contentRootContainerWidth,
				sizeY: +contentRootContainerHeight
			});

			await invoke('cmd_update_content_position');
		}, 10);
	});

	let bubbleOrientationRight = true;
	const listenToGlobalEvents = async () => {
		await listen('evt-bubble-icon-orientation', (event) => {
			const tauriEvent = event as Event<any>;
			bubbleOrientationRight = tauriEvent.payload.orientation_right;
		});
	};

	listenToGlobalEvents();
</script>

<div class={`flex flex-col ${bubbleOrientationRight ? 'items-end' : ''}`}>
	<div class="max-w-xs align-bottom bg-white rounded-lg p-4 overflow-hidden transform ">
		<OptionsMenu />
		<SearchReplace />
	</div>
	<div class="px-4">
		<BubbleIcon />
	</div>
</div>
