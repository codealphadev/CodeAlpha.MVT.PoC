const colorNames = [
	'background',
	'backgroundsecondary',
	'contrast',
	'contrastsecondary',
	'inactive',
	'primary',
	'secondary',
	'accent_primary',
	'accent_primary_muted',
	'accent_secondary',
	'signalbad',
	'signalgood',
	'signalmedium',
	'signalverybad',
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
