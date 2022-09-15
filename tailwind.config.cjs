const colorNames = [
	'primary',
	'secondary',
	'inactive',
	'contrast',
	'background',
	'primarybackground'
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
