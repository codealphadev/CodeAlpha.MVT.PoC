<script lang="ts">
	import WidgetIcon from '../components/widget/widget-icon.svelte';
	import { invoke } from '@tauri-apps/api/tauri';

	const toggleContent = () => {
		invoke('toggle_content_window');
	};
</script>

<button on:click={toggleContent} data-tauri-drag-region type="button">
	<WidgetIcon />
</button>
