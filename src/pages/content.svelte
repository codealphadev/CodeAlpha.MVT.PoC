<script lang="ts">
	import { afterUpdate } from 'svelte';
	import { Event, listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/tauri';
	import OptionsMenu from '../components/content/options-menu.svelte';
	import BubbleIcon from '../components/content/bubble-icon.svelte';
	import AlertsMain from '../components/content/alerts-main.svelte';
	import { Route, Router } from 'yrv';
	import SearchReplace from '../components/content/search-replace.svelte';

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
</script>

<div class="flex flex-col relative max-w-xs align-bottom transform overflow-hidden">
	<div class="flex flex-col bg-white rounded-md">
		<div class="h-10">
			<OptionsMenu />
		</div>
		<div class="overflow-auto h-[400px] rounded-b-md">
			<Router>
				<Route exact>
					<AlertsMain />
				</Route>
				<Route exact path="/search-replace">
					<SearchReplace />
				</Route>
			</Router>
		</div>
	</div>
	<div class="h-6 mr-4 ml-4 {`${bubbleOrientationRight ? 'ml-auto' : 'ml-4'}`}">
		<BubbleIcon />
	</div>
</div>
