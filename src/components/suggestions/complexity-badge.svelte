<script lang="ts">
	export let complexity: number;

	function map_complexity_to_text(complexity: number): {
		text: string;
		color: string;
		bg_color: string;
	} {
		if (complexity < 10) {
			return { text: `Low`, color: 'text-signalgood', bg_color: 'bg-signalgood' };
		} else if (complexity < 15) {
			return {
				text: `Medium`,
				color: 'text-signalmedium',
				bg_color: 'bg-signalmedium'
			};
		} else if (complexity < 20) {
			return { text: `High`, color: 'text-signalbad', bg_color: 'bg-signalbad' };
		} else {
			return {
				text: `Very high`,
				color: 'text-signalverybad',
				bg_color: 'bg-signalverybad'
			};
		}
	}
	$: styled_complexity = map_complexity_to_text(complexity);
</script>

<div
	class="flex rounded items-center font-bold gap-[6px] py-[5px] pl-[8px] pr-[6px] {styled_complexity.color} text-sm border-[1.5px] border-current"
>
	{styled_complexity.text}
	<div class="rounded px-[3px] py-[3px] {styled_complexity.bg_color} text-background text-xs">
		{complexity}
	</div>
</div>
