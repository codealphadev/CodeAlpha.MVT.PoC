<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import type { ChannelList } from '../../../src-tauri/bindings/ChannelList';
  import type { EventViewport } from '../../../src-tauri/bindings/macOS_specific/EventViewport';
  import Suggestions from '../suggestions/suggestions.svelte';
  import NoSuggestions from '../suggestions/no-suggestions.svelte';
  import WindowTitleBar from './window-title-bar.svelte';

  let CONTAINER_DOM_ID = 'main-window-container';

  let active_window_uid: number | null = null;

  const listenToViewportEvents = async () => {
    let ViewportChannel: ChannelList = 'EventViewport';
    await listen(ViewportChannel, (e) => {
      const { event, payload } = JSON.parse(e.payload as string) as EventViewport;
      switch (event) {
        case 'XcodeViewportUpdate':
          active_window_uid = payload.viewport_properties.window_uid;
          break;
        default:
          break;
      }
    });
  };

  listenToViewportEvents();
</script>

<div id={CONTAINER_DOM_ID} class="relative">
  <WindowTitleBar />
  {#if active_window_uid}
    <Suggestions {CONTAINER_DOM_ID} {active_window_uid} />
  {:else}
    <NoSuggestions />
  {/if}
</div>
