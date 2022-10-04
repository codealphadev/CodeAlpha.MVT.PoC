<script lang="ts">
	import { Router, Route } from 'yrv';
	import CodeOverlay from './pages/code-overlay.svelte';
	import Widget from './pages/widget.svelte';
	import ThemeContext from './theme-context.svelte';
	import Main from './pages/main.svelte';
	import Explainer from './pages/explainer.svelte';
</script>

<ThemeContext>
	<Router>
		<Route path="/widget">
			<Widget />
		</Route>
		<Route path="/codeoverlay">
			<CodeOverlay />
		</Route>
		<Route path="/explain">
			<Explainer />
		</Route>
		<Route path="/main">
			<Main />
		</Route>
	</Router>
</ThemeContext>
