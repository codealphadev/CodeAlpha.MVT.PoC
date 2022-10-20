<script lang="ts">
	import { fade } from 'svelte/transition';
	import H2 from '../common/typography/h2.svelte';
</script>

<div
	class="row-[1/2] col-[1/2] flex  flex-col items-center justify-center gap-1 h-[136px] mt-9"
	transition:fade={{ duration: 100 }}
>
	<!-- Positioning for consistency with NoSuggestions -->
	<div class="h-6 w-6" />
	<H2>Checking for suggestions…</H2>
	<div class="h-4 w-6" />
</div>
