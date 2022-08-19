<script lang="ts">
	export let color: string = "#fff";
</script>
<svg
	style="width: 100%; color: {color};"
	class="animate-spin"
	xmlns="http://www.w3.org/2000/svg"
	fill="none"
	viewBox="0 0 24 24"
>
	<circle class="opacity-25" cx="12" cy="12" r="8" stroke="currentColor" stroke-width="3.2" />
	<path
		class="opacity-75"
		fill="currentColor"
		d="M 5.6 12 a 6.4 6.4 90 0 1 6.4 -6.4 V 2.4 C 6.6984 2.4 2.4 6.6984 2.4 12 h 3.2 z m 1.6 4.2328 A 6.3696 6.3696 90 0 1 5.6 12 H 2.4 c 0 2.4336 0.908 4.6592 2.4 6.3504 l 2.4 -2.1176 z"
	/>
</svg>
