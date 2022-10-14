<script lang="ts">
  import IconProcessing from '../../widget/icons/icon-processing.svelte';
  import { colors } from '../../../themes';
  import Icon from '../../common/logo/icon.svelte';

  export let is_hovering = false;
  export let is_processing = false;
</script>

<div style="display: flex; align-items: center; height: 100%; width: 100%">
  {#if !is_processing}
    {#if is_hovering}
      <Icon color={colors.accent_primary_emphasis} />
    {:else}
      <Icon color={colors.accent_primary} />
    {/if}
  {:else}
    <IconProcessing />
  {/if}
</div>
