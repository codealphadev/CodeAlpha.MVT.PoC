<script lang="ts">	

	import { Router, Route } from 'yrv';
	import Explainer from './pages/explainer.svelte';
	import Widget from './pages/widget.svelte';
	import ThemeContext from './theme-context.svelte';

</script>

<ThemeContext>
	<Router>
		<Route path="/widget">
			<Widget />
		</Route>

		<Route path="/codeoverlay">
			<Explainer />
		</Route>

	</Router>
</ThemeContext>
