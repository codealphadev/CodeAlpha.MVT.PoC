<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import Button from '../common/button/button.svelte';
	import { ButtonType, ThumbButtonType, ThumbVote } from '../common/button/button';
	import ThumbButton from '../common/button/thumb-button.svelte';

	let thumb_vote: ThumbVote | null = null;

	const paste_docs = async () => {
		await invoke('cmd_paste_docs');
		await appWindow.hide();
	};

	const handle_click_thumbs_up = async () => {
		if (thumb_vote) {
			return;
		}
		await invoke('cmd_send_feedback', {
			feature: 'NodeExplainer',
			feedback: ThumbVote.Good
		});
		thumb_vote = ThumbVote.Good;
	};

	const handle_click_thumbs_down = async () => {
		if (thumb_vote) {
			return;
		}
		await invoke('cmd_send_feedback', {
			feature: 'NodeExplainer',
			feedback: ThumbVote.Bad
		});
		await appWindow.hide();
		thumb_vote = ThumbVote.Bad;
	};
</script>

<div class="flex justify-between w-full items-center">
	<div class="flex gap-3 px-1 items-center">
		<ThumbButton type={ThumbButtonType.Up} {thumb_vote} on:click={handle_click_thumbs_up} />
		<ThumbButton type={ThumbButtonType.Down} {thumb_vote} on:click={handle_click_thumbs_down} />
	</div>
	<Button type={ButtonType.Primary} on:click={paste_docs}>Insert as Docstring</Button>
</div>
