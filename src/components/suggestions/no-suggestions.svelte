<script lang="ts">
	import { fade } from 'svelte/transition';
	import Caption from '../common/typography/caption.svelte';
	import H2 from '../common/typography/h2.svelte';
	import NoSuggestionsFoundIcon from './icons/no-suggestions-found.svelte';
</script>

<div
	class="row-[1/2] col-[1/2] flex flex-col items-center justify-center gap-1 h-[136px] mt-9"
	transition:fade={{ duration: 100 }}
>
	<NoSuggestionsFoundIcon />
	<H2>Pretty clean here!</H2>
	<Caption>We couldn't spot anything. Happy coding!</Caption>
</div>
