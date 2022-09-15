<script lang="ts">
    import DOMPurify from 'dompurify';
    import { marked } from 'marked';
	import type { FunctionParameter } from '../../../src-tauri/bindings/features/node_explanation/FunctionParameter';

    export let parameters: FunctionParameter[];
</script>

<ul>
    {#each parameters as parameter}
    <li class="py-2 text-sm">
    <span class="font-mono">{parameter.name}</span>
    <span class="text-secondary">
    {@html DOMPurify.sanitize(marked.parse(parameter.explanation))}</span>
    </li>
    {/each}
</ul>
