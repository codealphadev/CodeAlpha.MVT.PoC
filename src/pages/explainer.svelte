<script lang="ts">
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { NodeExplanation } from '../../src-tauri/bindings/features/node_explanation/NodeExplanation';
	import { listen } from '@tauri-apps/api/event';
	import Header from '../components/node-explainer/header-section.svelte';
	import { afterUpdate } from 'svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import type { AppWindow } from '../../src-tauri/bindings/AppWindow';
	import ComplexitySection from '../components/node-explainer/complexity-section.svelte';
	import type { NodeExplanationEvent } from '../../src-tauri/bindings/features/node_explanation/NodeExplanationEvent';
	import ParametersSection from '../components/node-explainer/parameters-section.svelte';
	import Footer from '../components/node-explainer/footer-section.svelte';
	import Window from '../components/common/window.svelte';

	let explanation: NodeExplanation | null = null;
	let complexity: number | null = null;
	let node_name: string | null = null;

	let dom_id = 'explain-window-container';

	let window_width: number | null = null;
	let window_height: number | null = null;

	// Logic to always resize the content window to the size of the HTML
	afterUpdate(() => {
		updateDimensions();

		if (window_width && window_height) {
			let appWindow: AppWindow = 'Explain';

			invoke('cmd_resize_window', {
				appWindow: appWindow,
				sizeY: window_height,
				sizeX: window_width
			});
		}
	});

	const updateDimensions = () => {
		let element = document.getElementById(dom_id);

		if (element === null) {
			return;
		}

		let positionInfo = element.getBoundingClientRect();

		window_width = positionInfo.width;
		window_height = positionInfo.height;
	};

	const listen_to_node_explanation_events = async () => {
		let node_explanation_channel: ChannelList = 'NodeExplanationEvent';
		await listen(node_explanation_channel, (event) => {
			const { payload, event: event_type } = JSON.parse(
				event.payload as string
			) as NodeExplanationEvent;

			switch (event_type) {
				case 'CloseNodeExplanationWindow':
					explanation = null;
					node_name = null;
					complexity = null;
					break;
				case 'UpdateNodeExplanation':
					explanation = payload.explanation;
					node_name = payload.name;
					complexity = payload.complexity;
					break;
				default:
					break;
			}
		});
	};

	listen_to_node_explanation_events();
</script>

{#key JSON.stringify(explanation)}
	{#if explanation !== null}
		<Window id={dom_id}>
			<div data-tauri-drag-region class=" absolute w-full h-20" />
			<Header kind={explanation.kind} name={node_name} summary={explanation.summary} />

			{#if explanation.parameters}
				<ParametersSection parameters={explanation.parameters} />
			{/if}
			{#if complexity !== null}
				<ComplexitySection {complexity} />
			{/if}
			<hr class="border-background_secondary w-full" />
			<Footer />
		</Window>
	{/if}
{/key}
