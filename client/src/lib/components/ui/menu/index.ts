import type { Snippet } from 'svelte';
import type { HTMLButtonAttributes } from 'svelte/elements';
import { tv, type VariantProps, type WithClass } from '../tv.js';

/**
 * Port of `.menu` (topbar account dropdown) and `.gridtile__menu` (tile overflow).
 * They differed only in width/radius/offset, so they are variants of one surface.
 * Positioning is left to the caller's `class` — both anchors are `relative` parents.
 */
export const menu = tv({
	base: 'absolute rounded-[10px] border border-edge bg-surface p-1 shadow-pop',
	variants: {
		size: {
			default: 'z-50 min-w-[200px]',
			compact: 'z-20 min-w-[170px] rounded-[8px]'
		}
	},
	defaultVariants: { size: 'default' }
});

/** Port of `.menu__item` + `.menu__item[data-danger='1']`. */
export const menuItem = tv({
	base: 'flex w-full cursor-pointer items-center gap-2.5 rounded-md border-0 bg-transparent px-2.5 py-[7px] text-left font-system text-[13.5px] transition-colors duration-100 hover:bg-row-hover [&_svg]:size-[14px]',
	variants: {
		danger: {
			true: 'text-danger [&_svg]:text-danger',
			false: 'text-ink [&_svg]:text-ink-muted'
		}
	},
	defaultVariants: { danger: false }
});

export type MenuSize = VariantProps<typeof menu>['size'];

export interface MenuProps {
	size?: MenuSize;
	class?: string;
	/** fly transition distance/duration — topbar used -6/150, grid tile -4/120. */
	flyY?: number;
	flyDuration?: number;
	children: Snippet;
}

export interface MenuItemProps extends WithClass<HTMLButtonAttributes> {
	danger?: boolean;
	children: Snippet;
}

export { default as Menu } from './Menu.svelte';
export { default as MenuItem } from './MenuItem.svelte';
export { default as MenuSeparator } from './MenuSeparator.svelte';
export { default as MenuHeader } from './MenuHeader.svelte';
