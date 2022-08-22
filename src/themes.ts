
const CodeAlphaOrange = "#fc7456";
export interface Theme {
    colors: {
        primary: string;
        secondary: string;
        inactive: string;
        contrast: string;
    }
}
export type ThemeName = 'light' | 'dark';
export const themes: { [name in ThemeName]: Theme; } = {
    light: {
      colors: {
        primary: CodeAlphaOrange,
        secondary: "#555555",
        inactive: '#cccccc',
        contrast: "#000000"
      },
    },
    dark: {
      colors: {
        primary: CodeAlphaOrange,
        secondary: "#aaaaaa",
        inactive: '#808080',
        contrast: "#ffffff"
      },
    },
 };
