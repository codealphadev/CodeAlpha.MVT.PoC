<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import WidgetBackground from '../components/widget/widget-background.svelte';
	import IconLogo from '../components/widget/icons/icon-logo.svelte';
	import IconSwiftFormat from '../components/widget/icons/icon-swift-format.svelte';
	import IconDocsGen from '../components/widget/icons/icon-docs-gen.svelte';
	import IconLogoGreyscale from '../components/widget/icons/icon-logo-greyscale.svelte';
	import { listen, Event } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventRuleExecutionState } from '../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import WidgetProcessing from '../components/widget/widget-processing.svelte';
	import WidgetBackgroundGreyscale from '../components/widget/widget-background-greyscale.svelte';

	let app_active = true;
	let ruleExecutionState: EventRuleExecutionState | null = null;
	let ghostClickAlreadyHappened = true;

	const clickAction = async () => {
		if (ghostClickAlreadyHappened) {
			app_active = !app_active;
			await invoke('cmd_toggle_app_activation', { appActive: app_active });
		} else {
			// Case "Ghostclick happened"
			ghostClickAlreadyHappened = true;
		}
	};

	appWindow.listen('tauri://move', async ({ event, payload }) => {
		ghostClickAlreadyHappened = false;
	});

	const listenTauriEvents = async () => {
		await listen('EventRuleExecutionState' as ChannelList, (event) => {
			ruleExecutionState = JSON.parse(event.payload as string) as EventRuleExecutionState;

			// In case we receive a "finished" event, register a timeout to reset the
			// widget icon to the logo after some delay
			switch (ruleExecutionState.event) {
				case 'SwiftFormatFinished':
					console.log('SwiftFormatFinished');
					setTimeout(async () => {
						ruleExecutionState = null;
					}, 500);
					break;
				case 'DocsGenerationFinished':
					console.log('DocsGenerationFinished');
					setTimeout(async () => {
						ruleExecutionState = null;
					}, 500);
					break;
				default:
					console.log('Default');
					break;
			}
		});
	};

	listenTauriEvents();
</script>

<div class="relative">
	{#if app_active === false}
		<WidgetBackgroundGreyscale />
	{:else}
		<WidgetBackground />
	{/if}

	<div class="absolute bottom-0 right-0 w-12 h-12">
		<div
			data-tauri-drag-region
			on:click={clickAction}
			class="flex items-center justify-center h-screen"
		>
			<div class="w-[36px]">
				{#if app_active === false}
					<IconLogoGreyscale />
				{:else if ruleExecutionState != null && ruleExecutionState.event === 'SwiftFormatFinished'}
					<IconSwiftFormat />
				{:else if ruleExecutionState != null && ruleExecutionState.event === 'DocsGenerationFinished'}
					<IconDocsGen />
				{:else if ruleExecutionState != null && ruleExecutionState.event === 'DocsGenerationStarted'}
					<WidgetProcessing />
				{:else}
					<IconLogo />
				{/if}
			</div>
		</div>
	</div>
	<div data-tauri-drag-region on:click={clickAction} class="absolute bottom-0 right-0 w-12 h-12" />
</div>
