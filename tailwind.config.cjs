const colorNames = [
	'background',
	'background_secondary',
	'contrast',
	'contrast_secondary',
	'inactive',
	'primary',
	'secondary',
	'accent_primary',
	'accent_primary_emphasis',
	'accent_secondary',
	'signal_bad',
	'signal_good',
	'signal_medium',
	'signal_very_bad',
	'divider'
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

	plugins: []
};

module.exports = config;
