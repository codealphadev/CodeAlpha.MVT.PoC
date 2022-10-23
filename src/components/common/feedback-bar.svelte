<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import { ThumbButtonType, ThumbVote } from '../common/button/button';
	import ThumbButton from '../common/button/thumb-button.svelte';

	import type { FeedbackTarget } from '../../../src-tauri/bindings/feedback/FeedbackTarget';

	let thumb_vote: ThumbVote | null = null;

	export let feedback_target: FeedbackTarget;

	const handle_click_thumbs_up = async () => {
		if (thumb_vote) {
			return;
		}
		await invoke('cmd_send_feedback', {
			target: feedback_target,
			feedback: ThumbVote.Good
		});
		thumb_vote = ThumbVote.Good;
	};

	const handle_click_thumbs_down = async () => {
		if (thumb_vote) {
			return;
		}
		await invoke('cmd_send_feedback', {
			target: feedback_target,
			feedback: ThumbVote.Bad
		});
		await appWindow.hide();
		thumb_vote = ThumbVote.Bad;
	};
</script>

<div class="flex gap-3 px-1 items-center">
	<ThumbButton type={ThumbButtonType.Up} {thumb_vote} on:click={handle_click_thumbs_up} />
	<ThumbButton type={ThumbButtonType.Down} {thumb_vote} on:click={handle_click_thumbs_down} />
	<div class="text-secondary text-xs">
		{#if thumb_vote}
			Thanks for the feedback!
		{:else}
			Honestly, was this helpful?
		{/if}
	</div>
</div>
