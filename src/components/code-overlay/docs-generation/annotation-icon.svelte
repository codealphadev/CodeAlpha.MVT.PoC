<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import type { ChannelList } from '../../../../src-tauri/bindings/ChannelList';
	import type { EventRuleExecutionState } from '../../../../src-tauri/bindings/rule_execution_state/EventRuleExecutionState';
	import WidgetProcessing from '../../widget/widget-processing.svelte';

	export let show_highlighted = false;

	let is_processing = false; 

	let processing_timeout = 15000; // ms

	const listenTauriEvents = async () => {
		// Listen for rule execution events to determine if the processing icon should be displayed
		await listen('EventRuleExecutionState' as ChannelList, (event) => {
			const ruleExecutionState = JSON.parse(event.payload as string) as EventRuleExecutionState;
			
			switch (ruleExecutionState.event) {
				case 'DocsGenerationStarted':
					is_processing = true;
					setTimeout(async () => {
						is_processing = false;
					}, processing_timeout);
					break;
				case 'DocsGenerationFinished':
					is_processing = false;
					break;
				default:
					break;
			}
		});
	};

	listenTauriEvents();
</script>

<div style="display: flex; align-items: center; height: 100%; width: 100%">
{#if !is_processing}
	{#if show_highlighted}
		<svg
			style="width: 100%"
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 12 12"
		>
			<path
				fill="url(#a)"
				fill-rule="evenodd"
				d="M3.74 3.402a2.591 2.591 0 0 1 1.832.451l.012.008 1.834 1.433L9.04 4.046A3.5 3.5 0 0 0 6.117 2.47c-.916 0-1.751.353-2.376.933Zm4.605 2.616 2.15-1.654-.203-.418a4.646 4.646 0 0 0-4.175-2.617C3.547 1.33 1.469 3.423 1.469 6c0 2.577 2.078 4.67 4.648 4.67a4.645 4.645 0 0 0 4.151-2.569l.21-.417-2.133-1.666Zm-.932.717L5.646 8.093l-.002.002a2.592 2.592 0 0 1-1.903.503c.625.58 1.46.933 2.376.933a3.5 3.5 0 0 0 2.9-1.543L7.413 6.735Zm-.927-.724L4.907 4.777a1.453 1.453 0 0 0-.83-.258c-.808 0-1.47.66-1.47 1.481 0 .82.662 1.48 1.47 1.48.33 0 .634-.108.88-.293l1.53-1.176Z"
				clip-rule="evenodd"
			/>
			<defs>
				<linearGradient
					id="a"
					x1="9.729"
					x2="2.561"
					y1=".999"
					y2="9.48"
					gradientUnits="userSpaceOnUse"
				>
					<stop stop-color="#FF9C64" />
					<stop offset=".782" stop-color="#F84545" />
				</linearGradient>
			</defs>
		</svg>
	{:else}
		<svg
			style="width: 100%"
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 12 12"
		>
			<path
				fill="#000"
				fill-rule="evenodd"
				d="M3.74 3.402a2.591 2.591 0 0 1 1.832.451l.012.008 1.834 1.433L9.04 4.046A3.5 3.5 0 0 0 6.117 2.47c-.916 0-1.751.353-2.376.933Zm4.605 2.616 2.15-1.654-.203-.418a4.646 4.646 0 0 0-4.175-2.617C3.547 1.33 1.469 3.423 1.469 6c0 2.577 2.078 4.67 4.648 4.67a4.645 4.645 0 0 0 4.151-2.569l.21-.417-2.133-1.666Zm-.932.717L5.646 8.093l-.002.002a2.592 2.592 0 0 1-1.903.503c.625.58 1.46.933 2.376.933a3.5 3.5 0 0 0 2.9-1.543L7.413 6.735Zm-.927-.724L4.907 4.777a1.453 1.453 0 0 0-.83-.258c-.808 0-1.47.66-1.47 1.481 0 .82.662 1.48 1.47 1.48.33 0 .634-.108.88-.293l1.53-1.176Z"
				clip-rule="evenodd"
			/>
		</svg>
	{/if}
{:else}
		<WidgetProcessing />
{/if}
</div>