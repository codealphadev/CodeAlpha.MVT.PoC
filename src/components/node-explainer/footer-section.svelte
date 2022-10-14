<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import IconThumbsDown from './icons/icon-thumbs-down.svelte';
  import IconThumbsUp from './icons/icon-thumbs-up.svelte';
  import { appWindow } from '@tauri-apps/api/window';
  import ButtonPrimary from '../common/buttons/button-primary.svelte';
  import ButtonThumb from '../common/buttons/button-thumb.svelte';

  let vote: 'good' | 'bad' | null = null;
  const paste_docs = async () => {
    await invoke('cmd_paste_docs');
    await appWindow.hide();
  };

  const handle_click_thumbs_up = async () => {
    if (vote) {
      return;
    }
    await invoke('cmd_send_feedback', {
      feature: 'NodeExplainer',
      feedback: 'good',
    });
    vote = 'good';
  };

  const handle_click_thumbs_down = async () => {
    if (vote) {
      return;
    }
    await invoke('cmd_send_feedback', {
      feature: 'NodeExplainer',
      feedback: 'bad',
    });
    await appWindow.hide();
    vote = 'bad';
  };
</script>

<div class="flex justify-between w-full items-center">
  <div class="flex gap-3 px-1 items-center">
    <ButtonThumb on:click={handle_click_thumbs_up}>
      <IconThumbsUp activated={vote === 'good'} />
    </ButtonThumb>
    <ButtonThumb on:click={handle_click_thumbs_down}>
      <IconThumbsDown activated={vote === 'bad'} />
    </ButtonThumb>
    <div class="text-secondary text-xs">
      {#if vote}
        Thanks for the feedback!
      {:else}
        Honestly, was this helpful?
      {/if}
    </div>
  </div>
  <ButtonPrimary on:click={paste_docs}>Insert as Docstring</ButtonPrimary>
</div>
