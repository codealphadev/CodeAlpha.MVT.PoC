const CodeAlphaOrange = '#fc7456';

export type ThemeName = 'light' | 'dark';
export const colorNames = ['primary', 'secondary', 'inactive', 'contrast'] as const;

export type ColorName = typeof colorNames[number];
export interface Theme {
	colors: {
		[key in ColorName]: string;
	};
}

export const themes: { [name in ThemeName]: Theme } = {
	light: {
		colors: {
			primary: CodeAlphaOrange,
			secondary: '#555555',
			inactive: '#bbbbbb80',
			contrast: '#000000'
		}
	},
	dark: {
		colors: {
			primary: CodeAlphaOrange,
			secondary: '#aaaaaa',
			inactive: '#80808080',
			contrast: '#ffffff'
		}
	}
};

export function mapColorNameToCssVarString(colorName: ColorName) {
	return `--theme-${colorName}`;
}

export const colors = colorNames.reduce(
	(curr, colorName) => ({ ...curr, [colorName]: `var(${mapColorNameToCssVarString(colorName)})` }),
	{} as { [colorName in ColorName]: string }
);
