<script lang="ts">
	import { CogIcon } from "@rgossiaux/svelte-heroicons/outline";
	import { invoke } from "@tauri-apps/api/tauri";
	import { axEventTableStore } from "../event-listener/AXEventsTableStore";

	const { axEventList } = axEventTableStore;

	const resetAXEventList = () => {
		axEventList.set([]);
	};

	const openSettings = () => {
		invoke("open_settings_window");
	};
</script>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
	<div class="md:flex md:items-center md:justify-between">
		<div class="flex-1 min-w-0">
			<h2
				class="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate"
			>
				Accessibility Events
			</h2>
		</div>
		<div class="mt-4 flex md:mt-0 md:ml-4">
			<button
				on:click={resetAXEventList}
				type="button"
				class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
			>
				Reset
			</button>
			<button
				on:click={openSettings}
				type="button"
				class="ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
			>
				<CogIcon class="-ml-1 mr-2 h-5 w-5 text-gray-400" aria-hidden="true" />
				<span>Settings</span>
			</button>
		</div>
	</div>
</div>
