<script lang="ts">
	import { Event, listen } from '@tauri-apps/api/event';
	import Tail from '../components/main/tail.svelte';
	import type { ChannelList } from '../../src-tauri/bindings/ChannelList';
	import type { EventViewport } from '../../src-tauri/bindings/macOS_specific/EventViewport';
	import Suggestions from '../components/suggestions/suggestions.svelte';
	import type { TailOrientation } from '../../src-tauri/bindings/window_controls/TailOrientation';

	let dom_id = 'main-window-container';
	let active_window_uid: number | null = null;

	let tail_orientation: TailOrientation = 'Right';
	const listenToGlobalEvents = async () => {
		await listen('tail-orientation-changed', (event) => {
			tail_orientation = (event as Event<any>).payload as TailOrientation;
		});
	};

	listenToGlobalEvents();

	const listenToViewportEvents = async () => {
		let ViewportChannel: ChannelList = 'EventViewport';
		await listen(ViewportChannel, (e) => {
			const { event, payload } = JSON.parse(e.payload as string) as EventViewport;

			switch (event) {
				case 'XcodeViewportUpdate':
					active_window_uid = payload.viewport_properties.window_uid;
					break;
				default:
					break;
			}
		});
	};

	listenToViewportEvents();
</script>

{#if active_window_uid}
	<div class="bg-transparent flex flex-col shrink-0 justify-end transform overflow-hidden h-full">
		<div
			id={dom_id}
			class="bg-background flex flex-col shrink-0 rounded-xl max-h-[800px] overflow-y-auto overscroll-none"
		>
			<Suggestions window_dom_id={dom_id} {active_window_uid} />
		</div>
		<div class="h-3 {`${tail_orientation == 'Left' ? 'mr-auto ml-[18px]' : 'ml-auto mr-[18px]'}`}">
			<Tail />
		</div>
	</div>
{/if}
