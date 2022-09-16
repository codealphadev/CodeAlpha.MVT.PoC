<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";


	const clickAction = async () => {
        console.log('Hello');
		await invoke('cmd_paste_docs');
	};
</script>

<button on:click={clickAction} type="button" class="inline-flex items-center py-2 px-3 h-8 shadow-md rounded border border-transparent justify-center bg-gradient-to-bl hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-[#054b8b] from-[#0b9cda] to-[#054b8b] text-white text-xs font-medium">
    <slot>

    </slot>
</button>