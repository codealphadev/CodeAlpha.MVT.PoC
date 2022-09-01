<script lang="ts">
	export let fill: string;
	export let blur: boolean;
</script>

<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 47 47">
	<g filter="url(#a)">
		<path
			{fill}
			d="m32.65 13.35.7-.7-.7.7Zm-16.07 20.2v1a1 1 0 0 0 .71-.3l-.7-.7Zm-4.08 0h-1a1 1 0 0 0 1 1v-1Zm0-4.17-.7-.71a1 1 0 0 0-.3.7h1Zm16.73-15.32a1.92 1.92 0 0 1 2.7 0l1.42-1.41a3.92 3.92 0 0 0-5.54 0l1.42 1.41Zm2.7 0a1.9 1.9 0 0 1 0 2.71l1.42 1.42a3.92 3.92 0 0 0 0-5.54l-1.41 1.41Zm0 2.71L15.89 32.83l1.41 1.42 16.06-16.06-1.41-1.42ZM16.59 32.54H12.5v2h4.08v-2Zm11.23-19.9L11.8 28.68l1.42 1.41 16.02-16.02-1.42-1.41ZM11.5 29.39v4.16h2v-4.16h-2ZM26.06 15.8l4.13 4.13 1.41-1.42-4.12-4.12-1.42 1.41Z"
		/>
	</g>
	<defs>
		<filter
			id="a"
			width="34"
			height="34"
			x="6"
			y="6"
			color-interpolation-filters="sRGB"
			filterUnits="userSpaceOnUse"
		>
			<feFlood flood-opacity="0" result="BackgroundImageFix" />
			<feColorMatrix
				in="SourceAlpha"
				result="hardAlpha"
				values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
			/>
			<feOffset />
			<feGaussianBlur stdDeviation="1.5" />
			<feComposite in2="hardAlpha" operator="out" />
			{#if blur}
				<feColorMatrix values="0 0 0 0 1 0 0 0 0 1 0 0 0 0 1 0 0 0 0.8 0" />
			{:else}
				<feColorMatrix values="0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0.8 0" />
			{/if}
			<feBlend in2="BackgroundImageFix" result="effect1_dropShadow_866_5421" />
			<feBlend in="SourceGraphic" in2="effect1_dropShadow_866_5421" result="shape" />
		</filter>
	</defs>
</svg>
