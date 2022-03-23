<script lang="ts">
	import { XIcon } from '@rgossiaux/svelte-heroicons/outline';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import { afterUpdate, onMount } from 'svelte';
	import SearchReplace from '../components/content/search-replace.svelte';
	import OptionsMenu from '../components/content/options-menu.svelte';
	import BubbleIcon from '../components/content/bubble-icon.svelte';

	$: close = () => {
		invoke('cmd_close_window', { windowLabel: 'Content' });
	};

	appWindow.setAlwaysOnTop(true);

	afterUpdate(() => {
		setTimeout(() => {
			let contentRootContainerHeight: number = parseInt(
				window.getComputedStyle(document.body).getPropertyValue('height')
			);
			let contentRootContainerWidth: number = parseInt(
				window.getComputedStyle(document.body).getPropertyValue('width')
			);

			console.log(contentRootContainerHeight);
			console.log(contentRootContainerHeight);

			invoke('cmd_resize_window', {
				windowLabel: 'Content',
				sizeX: +contentRootContainerWidth,
				sizeY: +contentRootContainerHeight
			});
		}, 10);
	});

	let isOpen = true;
</script>

<div class="flex flex-col items-end">
	<div
		class="w-full inline-block align-bottom bg-white rounded-lg px-4 pt-5 pb-4 text-left overflow-hidden transform transition-all sm:align-middle sm:max-w-sm sm:w-full sm:p-6"
	>
		<OptionsMenu />
		<SearchReplace />
	</div>
	<div class="pr-4">
		<BubbleIcon />
	</div>
</div>
