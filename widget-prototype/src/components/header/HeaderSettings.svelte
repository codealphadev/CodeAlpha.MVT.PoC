<script lang="ts">
	import { CogIcon } from '@rgossiaux/svelte-heroicons/outline';
	import { invoke } from '@tauri-apps/api/tauri';
	import { axEventTableStore } from '../event-listener/AXEventsTableStore';

	const { axEventList } = axEventTableStore;

	const resetAXEventList = () => {
		axEventList.set([]);
	};
</script>

<div class="md:flex md:items-center md:justify-between">
	<div class="flex-1 min-w-0">
		<h2 class="text-2xl font-bold leading-7 text-gray-900 sm:text-2xl sm:truncate">Settings</h2>
	</div>
</div>
