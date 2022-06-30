<script lang="ts">
	import { Disclosure, DisclosureButton, DisclosurePanel } from '@rgossiaux/svelte-headlessui';

	import { alerts } from '../../state';
</script>

<div class="flex flex-col">
	{#each $alerts as alert}
		<Disclosure>
			<DisclosureButton class="flex items-stretch">
				<div
					class="{alert.match_properties.category == 'Error'
						? 'bg-red-500'
						: 'bg-yellow-500'} w-12 text-white text-sm"
				>
					{alert.rule_name}
				</div>
				<div class="flex-1 border-t border-r border-b border-gray-200 bg-white text-left">
					<div class="px-4 py-2 text-sm">
						<a href="/" class="text-gray-900 font-medium">
							{alert.match_properties.identifier}
						</a>
						<p class="text-gray-500">{alert.match_properties.description}</p>
					</div>
				</div>
			</DisclosureButton>

			<DisclosurePanel>
				Yes! You can purchase a license that you can share with your entire team.
			</DisclosurePanel>
		</Disclosure>
	{/each}
</div>
