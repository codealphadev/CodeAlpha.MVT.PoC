import './app.postcss';
import App from './App.svelte';
import { MouseManager } from './mouse-manager';

const app = new App({
	target: document.body,
	props: {
		name: 'world'
	}
});

const mouse_manager = new MouseManager();
mouse_manager.init();

export default app;
