<script>
	export let disabled = false;
</script>

<button
	on:click
	{disabled}
	type="button"
	class=" cursor-pointer
        inline-flex items-center disabled:bg-red-500 justify-center py-2 px-3 h-8 rounded
        border-none shadow-md 
        bg-primary-gradient
        text-white text-xs font-medium"
>
	<slot />
</button>
