<script lang="ts">
	import { Disclosure, DisclosureButton, DisclosurePanel } from '@rgossiaux/svelte-headlessui';
	import { emit } from '@tauri-apps/api/event';

	let selectedAlertId = null;

	function alertClicked(id) {
		if (selectedAlertId === id) {
			selectedAlertId = null;
			emit('alert-deselected');
		} else {
			selectedAlertId = id;
			emit('alert-selected', id);
		}
	}

	import { alerts } from '../../state';
</script>

<div class="flex flex-col items-stretch p-1 space-y-1">
	{#each $alerts as alert}
		<Disclosure class="border rounded-md">
			<DisclosureButton class="flex items-stretch w-full" on:click={() => alertClicked(alert.id)}>
				<div
					class="{alert.match_properties.category == 'Error'
						? 'bg-red-500'
						: 'bg-yellow-500'} w-12 text-white text-sm rounded-l-md"
				>
					{alert.rule_name}
				</div>
				<div
					class="grow border-gray-200 bg-white text-left px-4 py-2 text-sm text-gray-900 font-medium"
				>
					<p>{alert.match_properties.identifier}</p>
				</div>
			</DisclosureButton>

			<DisclosurePanel>
				<p class="text-gray-500 text-sm">{alert.match_properties.description}</p>
			</DisclosurePanel>
		</Disclosure>
	{/each}
</div>
