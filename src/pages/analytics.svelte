<script lang="ts">
	import EventListener from '../components/event-listener/EventListener.svelte';
	import HeaderAnalytics from '../components/header/HeaderAnalytics.svelte';
</script>

<div class="py-10">
	<header class="px-4 sm:px-6 lg:px-8">
		<HeaderAnalytics />
	</header>
	<main class="px-4 sm:px-6 lg:px-8">
		<EventListener />
	</main>
</div>
