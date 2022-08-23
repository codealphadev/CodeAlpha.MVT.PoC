<script lang="ts">
    import { setContext, onMount } from "svelte";
    import { writable } from "svelte/store";

    import { themes, ThemeName, colorNames, mapColorNameToCssVarString} from "./themes"
    
    let _currentTheme: ThemeName = 'light';
  
    // set up Theme store, holding current theme object
    const theme_store = writable(themes[_currentTheme]);
  
    setContext('theme', {
      // providing Theme store through context makes store readonly
      theme: theme_store,
      setTheme: (theme: ThemeName) => {
        console.log(`Setting theme to ${theme}`);

        _currentTheme = theme;

        theme_store.update(t => ({ ...t, ...themes[_currentTheme] }));
        setRootCssVars(_currentTheme);
      }
    });
  
    onMount(() => {
      setRootCssVars(_currentTheme);
    });
  
    const setRootCssVars = (themeName: ThemeName) => {
        const theme = themes[themeName];
      for (const colorName of colorNames) {
        const cssVar = mapColorNameToCssVarString(colorName);
        const color = theme.colors[colorName];
        document.documentElement.style.setProperty(cssVar, color);
      }
      document.documentElement.style.setProperty("--theme-name", themeName);
    };
  </script>
  
  <slot>
    <!-- content will go here -->
  </slot>