<script lang="ts">
	import IconSwiftFormat from './icons/icon-swift-format.svelte';

	export let state: 'SwiftFormatFinished' | 'SwiftFormatFailed';
</script>

{#if state == 'SwiftFormatFailed'}
	<div class="shake-animation">
		<IconSwiftFormat fill="#FD7A58" blur={false} />
	</div>
{:else}
	<IconSwiftFormat fill="#FFFFFF" blur={true} />
{/if}

<style>
	.shake-animation {
		animation: shake 0.82s cubic-bezier(0.36, 0.07, 0.19, 0.97) both;
		transform: translate3d(0, 0, 0);
		backface-visibility: hidden;
		perspective: 1000px;
	}

	@keyframes shake {
		10%,
		90% {
			transform: translate3d(-0.5px, 0, 0);
		}

		20%,
		80% {
			transform: translate3d(1px, 0, 0);
		}

		30%,
		50%,
		70% {
			transform: translate3d(-3px, 0, 0);
		}

		40%,
		60% {
			transform: translate3d(3px, 0, 0);
		}
	}
</style>
