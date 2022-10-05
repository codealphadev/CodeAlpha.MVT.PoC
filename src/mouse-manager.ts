import { listen } from '@tauri-apps/api/event';
import type { ChannelList } from '../src-tauri/bindings/ChannelList';
import type { LogicalPosition } from '../src-tauri/bindings/geometry/LogicalPosition';
import type { EventWindowControls } from '../src-tauri/bindings/window_controls/EventWindowControls';
import type { TrackingAreaClickedMessage } from '../src-tauri/bindings/window_controls/TrackingAreaClickedMessage';
import type { TrackingAreaMouseOverMessage } from '../src-tauri/bindings/window_controls/TrackingAreaMouseOverMessage';

export class MouseManager {
	elements: HTMLElement[] = [];

	constructor() {}

	async init() {
		await this.listen_mouse_events();
	}

	private async listen_mouse_events() {
		await listen('EventWindowControls' as ChannelList, (event) => {
			let tracking_event = JSON.parse(event.payload as string) as EventWindowControls;

			switch (tracking_event.event) {
				case 'TrackingAreaMouseOver':
					this.mouse_over(tracking_event.payload);
					break;
				case 'TrackingAreaClicked':
					this.clicked(tracking_event.payload);
					break;
				case 'TrackingAreaClickedOutside':
					console.log('TrackingAreaClickedOutside', tracking_event);
					break;
			}
		});
	}

	private mouse_over(msg: TrackingAreaMouseOverMessage) {
		const new_elements = this.elements_from_point(msg.mouse_position);

		const removed_elements = this.elements.filter((element) => !new_elements.includes(element));
		removed_elements.forEach((element) => {
				element.dispatchEvent(new MouseEvent('mouseleave', { bubbles: false })); // For control logic
				element.classList.remove('hover'); // For styling
			
		});

		const added_elements = new_elements.filter((element) => !this.elements.includes(element));
		added_elements.forEach((element) => {
				element.dispatchEvent(new MouseEvent('mouseenter', { bubbles: false }));
				element.classList.add('hover');
		});

    this.elements = new_elements;
	}

	private clicked(msg: TrackingAreaClickedMessage) {
		document.elementsFromPoint(msg.mouse_position.x, msg.mouse_position.y).forEach((element) => {
			this.simulate_click_on_element(element);
		});
	}

	private simulate_click_on_element(element: Element) {
		element.dispatchEvent(new MouseEvent('click', { bubbles: false }));
	}

	private elements_from_point(position: LogicalPosition): HTMLElement[] {
		return document.elementsFromPoint(position.x, position.y).map((element) => {
			return element as HTMLElement;
		});
	}
}
