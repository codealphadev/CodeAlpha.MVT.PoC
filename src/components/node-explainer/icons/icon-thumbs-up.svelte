<script lang="ts">
    import { colors, ThemeContextType, ThemeName } from "../../../themes";
	import { getContext } from "svelte";

	let theme_name: ThemeName;
	export let activated: boolean;

	getContext<ThemeContextType>("theme").theme.subscribe((t) => {
		theme_name = t.name;
	});
</script>


<svg width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
<rect x="0.75" y="0.75" width="26.5" height="26.5" rx="13.25" fill={activated ? `url(#paint24_${theme_name})` : 'none'} stroke="url(#paint24_{theme_name})" stroke-width="1.5"/>
<path d="M15.3333 12.6667H18.5093C19.5005 12.6667 20.1451 13.7098 19.7019 14.5963L17.3685 19.263C17.1427 19.7147 16.681 20 16.176 20H13.4975C13.3885 20 13.2799 19.9866 13.1741 19.9602L10.6667 19.3333M15.3333 12.6667V9.33333C15.3333 8.59695 14.7364 8 14 8H13.9364C13.6033 8 13.3333 8.26998 13.3333 8.60302C13.3333 9.07922 13.1924 9.54477 12.9282 9.94099L10.6667 13.3333V19.3333M15.3333 12.6667H14M10.6667 19.3333H9.33333C8.59695 19.3333 8 18.7364 8 18V14C8 13.2636 8.59695 12.6667 9.33333 12.6667H11" 
	stroke={activated ? `${colors.background}` : `url(#paint24_${theme_name})` }  stroke-width="1.33" stroke-linecap="round" stroke-linejoin="round"/>
<defs>
<linearGradient id="paint24_dark" x1="19.8443" y1="8" x2="7.84536" y2="19.8433" gradientUnits="userSpaceOnUse">
<stop stop-color="#FF9C64"/>
<stop offset="1" stop-color="#F84545"/>
</linearGradient>

<linearGradient id="paint24_light" x1="19.8443" y1="8" x2="7.84536" y2="19.8433" gradientUnits="userSpaceOnUse">
    <stop stop-color="#0B9CDA"/>
    <stop offset="1" stop-color="#054B8B"/>
    </linearGradient>
</defs>
</svg>