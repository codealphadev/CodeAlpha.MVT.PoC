<script lang="ts">
	/* This example requires Tailwind CSS v2.0+ */
	import {
		Dialog,
		DialogOverlay,
		DialogTitle,
		TransitionChild,
		Transition,
	} from "@rgossiaux/svelte-headlessui";
	import { CheckIcon } from "@rgossiaux/svelte-heroicons/outline";

	import type AXEvent from "../../models/AXEvent";
	import { axEventTableStore } from "./AXEventsTableStore";

	import { HighlightSvelte } from "svelte-highlight";
	import github from "svelte-highlight/src/styles/github";

	const { inspectModal } = axEventTableStore;

	export let event: AXEvent | undefined = undefined;
	let open = false;

	$: code = JSON.stringify(event?.payload, null, 2);
</script>

<svelte:head>
	{@html github}
</svelte:head>

<Transition show={$inspectModal}>
	<Dialog
		as="div"
		class="fixed z-10 inset-0 overflow-y-auto"
		on:close={() => inspectModal.set(false)}
	>
		<div
			class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0"
		>
			<TransitionChild
				enter="ease-out duration-300"
				enterFrom="opacity-0"
				enterTo="opacity-100"
				leave="ease-in duration-200"
				leaveFrom="opacity-100"
				leaveTo="opacity-0"
			>
				<DialogOverlay
					class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
				/>
			</TransitionChild>

			<TransitionChild
				enter="ease-out duration-300"
				enterFrom="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
				enterTo="opacity-100 translate-y-0 sm:scale-100"
				leave="ease-in duration-200"
				leaveFrom="opacity-100 translate-y-0 sm:scale-100"
				leaveTo="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
			>
				<span
					class="hidden sm:inline-block sm:align-middle sm:h-screen "
					aria-hidden="true"
				>
					&#8203;
				</span>
				<div
					class="relative inline-block align-bottom bg-white rounded-lg px-4 pt-5 pb-4 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-xl sm:w-full sm:p-6"
				>
					<div>
						<div class="mt-3  sm:mt-5">
							<DialogTitle
								as="h3"
								class="text-lg text-center leading-6 font-medium text-gray-900"
							>
								{event?.eventName}
							</DialogTitle>
							<div class="mt-5 text-sm text-left border rounded-lg px-3">
								<HighlightSvelte {code} />
							</div>
						</div>
					</div>
				</div>
			</TransitionChild>
		</div>
	</Dialog>
</Transition>
