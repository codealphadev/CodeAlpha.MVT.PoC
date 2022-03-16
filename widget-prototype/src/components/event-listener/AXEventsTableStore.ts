import { writable, Writable } from "svelte/store";
import AXEvent from "../../models/AXEvent";
import { Event, listen } from "@tauri-apps/api/event";

class AXEventTableStore {
	constructor(
		public inspectEventId: Writable<number> = writable(0),
		public inspectModal: Writable<boolean> = writable(false),
		public axEventList: Writable<Array<AXEvent>> = writable([]),
	) {}
}

export const axEventTableStore = new AXEventTableStore();

const addAXEvent = (tauriEvent: Event<unknown>) => {
	axEventTableStore.axEventList.update((n) => {
		n.push(
			new AXEvent(
				tauriEvent.id,
				tauriEvent.event,
				tauriEvent.event.substring(tauriEvent.event.indexOf("-") + 1),
				tauriEvent.payload,
			),
		);
		return n;
	});
};

const listenToGlobalEvents = async () => {
	await listen("StateEvent-AppFocusState", (event) => {
		console.log("StateEvent-AppFocusState", event);

		addAXEvent(event);
	});

	await listen("StateEvent-XCodeFocusStatus", (event) => {
		addAXEvent(event);
	});

	await listen("StateEvent-XCodeFocusStatusChange", (event) => {
		addAXEvent(event);
	});

	await listen("StateEvent-XCodeEditorContent", (event) => {
		addAXEvent(event);
	});
};

listenToGlobalEvents()
	// make sure to catch any error
	.catch(console.error);
