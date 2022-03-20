<script lang="ts">
	import { XIcon } from '@rgossiaux/svelte-heroicons/outline';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import { afterUpdate, onMount } from 'svelte';

	$: close = () => {
		invoke('close_window', { windowLabel: 'Content' });
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

			invoke('resize_content_window', {
				sizeX: +contentRootContainerWidth,
				sizeY: +contentRootContainerHeight
			});
		}, 10);
	});

	let isOpen = true;
</script>

<div
	data-tauri-drag-region
	class="relative border inline-block align-bottom bg-white rounded-lg px-4 pt-5 pb-4 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-sm sm:w-full sm:p-6"
>
	<div class="absolute top-0 right-0 pt-4 pr-4">
		<button
			type="button"
			class="bg-white rounded-md text-gray-400 hover:text-gray-500 "
			on:click={close}
		>
			<XIcon class="h-6 w-6" />
		</button>
	</div>
	<div class="sm:flex sm:items-start">
		<div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left ">
			<h1
				as="h3"
				class="focus:outline-none text-lg leading-6 font-medium text-gray-900 border-transparent focus:border-transparent focus:ring-0"
			>
				Content Window
			</h1>
			<div class="mt-2">
				<p class="text-sm text-gray-500">This is our fancy content window!</p>
			</div>
		</div>
	</div>
	<div class="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse">
		<button
			type="button"
			on:click={close}
			class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:w-auto sm:text-sm"
		>
			Cancel
		</button>
	</div>
</div>
