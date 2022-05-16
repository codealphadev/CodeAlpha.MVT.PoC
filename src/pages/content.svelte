<script lang="ts">
	import { afterUpdate } from 'svelte';
	import { Event, listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import SearchReplace from '../components/content/search-replace.svelte';
	import OptionsMenu from '../components/content/options-menu.svelte';
	import BubbleIcon from '../components/content/bubble-icon.svelte';
	import Welcome from '../components/content/welcome.svelte';
	import Documentation from '../components/content/documentation.svelte';
	import Tutorial from '../components/content/tutorial.svelte';
	import { fly } from 'svelte/transition';

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

			await invoke('cmd_resize_content_window', {
				sizeX: +contentRootContainerWidth,
				sizeY: +contentRootContainerHeight
			});
		}, 10);
	});

	let bubbleOrientationRight = true;
	const listenToGlobalEvents = async () => {
		await listen('evt-content-window-orientation', (event) => {
			const tauriEvent = event as Event<any>;
			bubbleOrientationRight = tauriEvent.payload.orientation_right;
		});
	};

	listenToGlobalEvents();

	let componentToShow = 1;

	$: setComponentToShowWelcome = () => {
		componentToShow = 1;
	};
</script>

<div class={`flex flex-col ${bubbleOrientationRight ? 'items-end' : ''}`}>
	<div class="max-w-xs align-bottom bg-white rounded-lg overflow-hidden transform ">
		<div class="p-4 ">
			<OptionsMenu bind:componentToShow />

			<div class="mt-1 ml-2">
				<img
					class="inline-block h-14 w-14 rounded-full ring-2 ring-gray-400"
					src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
					alt=""
				/>
			</div>
			{#if componentToShow === 1}
				<div in:fly>
					<Welcome bind:componentToShow />
				</div>
			{:else if componentToShow === 2}
				<div in:fly>
					<Documentation />
				</div>
			{:else if componentToShow === 3}
				<div in:fly>
					<Tutorial />
				</div>
			{:else if componentToShow === 4}
				<div in:fly>
					<SearchReplace />
				</div>
			{:else}
				<div in:fly>
					<Welcome bind:componentToShow />
				</div>
			{/if}
		</div>
		{#if componentToShow !== 1}
			<div class="w-full border-t border-gray-200" />
			<div class="flex py-5 flex-wrap justify-center">
				<button
					type="button"
					on:click={setComponentToShowWelcome}
					class="inline-flex items-center px-6 py-3 border border-gray-300 shadow-sm text-base font-medium rounded-md text-gray-400 hover:text-gray-700 bg-gray-white hover:bg-gray-200 active:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="-ml-1 mr-3 h-5 w-5"
						viewBox="0 0 20 20"
						fill="currentColor"
					>
						<path
							fill-rule="evenodd"
							d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z"
							clip-rule="evenodd"
						/>
					</svg>
					Go back</button
				>
			</div>
		{/if}
	</div>
	<div class="px-4">
		<BubbleIcon />
	</div>
</div>
