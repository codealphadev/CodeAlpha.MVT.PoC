<script lang="ts">
	import { colors } from '../../../themes';
	import { ThumbButtonType, ThumbVote } from './button';

	export let type: ThumbButtonType;
	export let thumb_vote: ThumbVote | null;

	$: activated =
		(type === ThumbButtonType.Up && thumb_vote === ThumbVote.Good) ||
		(type === ThumbButtonType.Down && thumb_vote === ThumbVote.Bad);
</script>

<button
	on:click
	type="button"
	class="thumb-button {type === ThumbButtonType.Down ? 'rotate-180' : undefined}"
>
	<svg width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
		{#if activated}
			<rect width="28" height="28" rx="14" fill={colors.primary} />
			<path
				d="M15.3333 12.6667H18.5093C19.5005 12.6667 20.1451 13.7098 19.7019 14.5963L17.3685 19.263C17.1427 19.7147 16.681 20 16.176 20H13.4975C13.3885 20 13.2799 19.9866 13.1741 19.9602L10.6667 19.3333M15.3333 12.6667V9.33333C15.3333 8.59695 14.7364 8 14 8H13.9364C13.6033 8 13.3333 8.26998 13.3333 8.60302C13.3333 9.07922 13.1924 9.54477 12.9282 9.94099L10.6667 13.3333V19.3333M15.3333 12.6667H14M10.6667 19.3333H9.33333C8.59695 19.3333 8 18.7364 8 18V14C8 13.2636 8.59695 12.6667 9.33333 12.6667H11"
				stroke={colors.background_secondary}
				stroke-width="1.33"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		{:else}
			<path
				d="M15.3333 12.6667H18.5093C19.5005 12.6667 20.1451 13.7098 19.7019 14.5963L17.3685 19.263C17.1427 19.7147 16.681 20 16.176 20H13.4975C13.3885 20 13.2799 19.9866 13.1741 19.9602L10.6667 19.3333M15.3333 12.6667V9.33333C15.3333 8.59695 14.7364 8 14 8H13.9364C13.6033 8 13.3333 8.26998 13.3333 8.60302C13.3333 9.07922 13.1924 9.54477 12.9282 9.94099L10.6667 13.3333V19.3333M15.3333 12.6667H14M10.6667 19.3333H9.33333C8.59695 19.3333 8 18.7364 8 18V14C8 13.2636 8.59695 12.6667 9.33333 12.6667H11"
				stroke={colors.primary}
				stroke-width="1.33"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
			<rect
				x="0.75"
				y="0.75"
				width="26.5"
				height="26.5"
				rx="13.25"
				stroke={colors.primary}
				stroke-width="1.5"
			/>
		{/if}
	</svg>
</button>
