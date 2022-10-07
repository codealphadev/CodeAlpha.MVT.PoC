<script>
	import Card from '../common/card.svelte';
	import Caption from '../common/typography/caption.svelte';
	import H2 from '../common/typography/h2.svelte';
	import NoSuggestionsFoundIcon from './icons/no-suggestions-found.svelte';
</script>

<Card additional_class="items-center justify-center h-36">
	<NoSuggestionsFoundIcon />

	<div class="flex flex-col items-center justify-center gap-1">
		<H2>Pretty clean here!</H2>
		<Caption>We couldn't spot anything. Happy coding!</Caption>
	</div>
</Card>
