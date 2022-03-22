<script lang="ts">
	import WidgetIcon from '../components/widget/widget-icon.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow, getAll, WebviewWindow } from '@tauri-apps/api/window';

	appWindow.setAlwaysOnTop(true);

	// To prevent opening/closing the dialog when the user DRAGS the widget icon
	// I introduce a threshold time to wait before opening the dialog after stoping the drag
	let timeing_threshold = 500;
	let last_drag: number = 0;

	const toggleContent = () => {
		if (performance.now() - last_drag > timeing_threshold) {
			invoke('toggle_window', { windowLabel: 'Content' });
		} else {
			// Case: "Ghost Click" after dragging the widget
			if (contentOpenBeforeMove) {
				invoke('open_window', { windowLabel: 'Content' });
				contentOpenBeforeMove = false;
			}
		}
	};

	let contentOpenBeforeMove = false;

	appWindow.listen('tauri://move', ({ event, payload }) => {
		setTimeout(() => {
			last_drag = performance.now();
		}, 10);

		// Hide the content window if the user is moving the widget
		let contentWindow = WebviewWindow.getByLabel('Content');

		if (contentWindow) {
			if (contentWindow.isVisible()) {
				contentOpenBeforeMove = true;
				invoke('close_window', { windowLabel: 'Content' });
			} else {
				contentOpenBeforeMove = false;
			}
		}
	});
</script>

<div class="relative">
	<WidgetIcon />
	<div
		data-tauri-drag-region
		on:click={toggleContent}
		class="absolute bottom-0 right-0 w-12 h-12"
	/>
</div>
