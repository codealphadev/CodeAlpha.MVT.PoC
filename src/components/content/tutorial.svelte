<script lang="ts">
	import { SwitchHorizontalIcon } from '@rgossiaux/svelte-heroicons/outline';
	import { invoke } from '@tauri-apps/api/tauri';
</script>

<form class="space-y-3">
	<div>
		<h3 class="text-lg leading-6 font-medium text-gray-900 pt-4 outline-none">
			Here is where you left off!
		</h3>
		<p class="mt-2 text-base font-normal text-gray-500">
			I make sure to not play the video when you are focused on coding.
		</p>
	</div>

	<div class=" flex flex-wrap justify-center">
		<div>
			<!-- svelte-ignore a11y-missing-attribute -->
			<iframe
				class="rounded-md shadow-md"
				src="https://www.youtube.com/embed/nAchMctX4YA?controls=0&amp;start=68"
				width="290"
				frameborder="0"
				allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
				allowfullscreen
			/>
		</div>
	</div>
</form>
