<script lang="ts">
	import { DialogTitle, Dialog, Transition } from '@rgossiaux/svelte-headlessui';
	import { XIcon } from '@rgossiaux/svelte-heroicons/outline';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onMount } from 'svelte';

	let isOpen = true;

	$: close = () => {
		invoke('close_window', { windowLabel: 'Content' });
	};

	let contentRootContainerHeight: number;
	let contentRootContainerWidth: number;

	onMount(() => {
		// invoke('resize_window', {
		// 	windowLabel: 'Content',
		// 	sizeX: contentRootContainerWidth,
		// 	sizeY: contentRootContainerHeight + 38
		// });
	});
</script>

<Transition
	show={isOpen}
	appear={true}
	enter="transition-opacity duration-500"
	enterFrom="opacity-0"
	enterTo="opacity-100"
	leave="transition-opacity duration-150"
	leaveFrom="opacity-100"
	leaveTo="opacity-0"
>
	<div class="bg-transparent">
		<Dialog>
			<div
				data-tauri-drag-region
				bind:offsetWidth={contentRootContainerWidth}
				bind:offsetHeight={contentRootContainerHeight}
				class="relative border inline-block align-bottom bg-white rounded-lg px-4 pt-5 pb-4 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-sm sm:w-full sm:p-6"
			>
				<div class="absolute top-0 right-0 pt-4 pr-4">
					<button
						type="button"
						class="bg-white rounded-md text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-200"
						on:click={close}
					>
						<XIcon class="h-6 w-6" />
					</button>
				</div>
				<div class="sm:flex sm:items-start">
					<div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
						<DialogTitle as="h3" class="text-lg leading-6 font-medium text-gray-900">
							Deactivate account
						</DialogTitle>
						<div class="mt-2">
							<p class="text-sm text-gray-500">
								Are you sure you want to deactivate your account? All of your data will be
								permanently removed from our servers forever. This action cannot be undone.
							</p>
						</div>
					</div>
				</div>
				<div class="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse">
					<button
						type="button"
						class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm"
					>
						Deactivate
					</button>
					<button
						type="button"
						class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:w-auto sm:text-sm"
					>
						Cancel
					</button>
				</div>
			</div>
		</Dialog>
	</div>
</Transition>
