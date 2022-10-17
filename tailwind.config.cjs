const colorNames = [
	'accent_primary_emphasis',
	'accent_primary',
	'accent_secondary',
	'background_emphasis',
	'background_secondary',
	'background',
	'code_overlay_primary',
	'code_overlay_primary_emphasis',
	'code_overlay_secondary',
	'contrast_secondary',
	'contrast',
	'divider',
	'inactive',
	'primary',
	'secondary',
	'signal_bad',
	'signal_good',
	'signal_medium',
	'signal_very_bad'
];

function mapColorNameToCssVarString(colorName) {
	return `--theme-${colorName}`;
}

const colors = colorNames.reduce(
	(curr, colorName) => ({ ...curr, [colorName]: `var(${mapColorNameToCssVarString(colorName)})` }),
	{}
);

const config = {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: { colors }
	},

	plugins: [require('@tailwindcss/line-clamp')]
};

module.exports = config;
