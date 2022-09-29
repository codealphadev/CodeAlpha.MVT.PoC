<script lang="ts">
    import ReactAdapter from "../common/ReactAdapter.svelte";
    import ReactDiffViewer from 'react-diff-viewer';
	import type { ThemeContextType, ThemeName } from "../../themes";
	import { getContext } from "svelte";
    
    export let old_code: string;
    export let new_code: string;

    let theme_name: ThemeName;

	getContext<ThemeContextType>("theme").theme.subscribe((t) => {
		theme_name = t.name;
	});
    
    const styles = {
		diffContainer: {
			pre: { lineHeight: '19px', fontSize: '12px' }
		},
		wordDiff: {
			padding: '0px'
		}
	};

</script>

<div class="w-full">
<ReactAdapter el={ReactDiffViewer} oldValue={old_code} newValue={new_code} splitView={true} useDarkTheme={theme_name === 'dark'} styles={styles}/>
</div>