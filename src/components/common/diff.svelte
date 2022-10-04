<script lang="ts">
	import ReactAdapter from '../common/ReactAdapter.svelte';
	import ReactDiffViewer, { DiffMethod } from 'react-diff-viewer';
	import type { ThemeContextType, ThemeName } from '../../themes';
	import { getContext } from 'svelte';

	export let new_code: string;
	export let old_code: string;

	let theme_name: ThemeName;
	console.log(old_code);

	getContext<ThemeContextType>('theme').theme.subscribe((t) => {
		theme_name = t.name;
	});

	const styles = {
		diffContainer: {
			pre: { lineHeight: '19px', fontSize: '12px' }
		},
		wordDiff: {
			padding: '0px',
			display: 'inline-block'
		}
	};
</script>

<div class="w-full max-h-96 overflow-y-auto overflow-x-hidden">
	<ReactAdapter
		el={ReactDiffViewer}
		oldValue={old_code.replaceAll(' ', ' ')}
		newValue={new_code}
		splitView={true}
		useDarkTheme={theme_name === 'dark'}
		compareMethod={DiffMethod.WORDS}
		{styles}
	/>
</div>
