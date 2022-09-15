<script lang="ts">
    import DOMPurify from 'dompurify';
    import { marked } from 'marked';
	import type { FunctionParameterWithExplanation } from '../../../src-tauri/bindings/features/node_explanation/FunctionParameterWithExplanation';

    export let parameters: FunctionParameterWithExplanation[];
</script>

<ul class="gap-2 text-sm flex flex-col">
    {#each parameters as parameter}
    <li>
        <span class="font-mono text-contrast">{parameter.name}</span>
        <span class="font-mono text-secondary ml-3">{parameter.param_type}</span>
        <span class="text-primary mt-[2px]">{@html DOMPurify.sanitize(marked.parse(parameter.explanation))}</span>
    </li>
    {/each}
</ul>
