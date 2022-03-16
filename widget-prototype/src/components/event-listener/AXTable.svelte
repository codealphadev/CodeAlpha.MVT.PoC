<script lang="ts">
	import { axEventTableStore } from "./AXEventsTableStore";
	import AxTableItem from "./AXTableItem.svelte";
	import type AXEvent from "../../models/AXEvent";
	import { onDestroy } from "svelte";

	const { inspectEventId, inspectModal, axEventList } = axEventTableStore;

	const resetAXEventList = () => {
		axEventList.set([]);
	};
</script>

<div
	class="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg"
>
	<table class="min-w-full divide-y divide-gray-300">
		<thead class="bg-gray-50">
			<tr>
				<th
					scope="col"
					class="whitespace-nowrap py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-6"
				>
					Timestamp
				</th>
				<th
					scope="col"
					class="whitespace-nowrap px-2 py-3.5 text-left text-sm font-semibold text-gray-900"
				>
					Event name
				</th>
				<th
					scope="col"
					class="whitespace-nowrap px-2 py-3.5 text-left text-sm font-semibold text-gray-900"
				>
					Message type
				</th>
				<th
					scope="col"
					class="relative whitespace-nowrap py-3.5 pl-3 pr-4 sm:pr-6"
				>
					<span class="sr-only">Inspect</span>
				</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-gray-200 bg-white">
			{#each $axEventList as axEvent (axEvent.id)}
				<AxTableItem {axEvent} />
			{/each}
		</tbody>
	</table>
</div>
