<script lang="ts">
	import WidgetIcon from '../components/widget/widget-icon.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import WidgetBackground from '../components/widget/widget-background.svelte';
	import IconLogo from '../components/widget/icons/icon-logo.svelte';
	import IconSwiftFormat from '../components/widget/icons/icon-swift-format.svelte';
	import IconDocsGen from '../components/widget/icons/icon-docs-gen.svelte';
	import IconLogoGreyscale from '../components/widget/icons/icon-logo-greyscale.svelte';

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
	<WidgetBackground />
	<div class="absolute bottom-0 right-0 w-12 h-12">
		<div
			data-tauri-drag-region
			on:click={clickAction}
			class="flex items-center justify-center h-screen"
		>
			<div class="w-[36px]">
				<IconLogo />
			</div>
		</div>
	</div>
	<div data-tauri-drag-region on:click={clickAction} class="absolute bottom-0 right-0 w-12 h-12" />
</div>
