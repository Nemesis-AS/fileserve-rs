import { browser } from '$app/environment';
import type { Density, ViewMode } from '$lib/types';

function load<T>(key: string, fallback: T): T {
	if (!browser) return fallback;
	const v = localStorage.getItem(key);
	return v !== null ? (JSON.parse(v) as T) : fallback;
}

function save(key: string, value: unknown) {
	if (browser) localStorage.setItem(key, JSON.stringify(value));
}

let _dark = $state<boolean>(load('fileserve-dark', false));
let _density = $state<Density>(load('fileserve-density', 'cozy'));
let _view = $state<ViewMode>(load('fileserve-view', 'list'));
let _showSidebar = $state<boolean>(load('fileserve-sidebar', true));

export const prefs = {
	get dark() { return _dark; },
	set dark(v: boolean) { _dark = v; save('fileserve-dark', v); },

	get density() { return _density; },
	set density(v: Density) { _density = v; save('fileserve-density', v); },

	get view() { return _view; },
	set view(v: ViewMode) { _view = v; save('fileserve-view', v); },

	get showSidebar() { return _showSidebar; },
	set showSidebar(v: boolean) { _showSidebar = v; save('fileserve-sidebar', v); }
};
