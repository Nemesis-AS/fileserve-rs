import { tv, type VariantProps } from '../tv.js';

/**
 * `.quota-bar` (sidebar 4px / user table 3px) and `.progressbar` (upload row 3px) were
 * the same control with different heights, so they collapse into one.
 *
 * Track radius differed — quota-bar was a pill, progressbar 2px — hence the `radius`
 * variant rather than tying radius to size.
 */
export const meter = tv({
	slots: {
		track: 'overflow-hidden bg-sunken',
		bar: 'block h-full'
	},
	variants: {
		size: {
			xs: { track: 'h-[3px]' },
			sm: { track: 'h-1' }
		},
		radius: {
			pill: { track: 'rounded-full', bar: 'rounded-full' },
			sharp: { track: 'rounded-[2px]', bar: '' }
		},
		/** Upload rows animate width on progress; the sidebar quota eases slower. */
		speed: {
			slow: { bar: 'transition-[width] duration-300 ease-out' },
			fast: { bar: 'transition-[width] duration-200' },
			none: { bar: '' }
		}
	},
	defaultVariants: { size: 'sm', radius: 'pill', speed: 'slow' }
});

export type MeterSize = VariantProps<typeof meter>['size'];

export interface MeterProps {
	/** 0-100. Clamped. */
	value: number;
	size?: MeterSize;
	radius?: 'pill' | 'sharp';
	speed?: 'slow' | 'fast' | 'none';
	/** Defaults to the accent token; upload rows pass a per-state colour. */
	color?: string;
	class?: string;
}

export { default as Meter } from './Meter.svelte';
