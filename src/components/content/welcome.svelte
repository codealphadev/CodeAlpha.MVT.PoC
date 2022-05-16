<script lang="ts">
	import { SwitchHorizontalIcon } from '@rgossiaux/svelte-heroicons/outline';
	import { invoke } from '@tauri-apps/api/tauri';

	export let componentToShow = 0;

	$: setComponentToShowDocumentation = () => {
		componentToShow = 2;
	};

	$: setComponentToShowTutorial = () => {
		componentToShow = 3;
	};
</script>

<form class="space-y-3">
	<div class="mx-2">
		<h3 class="text-lg leading-6 font-medium text-gray-900 pt-4 outline-none">
			Good morning Adrian!
		</h3>
		<p class="mt-2 text-base font-normal text-gray-500">What should we start with today?</p>
	</div>

	<div class="flex flex-wrap justify-center">
		<button
			type="button"
			on:click={setComponentToShowDocumentation}
			class="flex-wrap justify-center my-2 w-full items-center px-6 py-3 border border-gray-300 shadow-sm text-base font-medium rounded-md text-gray-700 bg-gray-100 hover:bg-gray-200 active:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
		>
			<span> Could you explain this ... ? </span>
		</button>
		<button
			type="button"
			on:click={setComponentToShowTutorial}
			class="flex-wrap justify-center my-2 w-full items-center px-6 py-3 border border-gray-300 shadow-sm text-base font-medium rounded-md text-gray-700 bg-gray-50 hover:bg-gray-200 active:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
			>I want to continue learning.</button
		>
	</div>
</form>
