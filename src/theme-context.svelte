<script lang="ts">
  import { setContext, onMount } from "svelte";
  import { Writable, writable } from "svelte/store";

  import { themes, ThemeName, colorNames, mapColorNameToCssVarString, Theme, ThemeContextType} from "./themes"
  import { getContext } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import type { ChannelList } from '../src-tauri/bindings/ChannelList';
  import type { EventWindowControls } from '../src-tauri/bindings/window_controls/EventWindowControls';

    
  
    let _currentTheme: ThemeName = 'light';
  
    // set up Theme store, holding current theme object
    const theme_store = writable(themes[_currentTheme]);
  
    setContext<ThemeContextType>('theme', {
      // providing Theme store through context makes store readonly
      theme: theme_store,
      setTheme: (theme: ThemeName) => {
        _currentTheme = theme;

        theme_store.update(t => ({ ...t, ...themes[_currentTheme] }));
        setRootCssVars(_currentTheme);
      },
    });

    const { setTheme } = getContext<ThemeContextType>('theme');
    const setRootCssVars = (themeName: ThemeName) => {
        const theme = themes[themeName];
      for (const colorName of colorNames) {
        const cssVar = mapColorNameToCssVarString(colorName);
        const color = theme.colors[colorName];
        document.documentElement.style.setProperty(cssVar, color);
      }
      document.documentElement.style.setProperty("--theme-name", themeName);
    };
    
    const listenToWindowControlsEvents = async () => {
      let WindowControlsChannel: ChannelList = 'EventWindowControls';
      await listen(WindowControlsChannel, (e) => {
        const { event, payload } = JSON.parse(e.payload as string) as EventWindowControls;

        switch (event) {
          case 'DarkModeUpdate':
            setTheme(payload.dark_mode ? 'dark' : 'light');
            break;

          default:
            break;
        }
    });
  };
  onMount(() => {
    
    listenToWindowControlsEvents();
    setRootCssVars(_currentTheme);
  });

  </script>
  <slot>
    <!-- content will go here -->
  </slot>