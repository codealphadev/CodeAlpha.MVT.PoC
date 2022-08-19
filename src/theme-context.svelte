<script lang="ts">
    import { setContext, onMount } from "svelte";
    import { writable } from "svelte/store";

    import { themes, ThemeName} from "./themes"

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
        setRootColors(_currentTheme);
      }
    });
  
    onMount(() => {
      setRootColors(_currentTheme);
    });
  
    // sets CSS vars for easy use in components
    // ex: var(--theme-background)
    const setRootColors = (themeName: ThemeName) => {
        const theme = themes[themeName];
      for (let [prop, color] of Object.entries(theme.colors)) {
        let varString = `--theme-${prop}`;
        document.documentElement.style.setProperty(varString, color);
      }
      document.documentElement.style.setProperty("--theme-name", themeName);
    };
  </script>
  
  <slot>
    <!-- content will go here -->
  </slot>