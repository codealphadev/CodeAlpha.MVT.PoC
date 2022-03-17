export { validators } from './client-validators.js';

export const components = [
	() => import("../runtime/components/layout.svelte"),
	() => import("../runtime/components/error.svelte"),
	() => import("../../src/routes/index.svelte"),
	() => import("../../src/routes/pages/analytics.svelte"),
	() => import("../../src/routes/pages/duder.svelte")
];

export const dictionary = {
	"": [[0, 2], [1]],
	"pages/analytics": [[0, 3], [1]],
	"pages/duder": [[0, 4], [1]]
};