<div class="w-16" data-tauri-drag-region>
	<svg data-tauri-drag-region xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 81 81">
		<!-- Outer Background -->
		<path
			class="outerBG"
			fill="#D8D8D8"
			stroke="#8E8B94"
			stroke-width=".3"
			d="M80.85 40.602V75A5.85 5.85 0 0 1 75 80.85H6A5.85 5.85 0 0 1 .15 75V6A5.85 5.85 0 0 1 6 .15h34.5c22.285 0 40.35 18.02 40.35 40.248v.203Z"
		/>
		<!-- Inner Background -->
		<path
			class="innerBG"
			fill="#353535"
			d="M40.5 72C57.897 72 72 57.897 72 40.5S57.897 9 40.5 9H15a6 6 0 0 0-6 6v51a6 6 0 0 0 6 6h25.5Z"
		/>

		<!-- The 6 dots -->
		<ellipse cx="69.688" cy="64.75" fill="#8E8B94" rx="1.688" ry="1.75" />
		<ellipse cx="75.313" cy="64.75" fill="#8E8B94" rx="1.688" ry="1.75" />
		<ellipse cx="75.313" cy="70" fill="#8E8B94" rx="1.688" ry="1.75" />
		<ellipse cx="69.688" cy="70" fill="#8E8B94" rx="1.688" ry="1.75" />
		<ellipse cx="69.688" cy="75.25" fill="#8E8B94" rx="1.688" ry="1.75" />
		<ellipse cx="75.313" cy="75.25" fill="#8E8B94" rx="1.688" ry="1.75" />

		<!-- Logo Symbol -->
		<path
			class="logoSymbol"
			stroke="url(#coolGradient)"
			stroke-width="5"
			d="M21 40c0 9.941 8.02 18 17.911 18 6.993 0 13.05-4.027 16-9.9l-10.296-8.038M21 40c0-9.941 8.02-18 17.911-18C45.975 22 52.084 26.11 55 32.08l-10.385 7.982M21 40c0 4.97 4.01 9 8.956 9a8.89 8.89 0 0 0 5.374-1.8l9.285-7.138M21 40c0-4.97 4.01-9 8.956-9 1.907 0 3.674.599 5.127 1.62l9.532 7.442"
		/>
		<defs>
			<linearGradient
				id="coolGradient"
				x1="57.234"
				x2="22.755"
				y1="16.324"
				y2="58.261"
				gradientUnits="userSpaceOnUse"
			>
				<stop stop-color="#FF9C64" />
				<stop offset=".782" stop-color="#F84545" />
			</linearGradient>
		</defs>
	</svg>
</div>

<style>
	@keyframes vibrating {
		0% {
			opacity: 1;
		}
		30% {
			opacity: 1;
		}
		50% {
			opacity: 0.65;
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
