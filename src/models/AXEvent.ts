class AXEvent {
	id: number;
	timestamp: Date;
	eventName: string;
	messageType: string;
	payload: any;

	constructor(
		id: number,
		eventName: string,
		messageType: string,
		payload: any,
	) {
		this.id = id * Math.random();
		this.timestamp = new Date();
		this.eventName = eventName;
		this.messageType = messageType;
		this.payload = payload;
	}
}

export default AXEvent;
