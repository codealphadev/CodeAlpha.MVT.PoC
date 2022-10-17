<script lang="ts">
	import { Router, Route } from 'yrv';
	import CodeOverlay from './pages/code-overlay.svelte';
	import Widget from './pages/widget.svelte';
	import ThemeContext from './theme-context.svelte';
	import Main from './pages/main.svelte';
	import Explain from './pages/explain.svelte';
	import { WindowPath } from './utils';
</script>

<ThemeContext>
	<Router>
		<Route path={WindowPath.Widget}>
			<Widget />
		</Route>
		<Route path={WindowPath.CodeOverlay}>
			<CodeOverlay />
		</Route>
		<Route path={WindowPath.Explain}>
			<Explain />
		</Route>
		<Route path={WindowPath.Main}>
			<Main />
		</Route>
	</Router>
</ThemeContext>
