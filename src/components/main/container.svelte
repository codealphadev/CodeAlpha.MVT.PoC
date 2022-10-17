<script lang="ts">
	import Suggestions from '../suggestions/suggestions.svelte';
	import WindowTitleBar from './window-title-bar.svelte';

	let CONTAINER_DOM_ID = 'main-window-container';
</script>

<div id={CONTAINER_DOM_ID} class="relative">
	<WindowTitleBar />
	<Suggestions {CONTAINER_DOM_ID} />
</div>
