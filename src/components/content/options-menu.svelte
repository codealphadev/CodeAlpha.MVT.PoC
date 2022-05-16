<script lang="ts">
	import { Menu, MenuButton, MenuItems, MenuItem, Transition } from '@rgossiaux/svelte-headlessui';
	import { CogIcon } from '@rgossiaux/svelte-heroicons/outline';
	import { invoke } from '@tauri-apps/api/tauri';

	$: openSettings = () => {
		invoke('cmd_open_window', { windowLabel: 'Settings' });
	};

	$: openDebug = () => {
		invoke('cmd_toggle_window', { windowLabel: 'Analytics' });
	};

	export let componentToShow = 0;

	$: setComponentToShowSearchAndReplace = () => {
		componentToShow = 4;
	};
</script>

<Menu as="div" class="relative">
	<MenuButton as="div" class="absolute top-0 right-0 outline-none">
		<button type="button" class="bg-white rounded-md text-gray-400 hover:text-gray-500 ">
			<CogIcon class="h-6 w-6" />
		</button>
	</MenuButton>

	<Transition
		enter="transition ease-out duration-100"
		enterFrom="transform opacity-0 scale-95"
		enterTo="transform opacity-100 scale-100"
		leave="transition ease-in duration-75"
		leaveFrom="transform opacity-100 scale-100"
		leaveTo="transform opacity-0 scale-95"
	>
		<MenuItems
			class="origin-top-right absolute z-30 right-0 mt-6 py-1 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 focus:outline-none"
		>
			<div class="hover:bg-gray-100">
				<MenuItem let:active>
					<button on:click={openDebug} type="button" class="block px-4 py-2 text-sm text-gray-700"
						>Debug Tools</button
					>
				</MenuItem>
			</div>
			<div class="hover:bg-gray-100">
				<MenuItem let:active>
					<button
						on:click={openSettings}
						type="button"
						class="block px-4 py-2 text-sm text-gray-700">Settings</button
					>
				</MenuItem>
			</div>
			<div class="hover:bg-gray-100">
				<MenuItem let:active>
					<button
						on:click={setComponentToShowSearchAndReplace}
						type="button"
						class="block px-4 py-2 text-sm text-gray-700">Test: Search & Replace</button
					>
				</MenuItem>
			</div>
		</MenuItems>
	</Transition>
</Menu>
