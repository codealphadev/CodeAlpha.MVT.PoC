import type { Writable } from 'svelte/store';

export type ThemeName = 'light' | 'dark';

export const colorNames = [
	'background',
	'backgroundsecondary',
	'contrast',
	'contrastsecondary',
	'divider',
	'inactive',
	'primary',
	'secondary',
  'accent_primary',
  'accent_secondary',
	'signalbad',
	'signalgood',
	'signalmedium',
	'signalverybad'
] as const;

const wild_sand = '#F7F7F7';
const fiord = '#414B69';
const palette = {
	base: {
		'50': wild_sand,
		'100': '#E2E3E7',
		'200': '#CED1D8',
		'300': '#BBBEC8',
		'400': '#A5AAB8',
		'600': '#7E8499',
    '800': '#555F79',
		'900': fiord
	},
	dodger_blue: {
		'300': '#80B2FE',
		'500': '#3A88FD',
		'800': '#155CC5'
	},
	outrageous_orange: {
		'300': '#FEA081',
		'500': '#FD583A'
	}
};

export type ColorName = typeof colorNames[number];
export interface Theme {
	colors: {
		[key in ColorName]: string;
	};
	name: ThemeName;
}
export interface ThemeContextType {
	setTheme: (theme: ThemeName) => void;
	theme: Writable<Theme>;
}

export const themes: { [name in ThemeName]: Theme } = {
	light: {
		colors: {
			background: '#ffffff',
			backgroundsecondary: palette.base[50],
			contrast: '#000000',
			contrastsecondary: palette.base[400],
			divider: palette.base[200],
			inactive: '#bbbbbb80',
			primary: fiord,
			secondary: palette.base[600],
      accent_primary: palette.dodger_blue[500],
      accent_secondary: palette.outrageous_orange[500],
			signalbad: '#F97316',
			signalgood: '#16A34A',
			signalmedium: palette.dodger_blue[800],
			signalverybad: '#E11D48'
		},
		name: 'light'
	},
	dark: {
		colors: {
			background: '#ffffff',
			backgroundsecondary: palette.base[100],
			contrast: '#000000',
			contrastsecondary: palette.base[600],
			divider: palette.base[300],
			inactive: '#80808080',
			primary: fiord,
			secondary: palette.base[600],
      accent_primary: palette.dodger_blue[500],
      accent_secondary: palette.outrageous_orange[500],
			signalbad: '#F97316',
			signalgood: '#16A34A',
			signalmedium: palette.dodger_blue[800],
			signalverybad: '#E11D48'
		},
		name: 'dark'
	}
};

export function mapColorNameToCssVarString(colorName: ColorName) {
	return `--theme-${colorName}`;
}

export const colors = colorNames.reduce(
	(curr, colorName) => ({ ...curr, [colorName]: `var(${mapColorNameToCssVarString(colorName)})` }),
	{} as { [colorName in ColorName]: string }
);
