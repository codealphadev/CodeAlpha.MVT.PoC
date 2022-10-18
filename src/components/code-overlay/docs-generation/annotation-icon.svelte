<script lang="ts">
	import IconProcessing from '../../widget/icons/icon-processing.svelte';
	import { colors } from '../../../themes';
	import PretzlIcon from '../../common/logo/pretzl-icon.svelte';

	export let is_hovering = false;
	export let is_processing = false;
</script>

<div style="display: flex; align-items: center; height: 100%; width: 100%">
	{#if !is_processing}
		{#if is_hovering}
			<PretzlIcon color={colors.code_overlay_primary} />
		{:else}
			<PretzlIcon color={colors.code_overlay_contrast} />
		{/if}
	{:else}
		<IconProcessing />
	{/if}
</div>
