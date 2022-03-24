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
		<div class="pt-3">
			<label for="search-str" class="font-medium text-gray-700 sm:pt-2"> Search </label>
			<div class="mt-1">
				<input
					type="text"
					class="py-2 px-3 border w-full outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-400 focus:border-gray-300 border-gray-300 rounded-md"
				/>
			</div>
		</div>

		<div class="pt-3">
			<label for="replace-str" class="font-medium text-gray-700 sm:pt-2"> Replace with </label>
			<div class="mt-1">
				<input
					type="text"
					class="py-2 px-3 border w-full outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-400 focus:border-gray-300 border-gray-300 rounded-md"
				/>
			</div>
		</div>
	</div>
	<div class="pt-5">
		<button
			type="button"
			class="w-full px-4 py-2 shadow-md bg-gray-800  active:bg-gray-600 hover:bg-gray-700 outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-400 focus:border-gray-300 border-gray-300 rounded-md"
		>
			<div class="inline-flex items-center justify-center text-base font-medium text-white ">
				<SwitchHorizontalIcon class="-ml-1 mr-3 h-5 w-5" aria-hidden="true" />
				Swap strings
			</div>
		</button>
	</div>
</form>
