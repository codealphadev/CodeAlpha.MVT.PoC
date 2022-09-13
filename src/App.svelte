<script lang="ts">	

	import { Router, Route } from 'yrv';
	import CodeOverlay from './pages/code-overlay.svelte';
	import NodeExplainer from './pages/node-explainer.svelte';
	import Widget from './pages/widget.svelte';
	import ThemeContext from './theme-context.svelte';

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
			<NodeExplainer />
		</Route>
	</Router>
</ThemeContext>
