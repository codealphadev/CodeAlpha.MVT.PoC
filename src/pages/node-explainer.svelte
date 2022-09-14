<script lang="ts">
  import {marked} from 'marked';
  import DOMPurify from 'dompurify';
  import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
  import type { EventDocsGeneration } from '../../src-tauri/bindings/features/docs_generation/EventDocsGeneration';
  import type { NodeExplanation } from '../../src-tauri/bindings/features/docs_generation/NodeExplanation';
  import { listen } from '@tauri-apps/api/event';
  import NodeExplainerHeader from '../components/node-explainer/node-explainer-header.svelte';

  let explanation: NodeExplanation | undefined = undefined;
  let node_name: string | undefined = undefined;
  $: summary = explanation ? DOMPurify.sanitize(marked.parse(explanation.summary)) : undefined;
  
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
<div class="shadow rounded-lg bg-background">
    <div class="p-4 sm:p-6">
      <NodeExplainerHeader kind={explanation.kind} name={node_name}/>
      <div class="mt-2 max-w-xl text-sm text-secondary">
        <p>{@html summary}</p>
      </div>
    </div>
  </div>
{/if}
