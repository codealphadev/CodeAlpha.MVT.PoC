<script lang="ts">
    import type { SwiftCodeBlockKind } from "../../../src-tauri/bindings/features/node_explanation/SwiftCodeBlockKind";
    export let name: string | null;
    export let kind: SwiftCodeBlockKind;
    export let summary: string;
</script>

<div class="gap-2 flex flex-col w-full">
    <h3 class="text-lg leading-6 text-gray-900 font-medium overflow-hidden text-ellipsis">
        <span class="font-light">{kind}</span> 
        {#if name}
            <span class="font-mono">{name}</span>
        {/if}
    </h3>
    <div class="max-w-xl text-base text-primary">
        <p>{@html summary}</p>
    </div>
</div>