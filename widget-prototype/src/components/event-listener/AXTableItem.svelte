<script lang="ts">
	import type AXEvent from "../../models/AXEvent";
	import { axEventTableStore } from "./AXEventsTableStore";

	export let axEvent: AXEvent;

	const { inspectEventId, inspectModal } = axEventTableStore;

	const updateInspectEvent = () => {
		inspectEventId.set(axEvent.id);
		inspectModal.set(true);
	};
</script>

<tr>
	<td class="whitespace-nowrap py-2 pl-4 pr-3 text-sm text-gray-500 sm:pl-6">
		{axEvent.timestamp.toUTCString()}
	</td>
	<td class="whitespace-nowrap px-2 py-2 text-sm font-medium text-gray-900">
		{axEvent.eventName}
	</td>
	<td class="whitespace-nowrap px-2 py-2 text-sm text-gray-900"
		>{axEvent.messageType}</td
	>
	<td>
		<button
			type="button"
			class="inline-flex items-center rounded-md border border-gray-300 bg-white px-1.5 py-1 text-sm font-medium leading-4 text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-30"
			on:click={updateInspectEvent}
		>
			Inspect
		</button>
	</td>
</tr>
