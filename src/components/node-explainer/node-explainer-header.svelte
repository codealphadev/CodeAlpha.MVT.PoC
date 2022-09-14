<script lang="ts">
    import type { SwiftCodeBlockKind } from "../../../src-tauri/bindings/features/docs_generation/SwiftCodeBlockKind";
    export let name: string | null;
    export let kind: SwiftCodeBlockKind;
</script>

<h3 class="text-lg leading-6 text-gray-900 font-medium overflow-hidden text-ellipsis">
    <span class="font-light">{kind}</span> 
    {#if name}
        <span class="font-mono">{name}</span>
    {/if}
</h3>