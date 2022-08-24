<script lang="ts">
	import { Router, Route } from 'yrv';
	import CodeOverlay from './pages/code-overlay.svelte';
	import Widget from './pages/widget.svelte';
	import ThemeContext from './theme-context.svelte';
</script>

<ThemeContext>
	<Router>
		<Route path="/widget">
			<Widget />
		</Route>

		<Route path="/overlay">
			<CodeOverlay />
		</Route>
	</Router>
</ThemeContext>
