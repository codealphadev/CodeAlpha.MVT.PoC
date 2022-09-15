<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { NodeExplanation } from '../../src-tauri/bindings/features/docs_generation/NodeExplanation';
	import { listen } from '@tauri-apps/api/event';
	import NodeExplainerHeader from '../components/node-explainer/node-explainer-header.svelte';
	import { afterUpdate } from 'svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import type { AppWindow } from '../../src-tauri/bindings/AppWindow';
	import ComplexitySection from '../components/node-explainer/complexity-section.svelte';	
	import type { NodeExplanationEvent } from '../../src-tauri/bindings/features/node_explanation/NodeExplanationEvent';
	import ParametersSection from '../components/node-explainer/parameters-section.svelte';

	let explanation: NodeExplanation | undefined = undefined;
	let complexity: number | null = null;
	let node_name: string | null = null;
	$: summary = explanation ? DOMPurify.sanitize(marked.parse(explanation.summary)) : undefined;

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

	const updateDimensions = async () => {
		let element = document.getElementById(dom_id);

		if (element === null) {
			return;
		}

		let positionInfo = element.getBoundingClientRect();

		window_width = positionInfo.width;
		window_height = positionInfo.height;
	};

	const listenToNodeAnnotationEvents = async () => {
		let node_explanation_channel: ChannelList = 'NodeExplanationEvent';
		console.log('started listening');
		await listen(node_explanation_channel, (event) => {
			console.log(event);
			const { payload, event: event_type } = JSON.parse(
				event.payload as string
			) as NodeExplanationEvent;

			switch (event_type) {
				case 'UpdateNodeExplanation':
					explanation = payload.explanation;
					node_name = payload.name;
					complexity = payload.complexity
					break;
				default:
					break;
			}
		});
	};

	listenToNodeAnnotationEvents();
</script>

{#if explanation !== undefined}
	<div data-tauri-drag-region class="absolute w-full h-full" />
	<div id={dom_id} class="rounded-lg bg-background overflow-hidden">
		<div class="p-4 sm:p-6 gap-2">
			<NodeExplainerHeader kind={explanation.kind} name={node_name} />
			<div class="mt-2 max-w-xl text-sm text-secondary">
				<p>{@html summary}</p>
			</div>
			{#if explanation.parameters}
				<ParametersSection parameters={explanation.parameters}/>
			{/if}
			{#if complexity}
				<ComplexitySection complexity={complexity}/>
			{/if}
		</div>
	</div>
{/if}
