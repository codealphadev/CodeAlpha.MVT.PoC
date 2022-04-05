<script lang="ts">
	import WidgetIcon from '../components/widget/widget-icon.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow, getAll, WebviewWindow } from '@tauri-apps/api/window';

	appWindow.setAlwaysOnTop(true);

	let ghostClickAlreadyHappened = true;

	const clickAction = async () => {
		if (ghostClickAlreadyHappened) {
			// 0.	Update content window position
			// await invoke('cmd_update_content_position');
			// 1. Toggle the content window
			await invoke('cmd_toggle_window', { windowLabel: 'Content' });
		} else {
			// Case "Ghostclick happened"
			ghostClickAlreadyHappened = true;
		}
	};

	appWindow.listen('tauri://move', async ({ event, payload }) => {
		ghostClickAlreadyHappened = false;
	});
</script>

<div class="relative">
	<WidgetIcon />
	<div data-tauri-drag-region on:click={clickAction} class="absolute bottom-0 right-0 w-12 h-12" />
</div>
