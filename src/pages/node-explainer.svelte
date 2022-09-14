<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventDocsGeneration } from '../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
	import type { NodeExplanation } from '../../src-tauri/bindings/features/docs_generation/NodeExplanation';
	import { listen } from '@tauri-apps/api/event';
	import NodeExplainerHeader from '../components/node-explainer/node-explainer-header.svelte';
	import { afterUpdate } from 'svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import type { AppWindow } from '../../src-tauri/bindings/AppWindow';

	let explanation: NodeExplanation | undefined = undefined;
	let node_name: string | undefined = undefined;
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
		let DocsGenerationChannel: ChannelList = 'EventDocsGeneration';
		console.log('started listening');
		await listen(DocsGenerationChannel, (event) => {
			console.log(event);
			const { payload, event: event_type } = JSON.parse(
				event.payload as string
			) as EventDocsGeneration;

			switch (event_type) {
				case 'NodeExplanationFetched':
					explanation = payload.explanation;
					node_name = payload.name;
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
	<div id={dom_id} class="rounded-lg bg-background">
		<div class="p-4 sm:p-6">
			<NodeExplainerHeader kind={explanation.kind} name={node_name} />
			<div class="mt-2 max-w-xl text-sm text-secondary">
				<p>{@html summary}</p>
			</div>
		</div>
	</div>
{/if}
