<script lang="ts">
	import Close from '../common/close.svelte';
	import { toggle_main_window } from '../../utils';
	import IconButton from '../common/button/icon-button.svelte';

	const close_main_window = async () => {
		toggle_main_window(false);
	};
</script>

<IconButton on:click={close_main_window}>
	<Close />
</IconButton>
