<script lang="ts">
	import { ButtonType } from './button';

	export let type: ButtonType = ButtonType.Primary;
</script>

<button
	on:click|stopPropagation
	type="button"
	class="
    inline-flex items-center justify-center rounded py-[7px] px-[11px] h-[30px] border-none text-xs
    {type === ButtonType.Primary
		? 'button-primary bg-accent_primary shadow-[2px_2px_0px_#FD583A] text-white'
		: 'button-secondary bg-background text-[#374151]'}
    "
>
	<slot />
</button>
