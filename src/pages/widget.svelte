<script lang="ts">
	import WidgetIcon from '../components/widget/widget-icon.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';

	let ghostClickAlreadyHappened = true;

	const clickAction = async () => {
		if (ghostClickAlreadyHappened) {
			await invoke('cmd_toggle_content_window');
		} else {
			// Case "Ghostclick happened"
			ghostClickAlreadyHappened = true;
		}
	};

	const isContentVisible = async (): Promise<boolean> => {
		return await invoke('cmd_is_window_visible', { windowType: 'Content' });
	};

	appWindow.listen('tauri://move', async ({ event, payload }) => {
		ghostClickAlreadyHappened = false;
	});
</script>

<div class="relative">
	<WidgetIcon />
	<div data-tauri-drag-region on:click={clickAction} class="absolute bottom-0 right-0 w-12 h-12" />
</div>
