<script lang="ts">
	export let complexity: number;
</script>

{#if complexity > 15}
	<div class="p-4 bg-backgroundsecondary rounded-md w-full">
		<div class="flex gap-3 items-center">
			<div class="flex-shrink-0">
				<!-- Heroicon name: mini/exclamation-triangle -->
				<svg
					class="h-5 w-5 text-secondary"
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 20 20"
					fill="currentColor"
					aria-hidden="true"
				>
					<path
						fill-rule="evenodd"
						d="M8.485 3.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 3.495zM10 6a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 6zm0 9a1 1 0 100-2 1 1 0 000 2z"
						clip-rule="evenodd"
					/>
				</svg>
			</div>
			<div>
				<p class="text-xs text-contrast">
					This function may be too <span class="decoration-dotted underline-offset-2" /> complex. Consider
					refactoring.
				</p>
			</div>
		</div>
	</div>
{/if}
