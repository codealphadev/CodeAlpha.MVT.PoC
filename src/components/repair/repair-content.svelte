<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { appWindow } from '@tauri-apps/api/window';

	let searchString: string;
	const add_docstring = () => {
		invoke('cmd_add_docstring', {
			replaceStr: docText
		});

		setTimeout(() => {
			appWindow.hide();
		}, 100);
	};

	const prepare_docstring = () => {
		// insert line breaks into searchString every 70 characters
		let docstring = '// ' + searchString.match(/.{1,70}/g).join('\n// ');
		return docstring;
	};

	let docText =
		"/**\r\n * \"If the node's value matches the value we're looking for, return the node. Otherwise, recursively\r\n * search the node's children.\"\r\n * \r\n * The function starts by checking if the node has children. If it doesn't, it returns null. If it\r\n * does, it checks if the node's value matches the value we're looking for. If it does, it returns the\r\n * node. If it doesn't, it loops through the node's children, calling the function recursively on each\r\n * child. If the function returns a node, it returns that node. Otherwise, it returns null\r\n * \n * @param {TreeNode | null} node - The node to start searching from.\r\n * @param {string} value - The value of the node you're looking for.\r\n * @returns The first node that matches the value.\r\n */";
</script>

<h2 class="text-md font-medium text-gray-900">Code block explanation:</h2>
<p
	class="mt-2 text-sm font- text-gray-500 border-l-4 border px-3 py-1 bg-gray-50"
	bind:textContent={searchString}
	contenteditable
>
	<!-- prettier-ignore -->
	If the node's value matches the value we're looking for, return the node. Otherwise, recursively search the node's children.
	<!-- prettier-ignore -->
</p>
<h2 class="mt-2 text-md font-medium text-gray-900">Control flow steps:</h2>
<div class="mt-2 flex flex-col">
	<div class="-my-2 overflow-x-auto -mx-8">
		<div class="inline-block min-w-full py-2 align-middle px-8">
			<div class=" shadow ring-1 ring-black ring-opacity-5 rounded-md">
				<table class="min-w-full divide-y divide-gray-300">
					<tbody class="divide-y divide-gray-200 bg-white">
						<tr>
							<td
								class="rounded-lg whitespace-nowrap py-2 pl-4 pr-3 w-16 text-sm font-medium text-gray-900 sm:pl-6"
								>Step 1</td
							>
							<td class=" rounded-lg whitespace-nowrap px-3 py-2 text-sm text-gray-500"
								>The function starts by checking if the node has children.</td
							>
						</tr>
						<tr>
							<td
								class="whitespace-nowrap py-2 pl-4 pr-3 w-16 text-sm font-medium text-gray-900 sm:pl-6"
								>Step 2</td
							>
							<td class="whitespace-nowrap px-3 py-2 text-sm text-gray-500"
								>If it doesn't, it returns null.</td
							>
						</tr>
						<tr>
							<td
								class="whitespace-nowrap py-2 pl-4 pr-3 w-16 text-sm font-medium text-gray-900 sm:pl-6"
								>Step 3</td
							>
							<td class="whitespace-nowrap px-3 py-2 text-sm text-gray-500"
								>If it does, it checks if the node's value matches the value<br />we're looking for.
							</td>
						</tr>
						<tr>
							<td
								class="whitespace-nowrap py-2 pl-4 pr-3 w-16 text-sm font-medium text-gray-900 sm:pl-6"
								>Step 4</td
							>
							<td class="whitespace-nowrap px-3 py-2 text-sm text-gray-500"
								>If it does, it returns the node.</td
							>
						</tr>
						<tr>
							<td
								class="whitespace-nowrap py-2 pl-4 pr-3 w-16 text-sm font-medium text-gray-900 sm:pl-6"
								>Step 5</td
							>
							<td class="whitespace-nowrap px-3 py-2 text-sm text-gray-500"
								>If it doesn't, it loops through the node's children, calling the<br />function
								recursively on each child.</td
							>
						</tr>
						<tr>
							<td
								class="whitespace-nowrap py-2 pl-4 pr-3 w-16 text-sm font-medium text-gray-900 sm:pl-6"
								>Step 6</td
							>
							<td class="whitespace-nowrap px-3 py-2 text-sm text-gray-500"
								>If the function returns a node, it returns that node.</td
							>
						</tr>
						<tr>
							<td
								class="rounded-lg whitespace-nowrap py-2 pl-4 pr-3 w-16 text-sm font-medium text-gray-900 sm:pl-6"
								>Step 7</td
							>
							<td class="rounded-lg whitespace-nowrap px-3 py-2 text-sm text-gray-500"
								>Otherwise, it returns null.</td
							>
						</tr>
					</tbody>
				</table>
			</div>
		</div>
	</div>
</div>

<div class="w-full mt-4 flex justify-end">
	<button
		type="button"
		on:click={add_docstring}
		class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-cyan-600 hover:bg-cyan-700 active:bg-cyan-800 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-cyan-500"
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="-ml-1 mr-2 h-5 w-5"
			viewBox="0 0 20 20"
			fill="currentColor"
		>
			<path
				fill-rule="evenodd"
				d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM6.293 6.707a1 1 0 010-1.414l3-3a1 1 0 011.414 0l3 3a1 1 0 01-1.414 1.414L11 5.414V13a1 1 0 11-2 0V5.414L7.707 6.707a1 1 0 01-1.414 0z"
				clip-rule="evenodd"
			/>
		</svg>
		Add as Docstring to file
	</button>
</div>
