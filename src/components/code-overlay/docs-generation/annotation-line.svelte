<script lang="ts">
	import { CodeAlphaOrange, NeutralGrey } from '../../../theme';

	export let visible = false;
	export let highlighted = false;
	const ANNOTATION_LINE_WIDTH_PCT = 10;

	export let height = 0;

	$: background_color = highlighted ? CodeAlphaOrange : NeutralGrey;
</script>

{#if visible}
	<div
		class="annotation-line"
		style="width: {ANNOTATION_LINE_WIDTH_PCT}%; top: 100%; left: {50 -
			ANNOTATION_LINE_WIDTH_PCT /
				2}%; height: {height}px; position: absolute; background-color: {background_color};"
	/>
{/if}

<style>
	@keyframes reveal {
		from {
			clip-path: inset(0% 0% 100% 0%);
		}
		to {
			clip-path: inset(0% 0% 0% 0%);
		}
	}

	.annotation-line {
		animation: reveal 0.13s ease-out forwards;
	}
</style>
