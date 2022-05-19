<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';

	import { afterUpdate } from 'svelte';

	import { HighlightAuto, HighlightSvelte } from 'svelte-highlight';
	import github from 'svelte-highlight/src/styles/github';
	import BubbleIconUpsidedown from '../components/content/bubble-icon-upsidedown.svelte';
	import BubbleIcon from '../components/content/bubble-icon.svelte';
	import RepairContent from '../components/repair/repair-content.svelte';
	import { fly } from 'svelte/transition';
	import { onMount } from 'svelte';
	import LoadingSpinner from '../components/repair/loading-spinner.svelte';
	import { CogIcon } from '@rgossiaux/svelte-heroicons/outline';
	import X from '@rgossiaux/svelte-heroicons/outline/X';
	import { appWindow, getAll, WebviewWindow } from '@tauri-apps/api/window';
	import { Event, listen } from '@tauri-apps/api/event';

	// Logic to always resize the content window to the size of the HTML
	afterUpdate(() => {
		setTimeout(async () => {
			let contentRootContainerHeight: number = parseInt(
				window.getComputedStyle(document.body).getPropertyValue('height')
			);
			let contentRootContainerWidth: number = parseInt(
				window.getComputedStyle(document.body).getPropertyValue('width')
			);

			await invoke('cmd_resize_repair_window', {
				sizeX: +contentRootContainerWidth,
				sizeY: +contentRootContainerHeight
			});
		}, 10);
	});

	// Making sure loading spinner is shown before content is loaded
	let content_loaded = false;
	$: close_window = () => {
		content_loaded = false;
		setTimeout(async () => {
			appWindow.hide();
		}, 30);
	};
	const listenToGlobalEvents = async () => {
		await listen('evt-repair-opened', (event) => {
			const tauriEvent = event as Event<any>;

			content_loaded = false;
			setTimeout(async () => {
				content_loaded = true;
			}, 3000);
		});
	};
	listenToGlobalEvents();
</script>

<div class="flex flex-col">
	<div data-tauri-drag-region class="px-4">
		<BubbleIconUpsidedown />
	</div>
	<div class="max-w-lg p-4 align-bottom bg-white rounded-lg ">
		{#if content_loaded === true}
			<div in:fly>
				<div as="div" class="relative">
					<button as="div" on:click={close_window} class="absolute -top-3 -right-3 outline-none">
						<button type="button" class="bg-white rounded-md text-gray-200 hover:text-gray-500 ">
							<X class="h-5 w-5" />
						</button>
					</button>
				</div>
				<RepairContent />
			</div>
		{:else}
			<div in:fly>
				<div as="div" class="relative">
					<button as="div" on:click={close_window} class="absolute -top-3 -right-3 outline-none">
						<button type="button" class="bg-white rounded-md text-gray-200 hover:text-gray-500 ">
							<X class="h-5 w-5" />
						</button>
					</button>
				</div>
				<LoadingSpinner />
			</div>
		{/if}
	</div>
</div>
