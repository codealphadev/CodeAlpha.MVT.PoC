<script lang="ts">
	import { fade } from 'svelte/transition';

	export let id: string | undefined = undefined;
</script>

<div
	{id}
	class="rounded-xl bg-background overflow-hidden py-4 px-5 flex flex-col items-start gap-3 border-none"
	in:fade={{ duration: 100 }}
>
	<slot />
</div>
