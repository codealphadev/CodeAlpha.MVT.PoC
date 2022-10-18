import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { ChannelList } from '../src-tauri/bindings/ChannelList';
import type { EventWindowControls } from '../src-tauri/bindings/window_controls/EventWindowControls';
import type { HideAppWindowMessage } from '../src-tauri/bindings/window_controls/HideAppWindowMessage';
import type { ShowAppWindowMessage } from '../src-tauri/bindings/window_controls/ShowAppWindowMessage';

export const main_window_active_store = writable(false);

const listenToMainWindowEvents = async () => {
	let WindowControlsChannel: ChannelList = 'EventWindowControls';
	await listen(WindowControlsChannel, (e) => {
		const { event, payload } = JSON.parse(e.payload as string) as EventWindowControls;
		switch (event) {
			case 'AppWindowHide':
				const hide_msg = payload as HideAppWindowMessage;
				if (hide_msg.app_windows.includes('Main')) {
					main_window_active_store.set(false);
				}
				break;
			case 'AppWindowShow':
				const show_msg = payload as ShowAppWindowMessage;
				if (show_msg.app_windows.includes('Main')) {
					main_window_active_store.set(true);
				}
				break;
			default:
				break;
		}
	});
};
listenToMainWindowEvents();
