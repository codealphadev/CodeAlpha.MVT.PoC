<script lang="ts">
	import { SwitchHorizontalIcon } from '@rgossiaux/svelte-heroicons/outline';
	import { invoke } from '@tauri-apps/api/tauri';
</script>

<form class="space-y-3">
	<div class="mx-2">
		<h3 class="text-lg leading-6 font-medium text-gray-900 pt-4 outline-none">
			Ok, let me help you here!
		</h3>
		<p class="mt-2 text-base font-normal text-gray-500">
			Select a block of code in your editor. I'll explain you what it does.
		</p>
	</div>
</form>
