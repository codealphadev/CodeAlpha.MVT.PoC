<script lang="ts">
	import type { SwiftCodeBlockKind } from '../../../src-tauri/bindings/features/node_explanation/SwiftCodeBlockKind';
	import IconExplainer from './icons/icon-explainer.svelte';
	import { annotate } from 'rough-notation';
	import { onMount } from 'svelte';

	export let name: string | null;
	export let kind: SwiftCodeBlockKind;
	export let summary: string;
	export let name_suggestion: string | null;

	function map_code_block_kind_to_text(kind: SwiftCodeBlockKind): string {
		if (kind == 'Function' || kind == 'Class') {
			return kind;
		} else {
			return `${kind} statement`;
		}
	}

	onMount(() => {
		if (name_suggestion !== null) {
			const element = document.getElementById('node-explainer-name');
			if (!element) {
				return;
			}
			const annotation = annotate(element, {
				type: 'highlight',
				animationDuration: 40,
				color: '#ff44ff'
			});
			annotation.show();
		}
	});
</script>

<div class="gap-2 flex flex-col w-full text-primary">
	<div class="flex justify-between items-start">
		<h3 class="text-sm leading-6 font-medium overflow-hidden text-ellipsis">
			<span class="text-[15px]">{map_code_block_kind_to_text(kind)}</span>
			{#if name}
				<span id="node-explainer-name" class="font-mono font-bold">{name}</span>
			{/if}
		</h3>
		<IconExplainer />
	</div>
	<div class="max-w-xl text-sm font-normal">
		<p>{@html summary}</p>
	</div>
</div>
