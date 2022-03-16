<script lang="ts">
	import AxTable from "./AXTable.svelte";
	import type AXEvent from "../../models/AXEvent";
	import InspectModal from "./InspectModal.svelte";

	import { axEventTableStore } from "./AXEventsTableStore";
	import { onDestroy } from "svelte";

	const { inspectEventId, inspectModal, axEventList } = axEventTableStore;
</script>

<div class="px-4 sm:px-6 lg:px-8">
	<div class="mt-8 flex flex-col">
		<div class="-my-2 -mx-4 overflow-x-auto sm:-mx-6 lg:-mx-8">
			<div class="inline-block min-w-full py-2 align-middle md:px-6 lg:px-8">
				<AxTable />
			</div>
			{#if $axEventList.length === 0}
				<div
					class="inline-block text-center min-w-full py-5 align-middle md:px-6 lg:px-8"
				>
					... waiting for events.
				</div>
			{/if}
		</div>
	</div>
</div>
<InspectModal
	event={$axEventList.find((element) => element.id === $inspectEventId)}
/>
