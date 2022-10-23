<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';
	import Button from '../common/button/button.svelte';
	import { ButtonType } from '../common/button/button';
	import FeedbackBar from '../common/feedback-bar.svelte';

	const paste_docs = async () => {
		await invoke('cmd_paste_docs');
		await appWindow.hide();
	};
</script>

<div class="flex justify-between w-full items-center">
	<FeedbackBar feedback_target={'NodeExplainer'} />

	<Button type={ButtonType.Primary} on:click={paste_docs}>Insert as Docstring</Button>
</div>
