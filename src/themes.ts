const CodeAlphaBlue = '#0A7CBB';

export type ThemeName = 'light' | 'dark';
export const colorNames = [
	'primary',
	'secondary',
	'inactive',
	'contrast',
	'background',
	'backgroundgrey',
	'frame'
] as const;

export type ColorName = typeof colorNames[number];
export interface Theme {
	colors: {
		[key in ColorName]: string;
	};
}

export const themes: { [name in ThemeName]: Theme } = {
	light: {
		colors: {
			background: '#ffffff',
			primary: '#0B92DA',
			secondary: '#636363',
			inactive: '#bbbbbb80',
			contrast: '#000000',
			backgroundgrey: '#f5f5f5',
			frame: '#e5e5e5'
		}
	},
	dark: {
		colors: {
			background: '#222222',
			primary: CodeAlphaBlue,
			secondary: '#aaaaaa',
			inactive: '#80808080',
			contrast: '#ffffff',
			backgroundgrey: '#2c2c2c',
			frame: '#3c3c3c'
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

export const gradient_tailwind = 'bg-gradient-to-bl from-[#0b9cda] to-[#054b8b]';
export const gradient_text_tailwind = `${gradient_tailwind} bg-clip-text text-transparent`;
