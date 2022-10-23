<script lang="ts">
	import InlineSuggestion from '../common/inline-suggestion.svelte';

	export let suggested_name: string;
</script>

<InlineSuggestion>
	Consider renaming to <span class="font-mono text-contrast font-medium">{suggested_name}.</span>
</InlineSuggestion>
