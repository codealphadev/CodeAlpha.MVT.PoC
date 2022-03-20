<script lang="ts">
	import { Popover, PopoverPanel, PopoverButton, Transition } from '@rgossiaux/svelte-headlessui';
	import { appWindow, LogicalSize } from '@tauri-apps/api/window';
	import { afterUpdate } from 'svelte';
	import WidgetIcon from './widget-icon.svelte';

	const duder = (panel: any) => {
		let allVal = window.getComputedStyle(document.body);
		console.log(allVal.getPropertyValue('height'));
		setTimeout(() => {
			let allVal = window.getComputedStyle(document.body);
			console.log(allVal.getPropertyValue('height'));
			let newHeight = allVal.getPropertyValue('height');
			let newWidth = panel ? 416 : 64;

			console.log(newHeight, newWidth);

			appWindow.setSize(new LogicalSize(newWidth, parseFloat(newHeight)));
		}, 10);
	};

	let panelObj;

	$: duder(panelObj);
</script>

<div class="w-full max-w-sm">
	<Popover class={panelObj ? 'flex flex-col items-end' : 'flex flex-col'} let:open={active}>
		<!-- content here -->
		<!-- <Transition
			enter="transition ease-out duration-200"
			enterFrom="opacity-0 translate-y-1"
			enterTo="opacity-100 translate-y-0"
			leave="transition ease-in duration-150"
			leaveFrom="opacity-100 translate-y-0"
			leaveTo="opacity-0 translate-y-1"
		> -->
		{#if active}
			<PopoverPanel bind:this={panelObj} class="w-full">
				<div
					class=" overflow-hidden rounded-lg shadow-lg ring-1 focus:outline-none   ring-black ring-opacity-5"
				>
					<div class="p-4 bg-gray-50">
						<a
							href="##"
							class="flow-root px-2 py-2 transition duration-150 ease-in-out rounded-md hover:bg-gray-100 focus:outline-none focus-visible:ring focus-visible:ring-orange-500 focus-visible:ring-opacity-50"
						>
							<span class="flex items-center">
								<span class="text-sm font-medium text-gray-900"> Documentation </span>
							</span>
							<span class="block text-sm text-gray-500">
								Start integrating products and tools
							</span>
						</a>
					</div>
				</div>
			</PopoverPanel>
		{/if}
		<PopoverButton class="focus:outline-none ">
			<div>
				<WidgetIcon />
			</div>
		</PopoverButton>
		<!-- </Transition> -->
	</Popover>
</div>
