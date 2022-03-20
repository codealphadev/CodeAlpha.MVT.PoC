<script lang="ts">
	import WidgetIcon from '../components/widget/widget-icon.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow, LogicalSize } from '@tauri-apps/api/window';
	import Content from './content.svelte';
	import { afterUpdate, onMount } from 'svelte';
	import TestPopover from '../components/widget/test-popover.svelte';

	// To prevent opening/closing the dialog when the user DRAGS the widget icon
	// I introduce a threshold time to wait before opening the dialog after stoping the drag
	let timeing_threshold = 500;
	let last_drag: number = 0;

	const toggleContent = () => {
		if (performance.now() - last_drag > timeing_threshold) {
			show_content = !show_content;
			// invoke('toggle_window', { windowLabel: 'Content' });
		}
	};

	appWindow.listen('tauri://move', ({ event, payload }) => {
		setTimeout(() => {
			last_drag = performance.now();
		}, 10);
	});

	// onMount(() => {
	// 	appWindow.setSize(new LogicalSize(64, 64));
	// });

	// afterUpdate(() => {
	// 	let newHeight = window.getComputedStyle(document.body).getPropertyValue('height');
	// 	let newWidth = window.getComputedStyle(document.body).getPropertyValue('width');

	// 	console.log(newHeight, newWidth);

	// 	appWindow.setSize(new LogicalSize(parseFloat(newWidth), parseFloat(newHeight)));
	// });

	let show_content = false;
</script>

<TestPopover />
<!-- <div>
	<div data-tauri-drag-region on:click={toggleContent} class="w-16 h-16 flex flex-col">
		<WidgetIcon />
		{#if show_content}
			<Content />
		{/if}
	</div>
</div> -->
