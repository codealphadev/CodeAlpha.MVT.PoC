<svg
	xmlns="http://www.w3.org/2000/svg"
	fill="none"
	style="width: 100%, height: 100%"
	viewBox="0 0 61 61"
>
	<path
		class="logoSymbol"
		fill="url(#dark)"
		fill-rule="evenodd"
		d="M19.12 17.41a13.21 13.21 0 0 1 9.23 2.27l.06.05 9.23 7.21 8.17-6.28a17.63 17.63 0 0 0-26.69-3.25Zm23.2 13.18 10.82-8.33-1.03-2.1A23.4 23.4 0 0 0 31.1 6.97 23.47 23.47 0 0 0 7.69 30.5a23.47 23.47 0 0 0 23.4 23.52 23.4 23.4 0 0 0 20.9-12.94l1.06-2.1-10.74-8.4Zm-4.7 3.61-8.9 6.84-.01.01a13.05 13.05 0 0 1-9.59 2.54 17.62 17.62 0 0 0 26.58-3.07l-8.08-6.32Zm-4.67-3.65L25 24.35a7.32 7.32 0 0 0-4.18-1.3 7.43 7.43 0 0 0-7.4 7.45 7.43 7.43 0 0 0 7.4 7.46 7.3 7.3 0 0 0 4.42-1.48l7.7-5.93Z"
		clip-rule="evenodd"
	/>
	<defs>
		<linearGradient
			id="dark"
			x1="49.28"
			x2="13.18"
			y1="5.31"
			y2="48.03"
			gradientUnits="userSpaceOnUse"
		>
			<stop stop-color="#FF9C64" />
			<stop offset=".78" stop-color="#F84545" />
		</linearGradient>
	</defs>
</svg>

<style>
	@keyframes vibrating {
		0% {
			opacity: 1;
		}
		30% {
			opacity: 1;
		}
		50% {
			opacity: 0.75;
		}
		70% {
			opacity: 1;
		}
		100% {
			opacity: 1;
		}
	}

	.logoSymbol {
		/* animation: vibrating 3s ease infinite; */
		stroke-dasharray: 450;
		stroke-dashoffset: 450;
		animation: draw 2s linear forwards;

		animation-name: draw, vibrating;
		animation-duration: 2000ms, 2500ms;
		animation-delay: 0ms, 2500ms;
		animation-timing-function: ease-in, ease;
		animation-iteration-count: 1, infinite;
	}

	@keyframes draw {
		to {
			stroke-dashoffset: 0;
		}
	}
</style>
