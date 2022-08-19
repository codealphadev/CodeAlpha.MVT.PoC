
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
        inactive: '#bcbcbcb3',
        contrast: "#000000"
      },
    },
    dark: {
      colors: {
        primary: CodeAlphaOrange,
        secondary: "#aaaaaa",
        inactive: '#bcbcbcb3',
        contrast: "#ffffff"
      },
    },
 };
