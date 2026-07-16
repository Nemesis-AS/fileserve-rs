import type { Action } from 'svelte/action';

/**
 * Calls `onOutside` when a mousedown lands outside the node.
 *
 * Replaces the identical hand-rolled `$effect` that lived in both TopBar and GridTile.
 * `mousedown` (not `click`) matches the original behaviour: menus close on press, so a
 * press-and-drag off the menu still dismisses it.
 *
 * Pass `enabled: false` to detach the listener entirely while the menu is closed.
 */
export const clickOutside: Action<
	HTMLElement,
	{ enabled?: boolean; onOutside: () => void } | undefined
> = (node, params) => {
	let current = params;

	function handler(e: MouseEvent) {
		if (!current || current.enabled === false) return;
		if (!node.contains(e.target as Node)) current.onOutside();
	}

	document.addEventListener('mousedown', handler);

	return {
		update(next) {
			current = next;
		},
		destroy() {
			document.removeEventListener('mousedown', handler);
		}
	};
};
