import React, { useEffect } from "react";
import logo from "./logo.svg";
import tauriCircles from "./tauri.svg";
import tauriWord from "./wordmark.svg";
import "./App.css";

import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { sendNotification } from "@tauri-apps/api/notification";

function App() {
	useEffect(() => {
		const listenToGlobalEvents = async () => {
			await listen("ax-messages", (event) => {
				// event.event is the event name (useful if you want to use a single callback fn for multiple event types)
				// event.payload is the payload object
				sendNotification(JSON.stringify(event.event));
			});
		};

		invoke("plugin:awesome|register_again");

		// call the function
		listenToGlobalEvents()
			// make sure to catch any error
			.catch(console.error);
	}, []);

	return (
		<div className="App">
			<header className="App-header">
				<div className="inline-logo">
					<img src={tauriCircles} className="App-logo rotate" alt="logo" />
					<img src={tauriWord} className="App-logo smaller" alt="logo" />
				</div>
				<a
					className="App-link"
					href="https://tauri.studio"
					target="_blank"
					rel="noopener noreferrer"
				>
					Learn Tauri
				</a>
				<img src={logo} className="App-logo rotate" alt="logo" />
				<a
					className="App-link"
					href="https://reactjs.org"
					target="_blank"
					rel="noopener noreferrer"
				>
					Learn React
				</a>
				<p>
					Edit <code>src/App.tsx</code> and save to reload.
				</p>
			</header>
		</div>
	);
}

export default App;
