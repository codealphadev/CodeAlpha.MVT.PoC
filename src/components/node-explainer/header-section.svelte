<script lang="ts">
    import type { SwiftCodeBlockKind } from "../../../src-tauri/bindings/features/node_explanation/SwiftCodeBlockKind";
	import { gradient_text_tailwind } from "../../themes";
    import IconNodeExplainer from './icons/icon-explainer.svelte';
    
    export let name: string | null;
    export let kind: SwiftCodeBlockKind;
    export let summary: string;

    function map_code_block_kind_to_text(kind: SwiftCodeBlockKind): string {
        if (kind == "Function" || kind == "Class") {
            return kind;
        } else {
            return `${kind} statement`;
        }

    }
</script>

<div class="gap-2 flex flex-col w-full">
    <div class="flex justify-between items-start">
        <h3 class="text-base leading-6 text-gray-900 font-medium overflow-hidden text-ellipsis">
            <span class="font-light">{map_code_block_kind_to_text(kind)}</span> 
            {#if name}
                <span class="font-mono font-bold">{name}</span>
            {/if}
        </h3>
        <IconNodeExplainer/>
    </div>
    <div class="max-w-xl text-sm font-normal {gradient_text_tailwind}">
        <p>{@html summary}</p>
    </div>
</div>