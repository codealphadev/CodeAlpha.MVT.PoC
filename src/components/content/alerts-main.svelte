<script lang="ts">
	import { DotsVertical } from '@rgossiaux/svelte-heroicons/solid/DotsVertical';
	import { alerts } from '../../state';
</script>

<ul class="mt-3 grid grid-cols-1 gap-5">
	{#each $alerts as alert}
		<li class="col-span-1 flex shadow-sm rounded-md">
			<div
				class="{alert.match_properties.category == 'Error'
					? 'bg-red-500'
					: 'bg-yellow-500'} flex-shrink-0 flex items-center justify-center w-16 text-white text-sm font-medium rounded-l-md"
			>
				{alert.rule_name}
			</div>
			<div
				class="flex-1 flex items-center justify-between border-t border-r border-b border-gray-200 bg-white rounded-r-md truncate"
			>
				<div class="flex-1 px-4 py-2 text-sm truncate">
					<a href="/" class="text-gray-900 font-medium hover:text-gray-600">
						{alert.match_properties.identifier}
					</a>
					<p class="text-gray-500">{alert.match_properties.description}</p>
				</div>
				<div class="flex-shrink-0 pr-2">
					<button
						type="button"
						class="w-8 h-8 bg-white inline-flex items-center justify-center text-gray-400 rounded-full bg-transparent hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
					>
						<span class="sr-only">Open options</span>
						<DotsVertical class="w-5 h-5" aria-hidden="true" />
					</button>
				</div>
			</div>
		</li>
	{/each}
</ul>
