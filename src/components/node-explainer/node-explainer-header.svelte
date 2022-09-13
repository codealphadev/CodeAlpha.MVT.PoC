<script lang="ts">
    import type { SwiftCodeBlockKind } from "../../../src-tauri/bindings/features/docs_generation/SwiftCodeBlockKind";
    export let name: string | undefined;
    export let kind: SwiftCodeBlockKind;
</script>

<h3 class="text-lg leading-6 text-gray-900 font-medium" >
    <span class="font-light">{kind}</span> 
    {#if name}
        <span class="font-mono">{name}</span>
    {/if}
</h3>