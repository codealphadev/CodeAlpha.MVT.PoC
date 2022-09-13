<script lang="ts">
    import {marked} from 'marked';
    import DOMPurify from 'dompurify';
    export let summary_markdown: string;
    export let kind: SwiftNodeType;
    export let name: string;
    export let node_explanation: Explanation    
    const summary = DOMPurify.sanitize(marked.parse(summary_markdown));


</script>

<div class="shadow rounded-lg bg-background">
    <div class="p-4 sm:p-6">
      <h3 class="text-lg leading-6 text-gray-900 font-medium" >
        <span class="font-light">Class</span> <span class="font-mono">UIElement</span>
    </h3>
      <div class="mt-2 max-w-xl text-sm text-gray-500">
        <p>{@html summary}</p>
      </div>
      <div class="mt-5">
        <button type="button" class="inline-flex items-center justify-center rounded-md border border-transparent bg-red-100 px-4 py-2 font-medium text-red-700 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 sm:text-sm">Add docstring</button>
      </div>
    </div>
  </div>