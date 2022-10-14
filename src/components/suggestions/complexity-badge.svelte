<script lang="ts">
  export let complexity: number;

  function map_complexity_to_text(complexity: number): {
    text: string;
    color: string;
    bg_color: string;
  } {
    if (complexity < 10) {
      return { text: `Low`, color: 'text-signal_good', bg_color: 'bg-signal_good' };
    } else if (complexity < 15) {
      return {
        text: `Medium`,
        color: 'text-signal_medium',
        bg_color: 'bg-signal_medium',
      };
    } else if (complexity < 20) {
      return { text: `High`, color: 'text-signal_bad', bg_color: 'bg-signal_bad' };
    } else {
      return {
        text: `Very high`,
        color: 'text-signal_very_bad',
        bg_color: 'bg-signal_very_bad',
      };
    }
  }
  $: styled_complexity = map_complexity_to_text(complexity);
</script>

<div
  class="flex rounded items-center font-bold gap-[6px] py-[5px] pl-[8px] pr-[6px] {styled_complexity.color} text-sm border-[1.5px] border-current"
>
  {styled_complexity.text}
  <div class="rounded px-[6px] py-[3px] {styled_complexity.bg_color} text-background text-xs">
    {complexity}
  </div>
</div>
