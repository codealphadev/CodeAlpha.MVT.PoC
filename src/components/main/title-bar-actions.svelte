<script lang="ts">
  import Close from '../common/close.svelte';
  import { toggle_main_window } from '../../utils';
  import Button from '../common/button/button.svelte';
  import { ButtonType } from '../common/button/button';

  const close_main_window = async () => {
    toggle_main_window(false);
  };
</script>

<Button type={ButtonType.Icon} on:click={close_main_window}>
  <Close />
</Button>
