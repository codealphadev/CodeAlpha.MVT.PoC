<script lang="ts">
  import Close from '../common/close.svelte';
  import { toggle_main_window } from '../../utils';
  import ButtonIcon from '../common/buttons/button-icon.svelte';

  const close_main_window = async () => {
    toggle_main_window(false);
  };
</script>

<ButtonIcon on:click={close_main_window}>
  <Close />
</ButtonIcon>
