<script lang="ts">
	import { Router, Route, Link } from 'svelte-navigator';
	import Analytics from './pages/analytics.svelte';
	import Content from './pages/content.svelte';
	import Settings from './pages/settings.svelte';
	import Widget from './pages/widget.svelte';
</script>

<Router>
	<Link to="/" />
	<Link to="settings" />

	<Route path="/">
		<Content />
		<!-- <Analytics /> -->
	</Route>

	<Route path="settings">
		<Settings />
	</Route>

	<Route path="analytics">
		<Analytics />
	</Route>

	<Route path="content">
		<Content />
	</Route>

	<Route path="widget">
		<Widget />
	</Route>
</Router>
