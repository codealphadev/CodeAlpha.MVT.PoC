<script lang="ts">
  import { ButtonType } from './button';

  export let type: ButtonType;

  const flex = 'inline-flex items-center';
  const text = 'text-xs font-medium';
  const py = 'py-[7px]';
  const px = 'px-[11px]';
  const h = 'h-[30px]';
</script>

{#if type === ButtonType.Primary}
  <button
    on:click
    type="button"
    class="button-primary
      inline-flex items-center justify-center rounded {py} {px} {h} 
      border-none shadow-[2px_2px_0px_#FD583A]
      bg-accent_primary
      text-white {text}"
  >
    <slot />
  </button>
{:else if type === ButtonType.Secondary}
  <button
    on:click
    type="button"
    class="button-muted
      {flex} justify-center rounded {py} {px} {h}
      border-none 
      bg-background
      text-[#374151] {text}"
  >
    <slot />
  </button>
{:else if type === ButtonType.Thumb}
  <button on:click type="button" class="{flex} rounded-full shadow-md h-fit">
    <slot />
  </button>
{:else if type === ButtonType.Icon}
  <button on:click type="button" class="button-icon {flex} rounded-full h-fit">
    <slot />
  </button>
{/if}
