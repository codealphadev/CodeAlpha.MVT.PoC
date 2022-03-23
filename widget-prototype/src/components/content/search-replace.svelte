<script lang="ts">
	import { SwitchHorizontalIcon } from '@rgossiaux/svelte-heroicons/outline';
	import BubbleIcon from './bubble-icon.svelte';
</script>

<form class="space-y-3 ">
	<div class="mt-1">
		<div>
			<h3 class="text-lg leading-6 font-medium text-gray-900 outline-none">Search and Replace</h3>
			<p class="mt-1 text-base font-normal text-gray-500">
				Type a character sequence and press “swap” to replace all instances in the currently focused
				editor.
			</p>
		</div>
		<div class="sm:items-start pt-3">
			<label for="search-str" class="block font-medium text-gray-700 sm:pt-2"> Search </label>
			<div class="mt-1">
				<input
					type="text"
					class="py-2 px-3  border block w-full shadow-sm focus:ring-gray-500 focus:border-gray-500 border-gray-300 rounded-md"
				/>
			</div>
		</div>

		<div class="sm:items-start  pt-3">
			<label for="replace-str" class="block font-medium text-gray-700 sm:pt-2">
				Replace with
			</label>
			<div class="mt-1">
				<input
					type="text"
					class="py-2 px-3  border block w-full shadow-sm focus:ring-gray-500 focus:border-gray-500 border-gray-300 rounded-md"
				/>
			</div>
		</div>
	</div>
	<div class="pt-5">
		<button
			type="button"
			class="inline-flex w-full items-center justify-center px-4 py-2 text-base font-medium rounded-md shadow-sm text-white bg-gray-800 hover:bg-gray-700 focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-100 focus:ring-gray-900"
		>
			<SwitchHorizontalIcon class="-ml-1 mr-3 h-5 w-5" aria-hidden="true" />
			Swap strings
		</button>
	</div>
</form>
