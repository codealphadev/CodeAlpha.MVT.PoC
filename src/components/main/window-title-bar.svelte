<script lang="ts">
	import PretzlLogo from '../common/logo/pretzl-logo.svelte';
	import TitleBarActions from './title-bar-actions.svelte';
</script>

<div
	class="bg-background_secondary absolute flex-shrink-0 h-9 w-full box-border flex flex-row justify-between items-center py-2 px-3 border-b border-solid border-divider rounded-t-xl"
>
	<PretzlLogo />
	<TitleBarActions />
</div>
