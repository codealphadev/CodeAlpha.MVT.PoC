<script lang="ts">
    import DOMPurify from 'dompurify';
    import { marked } from 'marked';
	import type { FunctionParameterWithExplanation } from '../../../src-tauri/bindings/features/node_explanation/FunctionParameterWithExplanation';

    export let parameters: FunctionParameterWithExplanation[];
</script>

<ul class="gap-2 text-xs flex flex-col">
    {#each parameters as parameter}
    <li>
        <span class="font-mono text-contrast font-medium">{parameter.name}</span>
        <span class="  text-tras font-mono text-secondary ml-3 font-medium">{parameter.param_type}</span>
        <span class="gradient-text clip mt-[2px] font-normal">{@html DOMPurify.sanitize(marked.parse(parameter.explanation))}</span>
    </li>
    {/each}
</ul>
