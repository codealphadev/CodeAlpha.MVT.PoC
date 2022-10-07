<script lang="ts">
	import { colors } from '../../themes';

	export let count: number;
</script>

<div
	class="text-background w-full h-full flex justify-center items-center font-medium text-xl"
	style="background: {colors.primary_gradient}"
>
	{count}
</div>
