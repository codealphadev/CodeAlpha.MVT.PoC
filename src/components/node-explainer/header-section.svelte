<script lang="ts">
    import type { SwiftCodeBlockKind } from "../../../src-tauri/bindings/features/node_explanation/SwiftCodeBlockKind";
    import IconNodeExplainer from './icons/icon-explainer.svelte';
    
    export let name: string | null;
    export let kind: SwiftCodeBlockKind;
    export let summary: string;
</script>

<div class="gap-2 flex flex-col w-full">
    <div class="flex justify-between items-start">
        <h3 class="text-base leading-6 text-gray-900 font-medium overflow-hidden text-ellipsis">
            <span class="font-light">{kind}</span> 
            {#if name}
                <span class="font-mono font-bold">{name}</span>
            {/if}
        </h3>
        <IconNodeExplainer/>
    </div>
    <div class="max-w-xl text-sm text-primary font-normal">
        <p>{@html summary}</p>
    </div>
</div>