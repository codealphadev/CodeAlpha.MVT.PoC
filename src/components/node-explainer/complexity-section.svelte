<script lang="ts">
  import InlineSuggestion from '../common/inline-suggestion.svelte';

  export let complexity: number;
</script>

{#if complexity > 15}
  <InlineSuggestion>This function may be too complex. Consider refactoring.</InlineSuggestion>
{/if}
